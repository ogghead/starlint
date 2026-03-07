//! Import-related lint rules.
//!
//! Rules are prefixed with `import/` in config and output.

pub mod consistent_type_specifier_style;
pub mod default;
pub mod export;
pub mod exports_last;
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
    vec![]
}
