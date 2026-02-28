//! Import-related lint rules.
//!
//! Rules are prefixed with `import/` in config and output.

pub mod consistent_type_specifier_style;
pub mod default;
pub mod export;
pub mod extensions;
pub mod first;
pub mod group_exports;
pub mod max_dependencies;
pub mod named;
pub mod namespace;
pub mod no_absolute_path;
pub mod no_amd;
pub mod no_anonymous_default_export;
pub mod no_commonjs;
pub mod no_cycle;
pub mod no_default_export;
pub mod no_duplicates;
pub mod no_dynamic_require;
pub mod no_empty_named_blocks;
pub mod no_mutable_exports;
pub mod no_named_as_default;
pub mod no_named_as_default_member;
pub mod no_named_default;
pub mod no_named_export;
pub mod no_namespace;
pub mod no_nodejs_modules;
pub mod no_relative_parent_imports;
pub mod no_restricted_imports;
pub mod no_self_import;
pub mod no_unassigned_import;
pub mod no_webpack_loader_syntax;
pub mod prefer_default_export;
pub mod unambiguous;

use crate::rule::NativeRule;

/// Return all import rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(consistent_type_specifier_style::ConsistentTypeSpecifierStyle),
        Box::new(default::DefaultExport),
        Box::new(export::ExportRule),
        Box::new(extensions::Extensions),
        Box::new(first::First),
        Box::new(group_exports::GroupExports),
        Box::new(max_dependencies::MaxDependencies::new()),
        Box::new(named::NamedExport),
        Box::new(namespace::NamespaceImport),
        Box::new(no_absolute_path::NoAbsolutePath),
        Box::new(no_amd::NoAmd),
        Box::new(no_anonymous_default_export::NoAnonymousDefaultExport),
        Box::new(no_commonjs::NoCommonjs),
        Box::new(no_cycle::NoCycle),
        Box::new(no_default_export::NoDefaultExport),
        Box::new(no_duplicates::NoDuplicates),
        Box::new(no_dynamic_require::NoDynamicRequire),
        Box::new(no_empty_named_blocks::NoEmptyNamedBlocks),
        Box::new(no_mutable_exports::NoMutableExports),
        Box::new(no_named_as_default::NoNamedAsDefault),
        Box::new(no_named_as_default_member::NoNamedAsDefaultMember),
        Box::new(no_named_default::NoNamedDefault),
        Box::new(no_named_export::NoNamedExport),
        Box::new(no_namespace::NoNamespace),
        Box::new(no_nodejs_modules::NoNodejsModules),
        Box::new(no_relative_parent_imports::NoRelativeParentImports),
        Box::new(no_restricted_imports::NoRestrictedImports),
        Box::new(no_self_import::NoSelfImport),
        Box::new(no_unassigned_import::NoUnassignedImport),
        Box::new(no_webpack_loader_syntax::NoWebpackLoaderSyntax),
        Box::new(prefer_default_export::PreferDefaultExport),
        Box::new(unambiguous::Unambiguous),
    ]
}
