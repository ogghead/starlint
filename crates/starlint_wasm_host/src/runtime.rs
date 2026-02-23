//! WASM runtime using wasmtime.
//!
//! Manages the wasmtime engine, store, and plugin instances.
//! [`WasmPluginHost`] loads WASM component plugins and produces
//! [`Plugin`](starlint_rule_framework::Plugin) instances for integration with
//! the lint engine via [`into_plugins`](WasmPluginHost::into_plugins).

// Generated WIT bindings for the `linter-plugin` world.
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
#[derive(Clone, Copy)]
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
    use std::sync::OnceLock;

    use globset::{Glob, GlobSet, GlobSetBuilder};
    use starlint_ast::tree::AstTree;
    use wasmtime::component::{Component, Linker};
    use wasmtime::{Engine, Store, StoreLimits, StoreLimitsBuilder};

    use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Label, Severity, Span};
    use starlint_plugin_sdk::rule::{Category as SdkCategory, FixKind, RuleMeta as SdkRuleMeta};
    use starlint_rule_framework::{FileContext as CoreFileContext, Plugin};

    use super::ResourceLimits;
    use super::bindings::LinterPluginPre;
    use super::bindings::starlint::plugin::types as wit;
    use crate::builtins;
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
        /// Cached file-pattern filter from `get-file-patterns()`.
        /// `None` means the plugin applies to all files.
        file_patterns: Option<GlobSet>,
        /// Plugin name (derived from filename).
        name: String,
        /// Plugin configuration JSON (applied per-file in the same store as linting).
        config_json: String,
        /// Cached rule metadata (queried once at load time).
        cached_rules: Vec<SdkRuleMeta>,
        /// Raw file-pattern strings (for [`Plugin::file_patterns`]).
        raw_file_patterns: Vec<String>,
    }

    /// WASM plugin host powered by wasmtime.
    ///
    /// Loads WASM component plugins and runs them against files in parallel.
    /// Each file gets a fresh [`Store`] with fuel and memory limits for safety.
    pub struct WasmPluginHost {
        /// Wasmtime engine (shared, thread-safe).
        engine: Engine,
        /// Component linker (no imports needed for linter-plugin worlds).
        linker: Linker<HostData>,
        /// Loaded plugins.
        plugins: Vec<LoadedPlugin>,
        /// Resource limits per plugin per file.
        limits: ResourceLimits,
    }

    /// Return a process-wide shared wasmtime `Engine`.
    ///
    /// wasmtime recommends sharing a single `Engine` across the process.
    /// `Engine::clone()` is a cheap `Arc` refcount bump.
    ///
    /// Uses the Winch baseline compiler instead of Cranelift to avoid
    /// a Cranelift JIT bug that causes SIGSEGV under concurrent
    /// compilation (observed with nextest's per-test process model).
    /// Winch compiles faster and produces correct code; the slight
    /// runtime overhead is negligible for lint-rule workloads.
    fn shared_engine() -> Result<Engine, WasmError> {
        static ENGINE: OnceLock<Result<Engine, String>> = OnceLock::new();
        let result = ENGINE.get_or_init(|| {
            let mut config = wasmtime::Config::new();
            config.consume_fuel(true);
            config.wasm_component_model(true);
            config.strategy(wasmtime::Strategy::Winch);
            Engine::new(&config).map_err(|err| err.to_string())
        });
        match result {
            Ok(engine) => Ok(engine.clone()),
            Err(reason) => Err(WasmError::CompileFailed {
                path: "<engine>".to_owned(),
                reason: reason.clone(),
            }),
        }
    }

    impl WasmPluginHost {
        /// Create a new WASM plugin host with the given resource limits.
        pub fn new(limits: ResourceLimits) -> Result<Self, WasmError> {
            let engine = shared_engine()?;
            let linker = Linker::new(&engine);

            Ok(Self {
                engine,
                linker,
                plugins: Vec::new(),
                limits,
            })
        }

        /// Load a WASM component plugin from disk.
        ///
        /// Compiles the component, pre-instantiates it, and caches metadata
        /// (rules, file patterns) for later use.
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

            self.finish_load(pre, plugin_name, config_json)
        }

        /// Complete loading a plugin: query metadata, validate config, store.
        fn finish_load(
            &mut self,
            pre: LinterPluginPre<HostData>,
            plugin_name: &str,
            config_json: &str,
        ) -> Result<(), WasmError> {
            let (file_patterns, wit_rules, raw_file_patterns) =
                query_plugin_metadata(&pre, &self.engine, plugin_name, &self.limits)?;

            if !config_json.is_empty() {
                validate_config(&pre, &self.engine, plugin_name, config_json, &self.limits)?;
            }

            let cached_rules = wit_rules.into_iter().map(wit_rule_meta_to_sdk).collect();

            self.plugins.push(LoadedPlugin {
                pre,
                file_patterns,
                name: plugin_name.to_owned(),
                config_json: config_json.to_owned(),
                cached_rules,
                raw_file_patterns,
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
            tree: &AstTree,
        ) -> Vec<Diagnostic> {
            let mut all_diagnostics = Vec::new();

            for plugin in &self.plugins {
                match lint_with_plugin(
                    plugin,
                    &self.engine,
                    &self.limits,
                    file_path,
                    source_text,
                    tree,
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

        /// Lint a single file with all loaded plugins.
        ///
        /// Convenience method that delegates to the internal linting pipeline.
        /// Prefer [`into_plugins`](Self::into_plugins) for integration with
        /// the unified [`Plugin`] system.
        pub fn lint_file(
            &self,
            file_path: &Path,
            source_text: &str,
            tree: &AstTree,
        ) -> Vec<Diagnostic> {
            self.lint_file_internal(file_path, source_text, tree)
        }

        /// Consume the host, producing individual [`Plugin`] instances.
        ///
        /// Each loaded WASM plugin becomes a separate `Box<dyn Plugin>` that
        /// can be added to the engine's plugin set alongside native rules.
        pub fn into_plugins(self) -> Vec<Box<dyn Plugin>> {
            let engine = self.engine;
            let limits = self.limits;
            self.plugins
                .into_iter()
                .map(|plugin| -> Box<dyn Plugin> {
                    Box::new(WasmPlugin {
                        plugin,
                        engine: engine.clone(),
                        limits,
                    })
                })
                .collect()
        }
    }

    /// A single WASM plugin implementing the unified [`Plugin`] trait.
    ///
    /// Created by [`WasmPluginHost::into_plugins`] after loading is complete.
    /// Each instance wraps one WASM component and can lint files independently.
    pub struct WasmPlugin {
        /// The loaded plugin state (pre-instantiated component + cached metadata).
        plugin: LoadedPlugin,
        /// Wasmtime engine (shared, cheap clone via Arc).
        engine: Engine,
        /// Resource limits for per-file stores.
        limits: ResourceLimits,
    }

    impl Plugin for WasmPlugin {
        fn rules(&self) -> Vec<SdkRuleMeta> {
            self.plugin.cached_rules.clone()
        }

        fn lint_file(&self, ctx: &CoreFileContext<'_>) -> Vec<Diagnostic> {
            match lint_with_plugin(
                &self.plugin,
                &self.engine,
                &self.limits,
                ctx.file_path,
                ctx.source_text,
                ctx.tree,
            ) {
                Ok(diags) => diags,
                Err(err) => {
                    tracing::warn!(
                        "plugin '{}' failed on {}: {err}",
                        self.plugin.name,
                        ctx.file_path.display()
                    );
                    Vec::new()
                }
            }
        }

        fn file_patterns(&self) -> Vec<String> {
            self.plugin.raw_file_patterns.clone()
        }

        fn needs_scope_analysis(&self) -> bool {
            false
        }

        fn configure(&mut self, config: &str) -> Vec<String> {
            // WASM plugins are configured during loading via WasmPluginHost.
            if config.is_empty() {
                return vec![];
            }
            vec!["WASM plugins must be configured at load time".to_owned()]
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

    /// Query a plugin's rules and file patterns using a temporary store.
    ///
    /// Returns `(compiled_glob_set, wit_rules, raw_file_patterns)`.
    #[allow(clippy::type_complexity)]
    fn query_plugin_metadata(
        pre: &LinterPluginPre<HostData>,
        engine: &Engine,
        plugin_name: &str,
        limits: &ResourceLimits,
    ) -> Result<(Option<GlobSet>, Vec<wit::RuleMeta>, Vec<String>), WasmError> {
        let mut store = create_store(engine, limits);
        let instance = pre
            .instantiate(&mut store)
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin_name.to_owned(),
                reason: err.to_string(),
            })?;

        let guest = instance.starlint_plugin_plugin();

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

        let file_patterns = compile_file_patterns(&file_patterns_raw, plugin_name);

        Ok((file_patterns, rules, file_patterns_raw))
    }

    /// Validate plugin config eagerly.
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

    /// Lint a file with a plugin (full AST tree + fix support).
    fn lint_with_plugin(
        plugin: &LoadedPlugin,
        engine: &Engine,
        limits: &ResourceLimits,
        file_path: &Path,
        source_text: &str,
        tree: &AstTree,
    ) -> Result<Vec<Diagnostic>, WasmError> {
        // Skip if plugin declares file patterns and this file doesn't match.
        if let Some(ref patterns) = plugin.file_patterns {
            if !patterns.is_match(file_path) {
                return Ok(Vec::new());
            }
        }

        // Serialize the AstTree directly to JSON bytes.
        let tree_bytes = serde_json::to_vec(tree).map_err(|err| WasmError::RuntimeError {
            plugin_name: plugin.name.clone(),
            reason: format!("failed to serialize AST tree: {err}"),
        })?;

        // Build file context.
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

        // Create a fresh store with fuel + memory limits, configure, then lint.
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
            .call_lint_file(&mut store, &file_context, &tree_bytes)
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin.name.clone(),
                reason: err.to_string(),
            })?;

        // Convert WIT diagnostics to SDK diagnostics.
        Ok(wit_diags.into_iter().map(wit_diagnostic_to_sdk).collect())
    }

    // ---- Shared helpers ----

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

    /// Convert a WIT `RuleMeta` to an SDK `RuleMeta`.
    fn wit_rule_meta_to_sdk(meta: wit::RuleMeta) -> SdkRuleMeta {
        SdkRuleMeta {
            name: meta.name,
            description: meta.description,
            category: wit_category_to_sdk(meta.category),
            default_severity: wit_severity_to_sdk(meta.default_severity),
        }
    }

    /// Convert a WIT `Category` to an SDK `Category`.
    fn wit_category_to_sdk(cat: wit::Category) -> SdkCategory {
        match cat {
            wit::Category::Correctness => SdkCategory::Correctness,
            wit::Category::Style => SdkCategory::Style,
            wit::Category::Performance => SdkCategory::Performance,
            wit::Category::Suggestion => SdkCategory::Suggestion,
            wit::Category::Custom => SdkCategory::Custom("custom".to_owned()),
        }
    }

    /// Convert a WIT `LintDiagnostic` to an SDK `Diagnostic` (with fix + labels).
    fn wit_diagnostic_to_sdk(diag: wit::LintDiagnostic) -> Diagnostic {
        Diagnostic {
            rule_name: diag.rule_name,
            message: diag.message,
            span: Span::new(diag.span.start, diag.span.end),
            severity: wit_severity_to_sdk(diag.severity),
            help: diag.help,
            fix: diag.fix.map(wit_fix_to_sdk),
            labels: diag.labels.into_iter().map(wit_label_to_sdk).collect(),
        }
    }

    /// Convert a WIT `Fix` to an SDK `Fix`.
    fn wit_fix_to_sdk(fix: wit::Fix) -> Fix {
        Fix {
            kind: match fix.kind {
                wit::FixKind::SafeFix => FixKind::SafeFix,
                wit::FixKind::SuggestionFix => FixKind::SuggestionFix,
                wit::FixKind::DangerousFix => FixKind::DangerousFix,
            },
            message: fix.message,
            edits: fix.edits.into_iter().map(wit_edit_to_sdk).collect(),
            is_snippet: fix.is_snippet,
        }
    }

    /// Convert a WIT `Edit` to an SDK `Edit`.
    fn wit_edit_to_sdk(edit: wit::Edit) -> Edit {
        Edit {
            span: Span::new(edit.span.start, edit.span.end),
            replacement: edit.replacement,
        }
    }

    /// Convert a WIT `Label` to an SDK `Label`.
    fn wit_label_to_sdk(label: wit::Label) -> Label {
        Label {
            span: Span::new(label.span.start, label.span.end),
            message: label.message,
        }
    }

    /// Convert a WIT `Severity` to an SDK `Severity`.
    const fn wit_severity_to_sdk(severity: wit::Severity) -> Severity {
        match severity {
            wit::Severity::Warning => Severity::Warning,
            wit::Severity::Error => Severity::Error,
            wit::Severity::Suggestion => Severity::Suggestion,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_plugin_name_from_path() {
            assert_eq!(
                plugin_name_from_path(Path::new("/plugins/my-plugin.wasm")),
                "my-plugin"
            );
            assert_eq!(plugin_name_from_path(Path::new("simple.wasm")), "simple");
            assert_eq!(
                plugin_name_from_path(Path::new("/no-extension")),
                "no-extension"
            );
        }

        #[test]
        fn test_compile_file_patterns_empty() {
            let result = compile_file_patterns(&[], "test-plugin");
            assert!(result.is_none(), "empty patterns should return None");
        }

        #[test]
        fn test_compile_file_patterns_valid() {
            let patterns = vec!["*.stories.tsx".to_owned(), "*.stories.ts".to_owned()];
            let result = compile_file_patterns(&patterns, "test-plugin");
            assert!(result.is_some(), "valid patterns should compile");
            if let Some(set) = &result {
                assert!(
                    set.is_match(Path::new("Button.stories.tsx")),
                    "should match .stories.tsx"
                );
                assert!(
                    !set.is_match(Path::new("Button.tsx")),
                    "should not match plain .tsx"
                );
            }
        }

        #[test]
        fn test_compile_file_patterns_invalid() {
            let patterns = vec!["[invalid".to_owned()];
            let result = compile_file_patterns(&patterns, "test-plugin");
            assert!(result.is_none(), "invalid patterns should return None");
        }

        #[test]
        fn test_wit_severity_to_sdk_conversions() {
            assert!(matches!(
                wit_severity_to_sdk(wit::Severity::Error),
                Severity::Error
            ));
            assert!(matches!(
                wit_severity_to_sdk(wit::Severity::Warning),
                Severity::Warning
            ));
            assert!(matches!(
                wit_severity_to_sdk(wit::Severity::Suggestion),
                Severity::Suggestion
            ));
        }

        #[test]
        fn test_wit_category_to_sdk_conversions() {
            assert!(matches!(
                wit_category_to_sdk(wit::Category::Correctness),
                SdkCategory::Correctness
            ));
            assert!(matches!(
                wit_category_to_sdk(wit::Category::Style),
                SdkCategory::Style
            ));
            assert!(matches!(
                wit_category_to_sdk(wit::Category::Performance),
                SdkCategory::Performance
            ));
            assert!(matches!(
                wit_category_to_sdk(wit::Category::Suggestion),
                SdkCategory::Suggestion
            ));
            assert!(matches!(
                wit_category_to_sdk(wit::Category::Custom),
                SdkCategory::Custom(_)
            ));
        }

        #[test]
        fn test_wit_rule_meta_to_sdk() {
            let meta = wit::RuleMeta {
                name: "test/rule".to_owned(),
                description: "A test rule".to_owned(),
                category: wit::Category::Correctness,
                default_severity: wit::Severity::Error,
            };
            let sdk_meta = wit_rule_meta_to_sdk(meta);
            assert_eq!(sdk_meta.name, "test/rule");
            assert_eq!(sdk_meta.description, "A test rule");
        }

        #[test]
        fn test_wit_edit_to_sdk() {
            let edit = wit::Edit {
                span: wit::Span { start: 0, end: 5 },
                replacement: "const".to_owned(),
            };
            let sdk_edit = wit_edit_to_sdk(edit);
            assert_eq!(sdk_edit.span.start, 0);
            assert_eq!(sdk_edit.span.end, 5);
            assert_eq!(sdk_edit.replacement, "const");
        }

        #[test]
        fn test_wit_label_to_sdk() {
            let label = wit::Label {
                span: wit::Span { start: 10, end: 20 },
                message: "here".to_owned(),
            };
            let sdk_label = wit_label_to_sdk(label);
            assert_eq!(sdk_label.span.start, 10);
            assert_eq!(sdk_label.span.end, 20);
            assert_eq!(sdk_label.message, "here");
        }

        #[test]
        fn test_wit_fix_to_sdk() {
            use starlint_plugin_sdk::rule::FixKind;

            let fix = wit::Fix {
                kind: wit::FixKind::SafeFix,
                message: "remove debugger".to_owned(),
                edits: vec![wit::Edit {
                    span: wit::Span { start: 0, end: 9 },
                    replacement: String::new(),
                }],
                is_snippet: false,
            };
            let sdk_fix = wit_fix_to_sdk(fix);
            assert!(matches!(sdk_fix.kind, FixKind::SafeFix));
            assert_eq!(sdk_fix.message, "remove debugger");
            assert_eq!(sdk_fix.edits.len(), 1);
            assert!(!sdk_fix.is_snippet);
        }

        #[test]
        fn test_wit_fix_to_sdk_suggestion() {
            use starlint_plugin_sdk::rule::FixKind;

            let fix = wit::Fix {
                kind: wit::FixKind::SuggestionFix,
                message: "suggestion".to_owned(),
                edits: vec![],
                is_snippet: true,
            };
            let sdk_fix = wit_fix_to_sdk(fix);
            assert!(matches!(sdk_fix.kind, FixKind::SuggestionFix));
            assert!(sdk_fix.is_snippet);
        }

        #[test]
        fn test_wit_fix_to_sdk_dangerous() {
            use starlint_plugin_sdk::rule::FixKind;

            let fix = wit::Fix {
                kind: wit::FixKind::DangerousFix,
                message: "dangerous".to_owned(),
                edits: vec![],
                is_snippet: false,
            };
            let sdk_fix = wit_fix_to_sdk(fix);
            assert!(matches!(sdk_fix.kind, FixKind::DangerousFix));
        }

        #[test]
        fn test_wit_diagnostic_to_sdk_full() {
            let diag = wit::LintDiagnostic {
                rule_name: "test/no-debugger".to_owned(),
                message: "Unexpected debugger statement".to_owned(),
                span: wit::Span { start: 0, end: 9 },
                severity: wit::Severity::Error,
                help: Some("Remove the debugger statement".to_owned()),
                fix: Some(wit::Fix {
                    kind: wit::FixKind::SafeFix,
                    message: "Remove debugger".to_owned(),
                    edits: vec![wit::Edit {
                        span: wit::Span { start: 0, end: 9 },
                        replacement: String::new(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![wit::Label {
                    span: wit::Span { start: 0, end: 8 },
                    message: "here".to_owned(),
                }],
            };
            let sdk_diag = wit_diagnostic_to_sdk(diag);
            assert_eq!(sdk_diag.rule_name, "test/no-debugger");
            assert_eq!(sdk_diag.span.start, 0);
            assert_eq!(sdk_diag.span.end, 9);
            assert!(sdk_diag.help.is_some());
            assert!(sdk_diag.fix.is_some());
            assert_eq!(sdk_diag.labels.len(), 1);
        }

        #[test]
        fn test_wit_diagnostic_to_sdk_minimal() {
            let diag = wit::LintDiagnostic {
                rule_name: "test/rule".to_owned(),
                message: "msg".to_owned(),
                span: wit::Span { start: 0, end: 1 },
                severity: wit::Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            };
            let sdk_diag = wit_diagnostic_to_sdk(diag);
            assert_eq!(sdk_diag.rule_name, "test/rule");
            assert!(sdk_diag.help.is_none());
            assert!(sdk_diag.fix.is_none());
            assert!(sdk_diag.labels.is_empty());
        }

        #[test]
        fn test_wasm_host_into_empty_plugins() {
            let host = WasmPluginHost::new(ResourceLimits::default());
            assert!(host.is_ok(), "should create host");
            if let Ok(h) = host {
                let plugins = h.into_plugins();
                assert!(
                    plugins.is_empty(),
                    "host with no loaded plugins should produce empty vec"
                );
            }
        }

        #[test]
        fn test_wasm_host_plugin_count() {
            let host = WasmPluginHost::new(ResourceLimits::default());
            assert!(host.is_ok(), "should create host");
            if let Ok(h) = host {
                assert_eq!(h.plugin_count(), 0, "new host should have 0 plugins");
            }
        }

        #[test]
        fn test_wasm_host_lint_file_no_plugins() {
            let host = WasmPluginHost::new(ResourceLimits::default());
            assert!(host.is_ok(), "should create host");
            if let Ok(h) = host {
                let tree = AstTree::new();
                let diags = h.lint_file(Path::new("test.js"), "const x = 1;", &tree);
                assert!(
                    diags.is_empty(),
                    "host with no plugins should produce no diagnostics"
                );
            }
        }
    }
}

#[cfg(feature = "wasm")]
pub use host::{WasmPlugin, WasmPluginHost};

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
            Edit, FileContext, Fix, FixKind, Label, LintDiagnostic, RuleMeta, Severity, Span,
        };
        use bindings::{LinterPlugin, LinterPluginPre};
        let _ = (
            core::any::type_name::<LinterPlugin>(),
            core::any::type_name::<LinterPluginPre<()>>(),
            core::any::type_name::<Span>(),
            core::any::type_name::<Severity>(),
            core::any::type_name::<RuleMeta>(),
            core::any::type_name::<LintDiagnostic>(),
            core::any::type_name::<FileContext>(),
            core::any::type_name::<Fix>(),
            core::any::type_name::<FixKind>(),
            core::any::type_name::<Edit>(),
            core::any::type_name::<Label>(),
        );
    }
}
