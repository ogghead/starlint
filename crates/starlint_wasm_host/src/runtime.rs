//! WASM runtime using wasmtime.
//!
//! Manages the wasmtime engine, store, and plugin instances.
//! [`WasmPluginHost`] loads WASM component plugins and implements
//! [`PluginHost`](starlint_core::plugin::PluginHost) for integration
//! with the lint engine.

// Generated WIT bindings for the `linter-plugin` world.
//
// The `bindgen!` macro generates Rust types and traits from `wit/plugin.wit`.
// Generated code triggers many clippy lints that our strict config denies,
// so we suppress them on the generated module.
#[cfg(feature = "wasm")]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::wildcard_imports,
    clippy::as_conversions,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::pub_without_shorthand,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_pass_by_value,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::future_not_send,
    clippy::arithmetic_side_effects,
    missing_docs
)]
pub(crate) mod bindings {
    wasmtime::component::bindgen!({
        world: "linter-plugin",
        path: "../../wit",
    });
}

/// Resource limits for WASM plugins.
pub struct ResourceLimits {
    /// Maximum fuel (instruction count) per file per plugin.
    pub fuel_per_file: u64,
    /// Maximum memory in bytes.
    pub max_memory_bytes: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            fuel_per_file: 10_000_000,
            max_memory_bytes: 16 * 1024 * 1024, // 16 MB
        }
    }
}

// ---------- Full WASM host implementation ----------

#[cfg(feature = "wasm")]
/// WASM host implementation with wasmtime engine, plugin loading, and linting.
mod host {
    use std::collections::HashSet;
    use std::path::Path;

    use globset::{Glob, GlobSet, GlobSetBuilder};
    use oxc_ast::ast::Program;
    use oxc_ast_visit::Visit;
    use wasmtime::component::{Component, Linker};
    use wasmtime::{Engine, Store, StoreLimits, StoreLimitsBuilder};

