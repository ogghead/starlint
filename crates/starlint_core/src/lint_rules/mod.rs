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
        // Batch 4
        Box::new(rules::approx_constant::ApproxConstant),
        Box::new(rules::bad_comparison_sequence::BadComparisonSequence),
        Box::new(rules::double_comparisons::DoubleComparisons),
        Box::new(rules::erasing_op::ErasingOp),
        Box::new(rules::id_length::IdLength::new()),
        Box::new(rules::use_isnan::UseIsnan),
        // Batch 5
        Box::new(rules::bad_bitwise_operator::BadBitwiseOperator),
        Box::new(rules::empty_brace_spaces::EmptyBraceSpaces),
        Box::new(rules::no_empty_character_class::NoEmptyCharacterClass),
        Box::new(rules::no_extra_boolean_cast::NoExtraBooleanCast),
        Box::new(rules::no_multi_str::NoMultiStr),
        Box::new(rules::no_new_wrappers::NoNewWrappers),
        Box::new(rules::no_useless_concat::NoUselessConcat),
        Box::new(rules::no_useless_escape::NoUselessEscape),
        // Batch 6
        Box::new(rules::bad_char_at_comparison::BadCharAtComparison),
        Box::new(rules::bad_replace_all_arg::BadReplaceAllArg),
        Box::new(rules::consistent_empty_array_spread::ConsistentEmptyArraySpread),
        Box::new(rules::error_message::ErrorMessage),
        Box::new(rules::escape_case::EscapeCase),
        Box::new(rules::func_names::FuncNames),
        Box::new(rules::func_style::FuncStyle),
        Box::new(rules::valid_typeof::ValidTypeof),
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
    "use-isnan",
    // Batch 4
    "approx-constant",
    "bad-comparison-sequence",
    "double-comparisons",
    "erasing-op",
    "id-length",
    // Batch 5
    "bad-bitwise-operator",
    "empty-brace-spaces",
    "no-empty-character-class",
    "no-extra-boolean-cast",
    "no-multi-str",
    "no-new-wrappers",
    "no-useless-concat",
    "no-useless-escape",
    // Batch 6
    "bad-char-at-comparison",
    "bad-replace-all-arg",
    "consistent-empty-array-spread",
    "error-message",
    "escape-case",
    "func-names",
    "func-style",
    "valid-typeof",
];
