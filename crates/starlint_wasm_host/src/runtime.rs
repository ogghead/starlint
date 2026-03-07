//! WASM runtime using wasmtime.
//!
//! Manages the wasmtime engine, store, and plugin instances.
//! [`WasmPluginHost`] loads WASM component plugins and produces
//! [`Plugin`](starlint_core::Plugin) instances for integration with
//! the lint engine via [`into_plugins`](WasmPluginHost::into_plugins).
//!
//! Supports two plugin ABIs:
//! - **v1** (`linter-plugin` world): Simplified AST nodes via `node-batch`, no fix support.
//! - **v2** (`linter-plugin-v2` world): Full AST tree (JSON-serialized) + fix support.
//!
//! When loading a plugin, the host tries v2 first and falls back to v1.

// Generated WIT bindings for the `linter-plugin` (v1) world.
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

// Generated WIT bindings for the `linter-plugin-v2` world.
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
pub(crate) mod bindings_v2 {
    wasmtime::component::bindgen!({
        world: "linter-plugin-v2",
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

    use starlint_core::plugin::{FileContext as CoreFileContext, Plugin};
    use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Label, Severity, Span};
    use starlint_plugin_sdk::rule::{Category as SdkCategory, FixKind, RuleMeta as SdkRuleMeta};

    use super::ResourceLimits;
    use super::bindings::LinterPluginPre;
    use super::bindings::starlint::plugin::types as wit;
    use super::bindings_v2::LinterPluginV2Pre;
    use super::bindings_v2::starlint::plugin::types as wit_v2;
    use crate::bridge::{NodeInterest, WitAstNode};
    use crate::builtins;
    use crate::collector::NodeCollector;
    use crate::error::WasmError;

    /// Store host data carrying resource limits for wasmtime.
    struct HostData {
        /// Memory and resource limits enforced by wasmtime.
        limits: StoreLimits,
    }

    /// Pre-instantiated plugin component, either v1 or v2.
    enum PluginPre {
        /// v1 plugin (simplified AST nodes, no fix support).
        V1(LinterPluginPre<HostData>),
        /// v2 plugin (full AST tree + fix support).
        V2(LinterPluginV2Pre<HostData>),
    }

    /// A loaded WASM plugin ready for linting.
    struct LoadedPlugin {
        /// Pre-instantiated component (cheap to instantiate per-file).
        pre: PluginPre,
        /// Cached node interests from `get-node-interests()` (v1 only).
        /// v2 plugins always receive the full tree, so this is unused for them.
        interests: Option<NodeInterest>,
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
    /// Supports both v1 (simplified AST) and v2 (full AST tree) plugins.
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
        /// (rules, node interests) for later use. Tries v2 first, falls back to v1.
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
        ///
        /// Tries to load as a v2 plugin first. If that fails (component doesn't
        /// export the `plugin-v2` interface), falls back to v1.
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

            // Try v2 first, fall back to v1.
            if let Ok(pre_v2) = LinterPluginV2Pre::new(instance_pre.clone()) {
                return self.finish_load_v2(pre_v2, plugin_name, config_json);
            }

            let pre_v1 =
                LinterPluginPre::new(instance_pre).map_err(|err| WasmError::CompileFailed {
                    path: plugin_name.to_owned(),
                    reason: err.to_string(),
                })?;

            self.finish_load_v1(pre_v1, plugin_name, config_json)
        }

        /// Complete loading a v1 plugin: query metadata, validate config, store.
        fn finish_load_v1(
            &mut self,
            pre: LinterPluginPre<HostData>,
            plugin_name: &str,
            config_json: &str,
        ) -> Result<(), WasmError> {
            let (interests, file_patterns, wit_rules, raw_file_patterns) =
                query_plugin_metadata_v1(&pre, &self.engine, plugin_name, &self.limits)?;

            if !config_json.is_empty() {
                validate_config_v1(&pre, &self.engine, plugin_name, config_json, &self.limits)?;
            }

            let cached_rules = wit_rules.into_iter().map(wit_rule_meta_v1_to_sdk).collect();

            self.plugins.push(LoadedPlugin {
                pre: PluginPre::V1(pre),
                interests: Some(interests),
                file_patterns,
                name: plugin_name.to_owned(),
                config_json: config_json.to_owned(),
                cached_rules,
                raw_file_patterns,
            });

            Ok(())
        }

        /// Complete loading a v2 plugin: query metadata, validate config, store.
        fn finish_load_v2(
            &mut self,
            pre: LinterPluginV2Pre<HostData>,
            plugin_name: &str,
            config_json: &str,
        ) -> Result<(), WasmError> {
            let (file_patterns, wit_rules, raw_file_patterns) =
                query_plugin_metadata_v2(&pre, &self.engine, plugin_name, &self.limits)?;

            if !config_json.is_empty() {
                validate_config_v2(&pre, &self.engine, plugin_name, config_json, &self.limits)?;
            }

            let cached_rules = wit_rules.into_iter().map(wit_rule_meta_v2_to_sdk).collect();

            self.plugins.push(LoadedPlugin {
                pre: PluginPre::V2(pre),
                interests: None, // v2 plugins receive the full tree
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
                let result = match &plugin.pre {
                    PluginPre::V1(_) => lint_with_plugin_v1(
                        plugin,
                        &self.engine,
                        &self.limits,
                        file_path,
                        source_text,
                        tree,
                    ),
                    PluginPre::V2(_) => lint_with_plugin_v2(
                        plugin,
                        &self.engine,
                        &self.limits,
                        file_path,
                        source_text,
                        tree,
                    ),
                };

                match result {
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
    /// Each instance wraps one WASM component (v1 or v2) and can lint files
    /// independently.
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
            let result = match &self.plugin.pre {
                PluginPre::V1(_) => lint_with_plugin_v1(
                    &self.plugin,
                    &self.engine,
                    &self.limits,
                    ctx.file_path,
                    ctx.source_text,
                    ctx.tree,
                ),
                PluginPre::V2(_) => lint_with_plugin_v2(
                    &self.plugin,
                    &self.engine,
                    &self.limits,
                    ctx.file_path,
                    ctx.source_text,
                    ctx.tree,
                ),
            };

            match result {
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

    // ---- v1 helpers ----

    /// Query a v1 plugin's rules, node interests, and file patterns using a temporary store.
    ///
    /// Returns `(interests, compiled_glob_set, wit_rules, raw_file_patterns)`.
    #[allow(clippy::type_complexity)]
    fn query_plugin_metadata_v1(
        pre: &LinterPluginPre<HostData>,
        engine: &Engine,
        plugin_name: &str,
        limits: &ResourceLimits,
    ) -> Result<
        (
            NodeInterest,
            Option<GlobSet>,
            Vec<wit::RuleMeta>,
            Vec<String>,
        ),
        WasmError,
    > {
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

        Ok((interests, file_patterns, rules, file_patterns_raw))
    }

    /// Validate v1 plugin config eagerly.
    fn validate_config_v1(
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

    /// Lint a file with a v1 plugin (simplified AST nodes).
    fn lint_with_plugin_v1(
        plugin: &LoadedPlugin,
        engine: &Engine,
        limits: &ResourceLimits,
        file_path: &Path,
        source_text: &str,
        tree: &AstTree,
    ) -> Result<Vec<Diagnostic>, WasmError> {
        let PluginPre::V1(ref pre) = plugin.pre else {
            return Ok(Vec::new());
        };

        // Skip if plugin declares file patterns and this file doesn't match.
        if let Some(ref patterns) = plugin.file_patterns {
            if !patterns.is_match(file_path) {
                return Ok(Vec::new());
            }
        }

        let interests = plugin.interests.unwrap_or_default();

        // Skip if plugin has no relevant interests.
        if !interests.any() {
            return Ok(Vec::new());
        }

        // Collect matching AST nodes from the AstTree.
        let mut collector = NodeCollector::new(interests);
        collector.collect(tree);
        let bridge_nodes = collector.into_nodes();

        // Skip calling the plugin if no matching nodes were found
        // AND the plugin doesn't need source-text access.
        if bridge_nodes.is_empty() && !interests.source_text {
            return Ok(Vec::new());
        }

        // Convert to WIT types and build the batch.
        let wit_batch = build_wit_batch(file_path, source_text, &bridge_nodes);

        // Create a fresh store with fuel + memory limits, configure, then lint.
        let mut store = create_store(engine, limits);
        let instance = pre
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
        Ok(wit_diags
            .into_iter()
            .map(wit_diagnostic_v1_to_sdk)
            .collect())
    }

    // ---- v2 helpers ----

    /// Query a v2 plugin's rules and file patterns using a temporary store.
    ///
    /// v2 plugins don't declare node interests — they receive the full AST tree.
    /// Returns `(compiled_glob_set, wit_rules, raw_file_patterns)`.
    #[allow(clippy::type_complexity)]
    fn query_plugin_metadata_v2(
        pre: &LinterPluginV2Pre<HostData>,
        engine: &Engine,
        plugin_name: &str,
        limits: &ResourceLimits,
    ) -> Result<(Option<GlobSet>, Vec<wit_v2::RuleMeta>, Vec<String>), WasmError> {
        let mut store = create_store(engine, limits);
        let instance = pre
            .instantiate(&mut store)
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin_name.to_owned(),
                reason: err.to_string(),
            })?;

        let guest = instance.starlint_plugin_plugin_v2();

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

    /// Validate v2 plugin config eagerly.
    fn validate_config_v2(
        pre: &LinterPluginV2Pre<HostData>,
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

        let guest = instance.starlint_plugin_plugin_v2();
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

    /// Lint a file with a v2 plugin (full AST tree + fix support).
    fn lint_with_plugin_v2(
        plugin: &LoadedPlugin,
        engine: &Engine,
        limits: &ResourceLimits,
        file_path: &Path,
        source_text: &str,
        tree: &AstTree,
    ) -> Result<Vec<Diagnostic>, WasmError> {
        let PluginPre::V2(ref pre) = plugin.pre else {
            return Ok(Vec::new());
        };

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

        let file_context = wit_v2::FileContext {
            file_path: file_path.display().to_string(),
            source_text: source_text.to_owned(),
            extension,
        };

        // Create a fresh store with fuel + memory limits, configure, then lint.
        let mut store = create_store(engine, limits);
        let instance = pre
            .instantiate(&mut store)
            .map_err(|err| WasmError::RuntimeError {
                plugin_name: plugin.name.clone(),
                reason: err.to_string(),
            })?;

        let guest = instance.starlint_plugin_plugin_v2();

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

        // Convert WIT v2 diagnostics (with fix support) to SDK diagnostics.
        Ok(wit_diags
            .into_iter()
            .map(wit_diagnostic_v2_to_sdk)
            .collect())
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

    // ---- v1 type conversion functions ----

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

    /// Convert a v1 WIT `RuleMeta` to an SDK `RuleMeta`.
    fn wit_rule_meta_v1_to_sdk(meta: wit::RuleMeta) -> SdkRuleMeta {
        SdkRuleMeta {
            name: meta.name,
            description: meta.description,
            category: wit_category_v1_to_sdk(meta.category),
            default_severity: wit_severity_to_sdk(meta.default_severity),
        }
    }

    /// Convert a v1 WIT `Category` to an SDK `Category`.
    fn wit_category_v1_to_sdk(cat: wit::Category) -> SdkCategory {
        match cat {
            wit::Category::Correctness => SdkCategory::Correctness,
            wit::Category::Style => SdkCategory::Style,
            wit::Category::Performance => SdkCategory::Performance,
            wit::Category::Suggestion => SdkCategory::Suggestion,
            wit::Category::Custom => SdkCategory::Custom("custom".to_owned()),
        }
    }

    /// Convert a v1 WIT `LintDiagnostic` to an SDK `Diagnostic` (no fix support).
    fn wit_diagnostic_v1_to_sdk(diag: wit::LintDiagnostic) -> Diagnostic {
        Diagnostic {
            rule_name: diag.rule_name,
            message: diag.message,
            span: Span::new(diag.span.start, diag.span.end),
            severity: wit_severity_to_sdk(diag.severity),
            help: diag.help,
            fix: None,
            labels: vec![],
        }
    }

    // ---- v2 type conversion functions ----

    /// Convert a v2 WIT `RuleMeta` to an SDK `RuleMeta`.
    fn wit_rule_meta_v2_to_sdk(meta: wit_v2::RuleMeta) -> SdkRuleMeta {
        SdkRuleMeta {
            name: meta.name,
            description: meta.description,
            category: wit_category_v2_to_sdk(meta.category),
            default_severity: wit_v2_severity_to_sdk(meta.default_severity),
        }
    }

    /// Convert a v2 WIT `Category` to an SDK `Category`.
    fn wit_category_v2_to_sdk(cat: wit_v2::Category) -> SdkCategory {
        match cat {
            wit_v2::Category::Correctness => SdkCategory::Correctness,
            wit_v2::Category::Style => SdkCategory::Style,
            wit_v2::Category::Performance => SdkCategory::Performance,
            wit_v2::Category::Suggestion => SdkCategory::Suggestion,
            wit_v2::Category::Custom => SdkCategory::Custom("custom".to_owned()),
        }
    }

    /// Convert a v2 WIT `LintDiagnosticV2` to an SDK `Diagnostic` (with fix + labels).
    fn wit_diagnostic_v2_to_sdk(diag: wit_v2::LintDiagnosticV2) -> Diagnostic {
        Diagnostic {
            rule_name: diag.rule_name,
            message: diag.message,
            span: Span::new(diag.span.start, diag.span.end),
            severity: wit_v2_severity_to_sdk(diag.severity),
            help: diag.help,
            fix: diag.fix.map(wit_fix_to_sdk),
            labels: diag.labels.into_iter().map(wit_label_to_sdk).collect(),
        }
    }

    /// Convert a WIT v2 `Fix` to an SDK `Fix`.
    fn wit_fix_to_sdk(fix: wit_v2::Fix) -> Fix {
        Fix {
            kind: match fix.kind {
                wit_v2::FixKind::SafeFix => FixKind::SafeFix,
                wit_v2::FixKind::SuggestionFix => FixKind::SuggestionFix,
                wit_v2::FixKind::DangerousFix => FixKind::DangerousFix,
            },
            message: fix.message,
            edits: fix.edits.into_iter().map(wit_edit_to_sdk).collect(),
            is_snippet: fix.is_snippet,
        }
    }

    /// Convert a WIT v2 `Edit` to an SDK `Edit`.
    fn wit_edit_to_sdk(edit: wit_v2::Edit) -> Edit {
        Edit {
            span: Span::new(edit.span.start, edit.span.end),
            replacement: edit.replacement,
        }
    }

    /// Convert a WIT v2 `Label` to an SDK `Label`.
    fn wit_label_to_sdk(label: wit_v2::Label) -> Label {
        Label {
            span: Span::new(label.span.start, label.span.end),
            message: label.message,
        }
    }

    /// Convert a v1 WIT `Severity` to an SDK `Severity`.
    const fn wit_severity_to_sdk(severity: wit::Severity) -> Severity {
        match severity {
            wit::Severity::Warning => Severity::Warning,
            wit::Severity::Error => Severity::Error,
            wit::Severity::Suggestion => Severity::Suggestion,
        }
    }

    /// Convert a v2 WIT `Severity` to an SDK `Severity`.
    const fn wit_v2_severity_to_sdk(severity: wit_v2::Severity) -> Severity {
        match severity {
            wit_v2::Severity::Warning => Severity::Warning,
            wit_v2::Severity::Error => Severity::Error,
            wit_v2::Severity::Suggestion => Severity::Suggestion,
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
    fn test_v1_bindings_types_exist() {
        // Verify the generated v1 WIT types compile and are accessible.
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

    #[cfg(feature = "wasm")]
    #[test]
    fn test_v2_bindings_types_exist() {
        // Verify the generated v2 WIT types compile and are accessible.
        use bindings_v2::starlint::plugin::types::{
            Edit, FileContext, Fix, FixKind, Label, LintDiagnosticV2, RuleMeta, Severity, Span,
        };
        use bindings_v2::{LinterPluginV2, LinterPluginV2Pre};
        let _ = (
            core::any::type_name::<LinterPluginV2>(),
            core::any::type_name::<LinterPluginV2Pre<()>>(),
            core::any::type_name::<Span>(),
            core::any::type_name::<Severity>(),
            core::any::type_name::<RuleMeta>(),
            core::any::type_name::<LintDiagnosticV2>(),
            core::any::type_name::<FileContext>(),
            core::any::type_name::<Fix>(),
            core::any::type_name::<FixKind>(),
            core::any::type_name::<Edit>(),
            core::any::type_name::<Label>(),
        );
    }
}