    use starlint_core::plugin::PluginHost;
    use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};

    use super::ResourceLimits;
    use super::bindings::LinterPluginPre;
    use super::bindings::starlint::plugin::types as wit;
    use crate::bridge::{NodeInterest, WitAstNode};
    use crate::builtins;
    use crate::collector::NodeCollector;
    use crate::error::WasmError;

    /// Store host data carrying resource limits for wasmtime.
    struct HostData {
        /// Memory and resource limits enforced by wasmtime.
        limits: StoreLimits,
    }

    /// A loaded WASM plugin ready for linting.
    struct LoadedPlugin {
        /// Pre-instantiated component (cheap to instantiate per-file).
        pre: LinterPluginPre<HostData>,
        /// Cached node interests from `get-node-interests()`.
        interests: NodeInterest,
        /// Cached file-pattern filter from `get-file-patterns()`.
        /// `None` means the plugin applies to all files.
        file_patterns: Option<GlobSet>,
        /// Plugin name (derived from filename).
        name: String,
        /// Plugin configuration JSON (applied per-file in the same store as linting).
        config_json: String,
    }

    /// WASM plugin host powered by wasmtime.
    ///
    /// Loads WASM component plugins and runs them against files in parallel.
    /// Each file gets a fresh [`Store`] with fuel and memory limits for safety.
    pub struct WasmPluginHost {
        /// Wasmtime engine (shared, thread-safe).
        engine: Engine,
        /// Component linker (no imports needed for linter-plugin world).
        linker: Linker<HostData>,
        /// Loaded plugins.
        plugins: Vec<LoadedPlugin>,
        /// Resource limits per plugin per file.
        limits: ResourceLimits,
    }

    impl WasmPluginHost {
        /// Create a new WASM plugin host with the given resource limits.
        pub fn new(limits: ResourceLimits) -> Result<Self, WasmError> {
            let mut config = wasmtime::Config::new();
            config.consume_fuel(true);
            config.wasm_component_model(true);

            let engine = Engine::new(&config).map_err(|err| WasmError::CompileFailed {
                path: "<engine>".to_owned(),
                reason: err.to_string(),
            })?;

            let linker = Linker::new(&engine);

            Ok(Self {
                engine,
                linker,
                plugins: Vec::new(),
                limits,
            })
        }

        /// Create a host with default limits and load plugins from `(path, config_json)` pairs.
        ///
        /// Convenience constructor that replaces the duplicated `build_plugin_host`
        /// logic in CLI and LSP.
        pub fn with_plugins(
            plugins: &[(&Path, &str)],
        ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
            let mut host = Self::new(ResourceLimits::default())?;
            for &(path, config_json) in plugins {
                host.load_plugin(path, config_json)?;
            }
            Ok(host)
        }

        /// Load a WASM component plugin from disk.
        ///
        /// Compiles the component, pre-instantiates it, and caches metadata
        /// (rules, node interests) for later use.
        pub fn load_plugin(&mut self, path: &Path, config_json: &str) -> Result<(), WasmError> {
            // Validate path early for better error messages (existence + .wasm extension).
            crate::loader::validate_plugin_path(path)?;

            let bytes = std::fs::read(path).map_err(|err| WasmError::LoadFailed {
                path: path.display().to_string(),
                reason: err.to_string(),
            })?;

            let plugin_name = plugin_name_from_path(path);
            self.load_plugin_bytes(&plugin_name, &bytes, config_json)
        }

        /// Load a WASM component plugin from raw bytes.
        ///
        /// Same as [`load_plugin`](Self::load_plugin) but takes pre-read WASM bytes
        /// instead of a file path. Used by the builtin plugin system to load
        /// embedded WASM plugins without disk I/O.
        pub fn load_plugin_from_bytes(
            &mut self,
            name: &str,
            bytes: &[u8],
            config_json: &str,
        ) -> Result<(), WasmError> {
            self.load_plugin_bytes(name, bytes, config_json)
        }

        /// Shared implementation for loading a plugin from raw WASM bytes.
        fn load_plugin_bytes(
            &mut self,
            plugin_name: &str,
            bytes: &[u8],
            config_json: &str,
        ) -> Result<(), WasmError> {
            let component =
                Component::new(&self.engine, bytes).map_err(|err| WasmError::CompileFailed {
                    path: plugin_name.to_owned(),
                    reason: err.to_string(),
                })?;

            let instance_pre = self.linker.instantiate_pre(&component).map_err(|err| {
                WasmError::CompileFailed {
                    path: plugin_name.to_owned(),
                    reason: err.to_string(),
                }
            })?;

            let pre =
                LinterPluginPre::new(instance_pre).map_err(|err| WasmError::CompileFailed {
                    path: plugin_name.to_owned(),
                    reason: err.to_string(),
                })?;

            // Query metadata from the plugin using a temporary store.
            let (interests, file_patterns, _rules) =
                query_plugin_metadata(&pre, &self.engine, plugin_name, &self.limits)?;

            // Validate config eagerly (in a throwaway store) so load_plugin fails fast.
            // The config is then re-applied per-file in the same store as linting.
            if !config_json.is_empty() {
                validate_config(&pre, &self.engine, plugin_name, config_json, &self.limits)?;
            }

            self.plugins.push(LoadedPlugin {
                pre,
                interests,
                file_patterns,
                name: plugin_name.to_owned(),
                config_json: config_json.to_owned(),
            });

            Ok(())
        }

        /// Return the number of loaded plugins.
        ///
        /// Useful for verifying deduplication in tests.
        #[must_use]
        pub fn plugin_count(&self) -> usize {
            self.plugins.len()
        }

        /// Load all active builtin plugins from embedded WASM bytes.
        ///
        /// Deduplicates automatically: `import`, `node`, and `promise` all map
        /// to the single `modules` plugin, so enabling all three loads it once.
        /// Unknown builtin names are silently skipped with a warning.
        pub fn load_builtins(&mut self, active: &HashSet<String>) -> Result<(), WasmError> {
            // Deduplicate config names → WASM plugin names.
            let wasm_names: HashSet<&str> = active
                .iter()
                .filter_map(|name| builtins::config_to_wasm_name(name))
                .collect();

            for wasm_name in wasm_names {
                if let Some(bytes) = builtins::get_builtin_bytes(wasm_name) {
                    self.load_plugin_from_bytes(wasm_name, bytes, "")?;
                } else {
                    tracing::warn!("unknown builtin plugin: {wasm_name}");
                }
            }

            Ok(())
        }

        /// Lint a single file with all loaded plugins.
        fn lint_file_internal(
            &self,
            file_path: &Path,
            source_text: &str,
            program: &Program<'_>,
        ) -> Vec<Diagnostic> {
            let mut all_diagnostics = Vec::new();

            for plugin in &self.plugins {
                match lint_with_plugin(
                    plugin,
                    &self.engine,
                    &self.limits,
                    file_path,
                    source_text,
                    program,
                ) {
                    Ok(diags) => all_diagnostics.extend(diags),
                    Err(err) => {
                        tracing::warn!(
                            "plugin '{}' failed on {}: {err}",
                            plugin.name,
                            file_path.display()
                        );
                    }
                }
            }

            all_diagnostics
        }
    }

    impl PluginHost for WasmPluginHost {
        fn lint_file(
            &self,
            file_path: &Path,
            source_text: &str,
            program: &Program<'_>,
        ) -> Vec<Diagnostic> {
            self.lint_file_internal(file_path, source_text, program)
        }
    }

    // ---- Helper functions ----

    /// Derive a plugin name from its file path.
    fn plugin_name_from_path(path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_owned()
    }

    /// Query a plugin's rules, node interests, and file patterns using a temporary store.
    fn query_plugin_metadata(
        pre: &LinterPluginPre<HostData>,
        engine: &Engine,
        plugin_name: &str,
        limits: &ResourceLimits,
    ) -> Result<(NodeInterest, Option<GlobSet>, Vec<wit::RuleMeta>), WasmError> {
        let mut store = create_store(engine, limits);
        let instance = pre
            .instantiate(&mut store)
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin_name.to_owned(),
                reason: err.to_string(),
            })?;

        let guest = instance.starlint_plugin_plugin();

        let interests_wit =
            guest
                .call_get_node_interests(&mut store)
                .map_err(|err| WasmError::RuntimeError {
                    plugin_name: plugin_name.to_owned(),
                    reason: err.to_string(),
                })?;

        let rules = guest
            .call_get_rules(&mut store)
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin_name.to_owned(),
                reason: err.to_string(),
            })?;

        let file_patterns_raw =
            guest
                .call_get_file_patterns(&mut store)
                .map_err(|err| WasmError::RuntimeError {
                    plugin_name: plugin_name.to_owned(),
                    reason: err.to_string(),
                })?;

        let interests = wit_interests_to_bridge(interests_wit);
        let file_patterns = compile_file_patterns(&file_patterns_raw, plugin_name);

        Ok((interests, file_patterns, rules))
    }

    /// Compile glob patterns from a plugin into a `GlobSet`.
    /// Returns `None` if the pattern list is empty (matches all files).
    fn compile_file_patterns(patterns: &[String], plugin_name: &str) -> Option<GlobSet> {
        if patterns.is_empty() {
            return None;
        }

        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            match Glob::new(pattern) {
                Ok(glob) => {
                    builder.add(glob);
                }
                Err(err) => {
                    tracing::warn!(
                        "plugin '{plugin_name}': invalid file pattern '{pattern}': {err}"
                    );
                }
            }
        }

        match builder.build() {
            Ok(set) if !set.is_empty() => Some(set),
            Ok(_) => None,
            Err(err) => {
                tracing::warn!("plugin '{plugin_name}': failed to compile file patterns: {err}");
                None
            }
        }
    }

    /// Validate plugin config eagerly so `load_plugin` fails fast on bad config.
    ///
    /// This uses a throwaway store — the config is re-applied per-file in
    /// [`lint_with_plugin`] so it actually takes effect in the same WASM
    /// instance that runs `lint-file`.
    fn validate_config(
        pre: &LinterPluginPre<HostData>,
        engine: &Engine,
        plugin_name: &str,
        config_json: &str,
        limits: &ResourceLimits,
    ) -> Result<(), WasmError> {
        let mut store = create_store(engine, limits);
        let instance = pre
            .instantiate(&mut store)
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin_name.to_owned(),
                reason: err.to_string(),
            })?;

        let guest = instance.starlint_plugin_plugin();
        let errors = guest
            .call_configure(&mut store, &config_json.to_owned())
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin_name.to_owned(),
                reason: err.to_string(),
            })?;

        if !errors.is_empty() {
            return Err(WasmError::ConfigRejected {
                plugin_name: plugin_name.to_owned(),
                errors: errors.join("; "),
            });
        }

        Ok(())
    }

    /// Lint a file with a single plugin.
    fn lint_with_plugin(
        plugin: &LoadedPlugin,
        engine: &Engine,
        limits: &ResourceLimits,
        file_path: &Path,
        source_text: &str,
        program: &Program<'_>,
    ) -> Result<Vec<Diagnostic>, WasmError> {
        // Skip if plugin declares file patterns and this file doesn't match.
        if let Some(ref patterns) = plugin.file_patterns {
            if !patterns.is_match(file_path) {
                return Ok(Vec::new());
            }
        }

        // Skip if plugin has no relevant interests.
        if !plugin.interests.any() {
            return Ok(Vec::new());
        }

        // Collect matching AST nodes.
        let mut collector = NodeCollector::new(plugin.interests);
        collector.visit_program(program);
        let bridge_nodes = collector.into_nodes();

        // Skip calling the plugin if no matching nodes were found
        // AND the plugin doesn't need source-text access.
        if bridge_nodes.is_empty() && !plugin.interests.source_text {
            return Ok(Vec::new());
        }

        // Convert to WIT types and build the batch.
        let wit_batch = build_wit_batch(file_path, source_text, &bridge_nodes);

        // Create a fresh store with fuel + memory limits, configure, then lint.
        // Config is applied in the same store so WASM linear memory state persists.
        let mut store = create_store(engine, limits);
        let instance =
            plugin
                .pre
                .instantiate(&mut store)
                .map_err(|err| WasmError::RuntimeError {
                    plugin_name: plugin.name.clone(),
                    reason: err.to_string(),
                })?;

        let guest = instance.starlint_plugin_plugin();

        // Apply plugin configuration in the same store as linting.
        if !plugin.config_json.is_empty() {
            guest
                .call_configure(&mut store, &plugin.config_json)
                .map_err(|err| WasmError::RuntimeError {
                    plugin_name: plugin.name.clone(),
                    reason: err.to_string(),
                })?;
        }

        let wit_diags = guest
            .call_lint_file(&mut store, &wit_batch)
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin.name.clone(),
                reason: err.to_string(),
            })?;

        // Convert WIT diagnostics to SDK diagnostics.
        Ok(wit_diags.into_iter().map(wit_diagnostic_to_sdk).collect())
    }

    /// Create a wasmtime Store with fuel and memory limits.
    fn create_store(engine: &Engine, limits: &ResourceLimits) -> Store<HostData> {
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes)
            .build();
        let host_data = HostData {
            limits: store_limits,
        };
        let mut store = Store::new(engine, host_data);
        store.limiter(|data| &mut data.limits);
        #[allow(clippy::let_underscore_must_use)]
        let _ = store.set_fuel(limits.fuel_per_file);
        store
    }

    // ---- Type conversion functions ----

    /// Convert WIT `NodeInterest` flags to bridge `NodeInterest` bools.
    fn wit_interests_to_bridge(wit: wit::NodeInterest) -> NodeInterest {
        NodeInterest {
            import_declaration: wit.contains(wit::NodeInterest::IMPORT_DECLARATION),
            export_default_declaration: wit.contains(wit::NodeInterest::EXPORT_DEFAULT_DECLARATION),
            export_named_declaration: wit.contains(wit::NodeInterest::EXPORT_NAMED_DECLARATION),
            call_expression: wit.contains(wit::NodeInterest::CALL_EXPRESSION),
            member_expression: wit.contains(wit::NodeInterest::MEMBER_EXPRESSION),
            identifier_reference: wit.contains(wit::NodeInterest::IDENTIFIER_REFERENCE),
            arrow_function_expression: wit.contains(wit::NodeInterest::ARROW_FUNCTION_EXPRESSION),
            function_declaration: wit.contains(wit::NodeInterest::FUNCTION_DECLARATION),
            variable_declaration: wit.contains(wit::NodeInterest::VARIABLE_DECLARATION),
            string_literal: wit.contains(wit::NodeInterest::STRING_LITERAL),
            object_expression: wit.contains(wit::NodeInterest::OBJECT_EXPRESSION),
            array_expression: wit.contains(wit::NodeInterest::ARRAY_EXPRESSION),
            debugger_statement: wit.contains(wit::NodeInterest::DEBUGGER_STATEMENT),
            jsx_opening_element: wit.contains(wit::NodeInterest::JSX_OPENING_ELEMENT),
            source_text: wit.contains(wit::NodeInterest::SOURCE_TEXT),
        }
    }

    /// Build a WIT `NodeBatch` from bridge types.
    fn build_wit_batch(
        file_path: &Path,
        source_text: &str,
        bridge_nodes: &[WitAstNode],
    ) -> wit::NodeBatch {
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_owned();

        let file_context = wit::FileContext {
            file_path: file_path.display().to_string(),
            source_text: source_text.to_owned(),
            extension,
        };

        let nodes = bridge_nodes.iter().map(bridge_node_to_wit).collect();

        wit::NodeBatch {
            file: file_context,
            nodes,
        }
    }

    /// Convert a bridge AST node to a WIT AST node.
    #[allow(clippy::too_many_lines)]
    fn bridge_node_to_wit(node: &WitAstNode) -> wit::AstNode {
        match node {
            WitAstNode::ImportDecl(import) => {
                wit::AstNode::ImportDecl(wit::ImportDeclarationNode {
                    span: to_wit_span(import.span),
                    source: import.source.clone(),
                    specifiers: import
                        .specifiers
                        .iter()
                        .map(|s| wit::ImportSpecifier {
                            local: s.local.clone(),
                            imported: s.imported.clone(),
                        })
                        .collect(),
                })
            }
            WitAstNode::DebuggerStmt(stmt) => {
                wit::AstNode::DebuggerStmt(wit::DebuggerStatementNode {
                    span: to_wit_span(stmt.span),
                })
            }
            WitAstNode::CallExpr(call) => wit::AstNode::CallExpr(wit::CallExpressionNode {
                span: to_wit_span(call.span),
                callee_path: call.callee_path.clone(),
                argument_count: call.argument_count,
                is_awaited: call.is_awaited,
            }),
            WitAstNode::ExportDefaultDecl(export) => {
                wit::AstNode::ExportDefaultDecl(wit::ExportDefaultNode {
                    span: to_wit_span(export.span),
                })
            }
            WitAstNode::ExportNamedDecl(export) => {
                wit::AstNode::ExportNamedDecl(wit::ExportNamedNode {
                    span: to_wit_span(export.span),
                    names: export.names.clone(),
                })
            }
            WitAstNode::MemberExpr(member) => wit::AstNode::MemberExpr(wit::MemberExpressionNode {
                span: to_wit_span(member.span),
                object: member.object.clone(),
                property: member.property.clone(),
                computed: member.computed,
            }),
            WitAstNode::IdentifierRef(id) => {
                wit::AstNode::IdentifierRef(wit::IdentifierReferenceNode {
                    span: to_wit_span(id.span),
                    name: id.name.clone(),
                })
            }
            WitAstNode::ArrowFnExpr(arrow) => {
                wit::AstNode::ArrowFnExpr(wit::ArrowFunctionExpressionNode {
                    span: to_wit_span(arrow.span),
                    params_count: arrow.params_count,
                    is_async: arrow.is_async,
                    is_expression: arrow.is_expression,
                })
            }
            WitAstNode::FnDecl(func) => wit::AstNode::FnDecl(wit::FunctionDeclarationNode {
                span: to_wit_span(func.span),
                name: func.name.clone(),
                params_count: func.params_count,
                is_async: func.is_async,
                is_generator: func.is_generator,
            }),
            WitAstNode::VarDecl(var) => wit::AstNode::VarDecl(wit::VariableDeclarationNode {
                span: to_wit_span(var.span),
                kind: var.kind.clone(),
                declarations: var
                    .declarations
                    .iter()
                    .map(|d| wit::VariableDeclarator {
                        name: d.name.clone(),
                        has_init: d.has_init,
                    })
                    .collect(),
            }),
            WitAstNode::StringLit(lit) => wit::AstNode::StringLit(wit::StringLiteralNode {
                span: to_wit_span(lit.span),
                value: lit.value.clone(),
            }),
            WitAstNode::ObjectExpr(obj) => wit::AstNode::ObjectExpr(wit::ObjectExpressionNode {
                span: to_wit_span(obj.span),
                property_count: obj.property_count,
            }),
            WitAstNode::ArrayExpr(arr) => wit::AstNode::ArrayExpr(wit::ArrayExpressionNode {
                span: to_wit_span(arr.span),
                element_count: arr.element_count,
            }),
            WitAstNode::JsxElement(el) => wit::AstNode::JsxElement(wit::JsxOpeningElementNode {
                span: to_wit_span(el.span),
                name: el.name.clone(),
                attributes: el
                    .attributes
                    .iter()
                    .map(|a| wit::JsxAttribute {
                        name: a.name.clone(),
                        value: a.value.clone(),
                        is_spread: a.is_spread,
                    })
                    .collect(),
                self_closing: el.self_closing,
                children_count: el.children_count,
            }),
        }
    }

    /// Convert an SDK `Span` to a WIT `Span`.
    const fn to_wit_span(span: Span) -> wit::Span {
        wit::Span {
            start: span.start,
            end: span.end,
        }
    }

    /// Convert a WIT `LintDiagnostic` to an SDK `Diagnostic`.
    fn wit_diagnostic_to_sdk(diag: wit::LintDiagnostic) -> Diagnostic {
        Diagnostic {
            rule_name: diag.rule_name,
            message: diag.message,
            span: Span::new(diag.span.start, diag.span.end),
            severity: match diag.severity {
                wit::Severity::Warning => Severity::Warning,
                wit::Severity::Error => Severity::Error,
                wit::Severity::Suggestion => Severity::Suggestion,
            },
            help: diag.help,
            fix: None,
            labels: vec![],
        }
    }
}

