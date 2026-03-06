//! Lint rules using the unified [`LintRule`] trait.
//!
//! These rules use [`AstTree`] instead of oxc's `AstKind`. As rules are
//! migrated from `NativeRule` to `LintRule`, they move here.

pub mod eqeqeq;
pub mod no_debugger;
pub mod no_empty;
pub mod no_var;

use crate::lint_rule::LintRule;

/// Return all unified [`LintRule`] implementations.
#[must_use]
pub fn all_lint_rules() -> Vec<Box<dyn LintRule>> {
    use crate::rules;

    vec![
        // POC rules (Phase 2)
        Box::new(eqeqeq::Eqeqeq),
        Box::new(no_debugger::NoDebugger),
        Box::new(no_empty::NoEmpty),
        Box::new(no_var::NoVar),
        // Migrated rules (Phase 4)
        Box::new(rules::no_continue::NoContinue),
        Box::new(rules::no_with::NoWith),
        Box::new(rules::no_ternary::NoTernary),
        Box::new(rules::no_null::NoNull),
        Box::new(rules::no_extra_semi::NoExtraSemi),
        // Batch 2
        Box::new(rules::no_const_enum::NoConstEnum),
        Box::new(rules::no_empty_static_block::NoEmptyStaticBlock),
        Box::new(rules::no_iterator::NoIterator),
        Box::new(rules::no_labels::NoLabels),
        Box::new(rules::no_multi_assign::NoMultiAssign),
        Box::new(rules::no_script_url::NoScriptUrl),
        Box::new(rules::no_void::NoVoid),
        // Batch 3
        Box::new(rules::no_caller::NoCaller),
        Box::new(rules::no_delete_var::NoDeleteVar),
        Box::new(rules::no_empty_function::NoEmptyFunction),
        Box::new(rules::no_nested_ternary::NoNestedTernary),
        Box::new(rules::no_new::NoNew),
        Box::new(rules::no_proto::NoProto),
        Box::new(rules::prefer_rest_params::PreferRestParams),
    ]
}

/// Names of rules that have been migrated to [`LintRule`].
///
/// These must be excluded from the native rule set to avoid duplicate diagnostics.
pub const MIGRATED_RULE_NAMES: &[&str] = &[
    "eqeqeq",
    "no-caller",
    "no-const-enum",
    "no-continue",
    "no-debugger",
    "no-delete-var",
    "no-empty",
    "no-empty-function",
    "no-empty-static-block",
    "no-extra-semi",
    "no-iterator",
    "no-labels",
    "no-multi-assign",
    "no-nested-ternary",
    "no-new",
    "no-null",
    "no-proto",
    "no-script-url",
    "no-ternary",
    "no-var",
    "no-void",
    "no-with",
    "prefer-rest-params",
];
