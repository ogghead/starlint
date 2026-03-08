//! Modules lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! modules rules.

pub mod rules;

use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

/// Create the modules plugin with all its rules.
#[must_use]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(LintRulePlugin::new(all_rules()))
}

/// Return all modules lint rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(
            crate::rules::import::consistent_type_specifier_style::ConsistentTypeSpecifierStyle,
        ),
        Box::new(crate::rules::import::default::DefaultExport),
        Box::new(crate::rules::import::export::ExportRule),
        Box::new(crate::rules::import::exports_last::ExportsLast),
        Box::new(crate::rules::import::extensions::Extensions),
        Box::new(crate::rules::import::first::First),
        Box::new(crate::rules::import::group_exports::GroupExports),
        Box::new(crate::rules::import::max_dependencies::MaxDependencies::new()),
        Box::new(crate::rules::import::named::NamedExport),
        Box::new(crate::rules::import::namespace::NamespaceImport),
        Box::new(crate::rules::import::no_absolute_path::NoAbsolutePath),
        Box::new(crate::rules::import::no_amd::NoAmd),
        Box::new(crate::rules::import::no_anonymous_default_export::NoAnonymousDefaultExport),
        Box::new(crate::rules::import::no_commonjs::NoCommonjs),
        Box::new(crate::rules::import::no_cycle::NoCycle),
        Box::new(crate::rules::import::no_default_export::NoDefaultExport),
        Box::new(crate::rules::import::no_duplicates::NoDuplicates),
        Box::new(crate::rules::import::no_dynamic_require::NoDynamicRequire),
        Box::new(crate::rules::import::no_empty_named_blocks::NoEmptyNamedBlocks),
        Box::new(crate::rules::import::no_mutable_exports::NoMutableExports),
        Box::new(crate::rules::import::no_named_as_default::NoNamedAsDefault),
        Box::new(crate::rules::import::no_named_as_default_member::NoNamedAsDefaultMember),
        Box::new(crate::rules::import::no_named_default::NoNamedDefault),
        Box::new(crate::rules::import::no_named_export::NoNamedExport),
        Box::new(crate::rules::import::no_namespace::NoNamespace),
        Box::new(crate::rules::import::no_nodejs_modules::NoNodejsModules),
        Box::new(crate::rules::import::no_relative_parent_imports::NoRelativeParentImports),
        Box::new(crate::rules::import::no_restricted_imports::NoRestrictedImports),
        Box::new(crate::rules::import::no_self_import::NoSelfImport),
        Box::new(crate::rules::import::no_unassigned_import::NoUnassignedImport),
        Box::new(crate::rules::import::no_webpack_loader_syntax::NoWebpackLoaderSyntax),
        Box::new(crate::rules::import::prefer_default_export::PreferDefaultExport),
        Box::new(crate::rules::import::unambiguous::Unambiguous),
        Box::new(crate::rules::node::global_require::GlobalRequire::new()),
        Box::new(crate::rules::node::no_exports_assign::NoExportsAssign),
        Box::new(crate::rules::node::no_new_require::NoNewRequire),
        Box::new(crate::rules::node::no_path_concat::NoPathConcat),
        Box::new(crate::rules::node::no_process_env::NoProcessEnv),
        Box::new(crate::rules::node::no_process_exit::NoProcessExit),
        Box::new(crate::rules::promise::always_return::AlwaysReturn),
        Box::new(crate::rules::promise::avoid_new::AvoidNew),
        Box::new(crate::rules::promise::catch_or_return::CatchOrReturn),
        Box::new(crate::rules::promise::no_callback_in_promise::NoCallbackInPromise),
        Box::new(crate::rules::promise::no_multiple_resolved::NoMultipleResolved),
        Box::new(crate::rules::promise::no_native::NoNative),
        Box::new(crate::rules::promise::no_nesting::NoNesting),
        Box::new(crate::rules::promise::no_new_statics::NoNewStatics),
        Box::new(crate::rules::promise::no_promise_in_callback::NoPromiseInCallback),
        Box::new(crate::rules::promise::no_return_in_finally::NoReturnInFinally),
        Box::new(crate::rules::promise::no_return_wrap::NoReturnWrap),
        Box::new(crate::rules::promise::param_names::ParamNames),
        Box::new(crate::rules::promise::prefer_await_to_callbacks::PreferAwaitToCallbacks),
        Box::new(crate::rules::promise::prefer_await_to_then::PreferAwaitToThen),
        Box::new(crate::rules::promise::spec_only::SpecOnly),
        Box::new(crate::rules::promise::valid_params::ValidParams),
    ]
}