#[cfg(feature = "wasm")]
pub use host::WasmPluginHost;

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(
            limits.fuel_per_file, 10_000_000,
            "default fuel should be 10M"
        );
        assert_eq!(
            limits.max_memory_bytes,
            16 * 1024 * 1024,
            "default memory should be 16MB"
        );
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_wasm_host_creation() {
        let host = WasmPluginHost::new(ResourceLimits::default());
        assert!(host.is_ok(), "should create WASM host successfully");
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_load_nonexistent_plugin() {
        let mut host = WasmPluginHost::new(ResourceLimits::default());
        assert!(host.is_ok(), "should create WASM host successfully");

        if let Ok(ref mut h) = host {
            let result = h.load_plugin(std::path::Path::new("/nonexistent/plugin.wasm"), "");
            assert!(result.is_err(), "loading nonexistent plugin should fail");
        }
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_bindings_types_exist() {
        // Verify the generated WIT types compile and are accessible.
        use bindings::starlint::plugin::types::{
            AstNode, Category, FileContext, JsxAttribute, JsxOpeningElementNode, LintDiagnostic,
            NodeBatch, NodeInterest, RuleMeta, Severity, Span,
        };
        use bindings::{LinterPlugin, LinterPluginPre};
        let _ = (
            core::any::type_name::<LinterPlugin>(),
            core::any::type_name::<LinterPluginPre<()>>(),
            core::any::type_name::<Span>(),
            core::any::type_name::<Severity>(),
            core::any::type_name::<Category>(),
            core::any::type_name::<RuleMeta>(),
            core::any::type_name::<LintDiagnostic>(),
            core::any::type_name::<NodeInterest>(),
            core::any::type_name::<AstNode>(),
            core::any::type_name::<NodeBatch>(),
            core::any::type_name::<FileContext>(),
            core::any::type_name::<JsxAttribute>(),
            core::any::type_name::<JsxOpeningElementNode>(),
        );
    }
}
