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
    ]
}

/// Names of rules that have been migrated to [`LintRule`].
///
/// These must be excluded from the native rule set to avoid duplicate diagnostics.
pub const MIGRATED_RULE_NAMES: &[&str] = &[
    "eqeqeq",
    "no-continue",
    "no-debugger",
    "no-empty",
    "no-extra-semi",
    "no-null",
    "no-ternary",
    "no-var",
    "no-with",
];
