//! Built-in native lint rules and rule registry.
//!
//! Rules are organized into named plugin bundles via [`native_plugin_registry`].
//! The unified plugin loader in `starlint_loader` handles
//! config-based filtering and severity overrides.

pub mod accessor_pairs;
pub mod approx_constant;
pub mod array_callback_return;
pub mod arrow_body_style;
pub mod bad_array_method_on_arguments;
pub mod bad_bitwise_operator;
pub mod bad_char_at_comparison;
pub mod bad_comparison_sequence;
pub mod bad_min_max_func;
pub mod bad_object_literal_comparison;
pub mod bad_replace_all_arg;
pub mod better_regex;
pub mod block_scoped_var;
pub mod branches_sharing_code;
pub mod capitalized_comments;
pub mod catch_error_name;
pub mod class_methods_use_this;
pub mod consistent_assert;
pub mod consistent_date_clone;
pub mod consistent_destructuring;
pub mod consistent_empty_array_spread;
pub mod consistent_existence_index_check;
pub mod consistent_function_scoping;
pub mod consistent_template_literal_escape;
pub mod const_comparisons;
pub mod constructor_super;
pub mod curly;
pub mod default_case;
pub mod default_case_last;
pub mod default_param_last;
pub mod double_comparisons;
pub mod empty_brace_spaces;
pub mod eqeqeq;
pub mod erasing_op;
pub mod error_message;
pub mod escape_case;
pub mod expiring_todo_comments;
pub mod explicit_length_check;
pub mod filename_case;
pub mod for_direction;
pub mod func_names;
pub mod func_style;
pub mod getter_return;
pub mod grouped_accessor_pairs;
pub mod guard_for_in;
pub mod id_length;
pub mod init_declarations;
pub mod max_classes_per_file;
pub mod max_complexity;
pub mod max_depth;
pub mod max_lines;
pub mod max_lines_per_function;
pub mod max_nested_callbacks;
pub mod max_params;
pub mod max_statements;
pub mod misrefactored_assign_op;
pub mod missing_throw;
pub mod new_cap;
pub mod new_for_builtins;
pub mod no_abusive_eslint_disable;
pub mod no_accessor_recursion;
pub mod no_accumulating_spread;
pub mod no_alert;
pub mod no_anonymous_default_export;
pub mod no_array_callback_reference;
pub mod no_array_constructor;
pub mod no_array_for_each;
pub mod no_array_method_this_argument;
pub mod no_array_push_push;
pub mod no_array_reduce;
pub mod no_array_reverse;
pub mod no_array_sort;
pub mod no_async_await;
pub mod no_async_endpoint_handlers;
pub mod no_async_promise_executor;
pub mod no_await_expression_member;
pub mod no_await_in_loop;
pub mod no_await_in_promise_methods;
pub mod no_barrel_file;
pub mod no_bitwise;
pub mod no_caller;
pub mod no_case_declarations;
pub mod no_class_assign;
pub mod no_compare_neg_zero;
pub mod no_cond_assign;
pub mod no_console;
pub mod no_console_spaces;
pub mod no_const_assign;
pub mod no_const_enum;
pub mod no_constant_binary_expression;
pub mod no_constant_condition;
pub mod no_constructor_return;
pub mod no_continue;
pub mod no_control_regex;
pub mod no_debugger;
pub mod no_delete_var;
pub mod no_div_regex;
pub mod no_document_cookie;
pub mod no_dupe_class_members;
pub mod no_dupe_else_if;
pub mod no_dupe_keys;
pub mod no_duplicate_case;
pub mod no_duplicate_imports;
pub mod no_else_return;
pub mod no_empty;
pub mod no_empty_character_class;
pub mod no_empty_file;
pub mod no_empty_function;
pub mod no_empty_pattern;
pub mod no_empty_static_block;
pub mod no_eq_null;
pub mod no_eval;
pub mod no_ex_assign;
pub mod no_extend_native;
pub mod no_extra_bind;
pub mod no_extra_boolean_cast;
pub mod no_extra_label;
pub mod no_extra_semi;
pub mod no_fallthrough;
pub mod no_func_assign;
pub mod no_global_assign;
pub mod no_hex_escape;
pub mod no_immediate_mutation;
pub mod no_implicit_coercion;
pub mod no_import_assign;
pub mod no_inline_comments;
pub mod no_inner_declarations;
pub mod no_instanceof_array;
pub mod no_instanceof_builtins;
pub mod no_invalid_fetch_options;
pub mod no_invalid_regexp;
pub mod no_invalid_remove_event_listener;
pub mod no_irregular_whitespace;
pub mod no_iterator;
pub mod no_keyword_prefix;
pub mod no_label_var;
pub mod no_labels;
pub mod no_length_as_slice_end;
pub mod no_lone_blocks;
pub mod no_lonely_if;
pub mod no_loop_func;
pub mod no_loss_of_precision;
pub mod no_magic_array_flat_depth;
pub mod no_magic_numbers;
pub mod no_map_spread;
pub mod no_misleading_character_class;
pub mod no_multi_assign;
pub mod no_multi_str;
pub mod no_negated_condition;
pub mod no_negation_in_equality_check;
pub mod no_nested_ternary;
pub mod no_new;
pub mod no_new_array;
pub mod no_new_buffer;
pub mod no_new_func;
pub mod no_new_native_nonconstructor;
pub mod no_new_wrappers;
pub mod no_nonoctal_decimal_escape;
pub mod no_null;
pub mod no_obj_calls;
pub mod no_object_as_default_parameter;
pub mod no_object_constructor;
pub mod no_optional_chaining;
pub mod no_param_reassign;
pub mod no_plusplus;
pub mod no_process_exit;
pub mod no_promise_executor_return;
pub mod no_proto;
pub mod no_prototype_builtins;
pub mod no_redeclare;
pub mod no_regex_spaces;
pub mod no_rest_spread_properties;
pub mod no_restricted_globals;
pub mod no_restricted_imports;
pub mod no_return_assign;
pub mod no_script_url;
pub mod no_self_assign;
pub mod no_self_compare;
pub mod no_sequences;
pub mod no_setter_return;
pub mod no_shadow;
pub mod no_shadow_restricted_names;
pub mod no_single_promise_in_promise_methods;
pub mod no_sparse_arrays;
pub mod no_static_only_class;
pub mod no_template_curly_in_string;
pub mod no_ternary;
pub mod no_thenable;
pub mod no_this_assignment;
pub mod no_this_before_super;
pub mod no_this_in_exported_function;
pub mod no_throw_literal;
pub mod no_typeof_undefined;
pub mod no_unassigned_vars;
pub mod no_undef;
pub mod no_undefined;
pub mod no_unexpected_multiline;
pub mod no_unmodified_loop_condition;
pub mod no_unnecessary_array_flat_depth;
pub mod no_unnecessary_array_splice_count;
pub mod no_unnecessary_await;
pub mod no_unnecessary_slice_end;
pub mod no_unneeded_ternary;
pub mod no_unreachable;
pub mod no_unreadable_array_destructuring;
pub mod no_unreadable_iife;
pub mod no_unsafe_finally;
pub mod no_unsafe_negation;
pub mod no_unsafe_optional_chaining;
pub mod no_unused_expressions;
pub mod no_unused_labels;
pub mod no_unused_private_class_members;
pub mod no_unused_vars;
pub mod no_use_before_define;
pub mod no_useless_backreference;
pub mod no_useless_call;
pub mod no_useless_catch;
pub mod no_useless_collection_argument;
pub mod no_useless_computed_key;
pub mod no_useless_concat;
pub mod no_useless_constructor;
pub mod no_useless_error_capture_stack_trace;
pub mod no_useless_escape;
pub mod no_useless_fallback_in_spread;
pub mod no_useless_length_check;
pub mod no_useless_promise_resolve_reject;
pub mod no_useless_rename;
pub mod no_useless_return;
pub mod no_useless_spread;
pub mod no_useless_switch_case;
pub mod no_useless_undefined;
pub mod no_var;
pub mod no_void;
pub mod no_warning_comments;
pub mod no_with;
pub mod no_zero_fractions;
pub mod number_arg_out_of_range;
pub mod number_literal_case;
pub mod numeric_separators_style;
pub mod only_used_in_recursion;
pub mod operator_assignment;
pub mod prefer_add_event_listener;
pub mod prefer_array_find;
pub mod prefer_array_flat;
pub mod prefer_array_flat_map;
pub mod prefer_array_index_of;
pub mod prefer_array_some;
pub mod prefer_at;
pub mod prefer_bigint_literals;
pub mod prefer_blob_reading_methods;
pub mod prefer_class_fields;
pub mod prefer_classlist_toggle;
pub mod prefer_code_point;
pub mod prefer_const;
pub mod prefer_date_now;
pub mod prefer_default_parameters;
pub mod prefer_destructuring;
pub mod prefer_dom_node_append;
pub mod prefer_dom_node_dataset;
pub mod prefer_dom_node_remove;
pub mod prefer_dom_node_text_content;
pub mod prefer_event_target;
pub mod prefer_exponentiation_operator;
pub mod prefer_global_this;
pub mod prefer_includes;
pub mod prefer_keyboard_event_key;
pub mod prefer_logical_operator_over_ternary;
pub mod prefer_math_min_max;
pub mod prefer_math_trunc;
pub mod prefer_modern_dom_apis;
pub mod prefer_modern_math_apis;
pub mod prefer_module;
pub mod prefer_native_coercion_functions;
pub mod prefer_negative_index;
pub mod prefer_node_protocol;
pub mod prefer_number_properties;
pub mod prefer_numeric_literals;
pub mod prefer_object_from_entries;
pub mod prefer_object_has_own;
pub mod prefer_object_spread;
pub mod prefer_optional_catch_binding;
pub mod prefer_promise_reject_errors;
pub mod prefer_prototype_methods;
pub mod prefer_query_selector;
pub mod prefer_reflect_apply;
pub mod prefer_regexp_test;
pub mod prefer_response_static_json;
pub mod prefer_rest_params;
pub mod prefer_set_has;
pub mod prefer_set_size;
pub mod prefer_spread;
pub mod prefer_string_raw;
pub mod prefer_string_replace_all;
pub mod prefer_string_slice;
pub mod prefer_string_starts_ends_with;
pub mod prefer_string_trim_start_end;
pub mod prefer_structured_clone;
pub mod prefer_switch;
pub mod prefer_template;
pub mod prefer_ternary;
pub mod prefer_top_level_await;
pub mod prefer_type_error;
pub mod preserve_caught_error;
pub mod prevent_abbreviations;
pub mod radix;
pub mod relative_url_style;
pub mod require_array_join_separator;
pub mod require_await;
pub mod require_module_attributes;
pub mod require_module_specifiers;
pub mod require_number_to_fixed_digits_argument;
pub mod require_post_message_target_origin;
pub mod require_yield;
pub mod sort_imports;
pub mod sort_keys;
pub mod sort_vars;
pub mod switch_case_braces;
pub mod symbol_description;
pub mod text_encoding_identifier_case;
pub mod throw_new_error;
pub mod unicode_bom;
pub mod uninvoked_array_callback;
pub mod use_isnan;
pub mod valid_typeof;
pub mod vars_on_top;
pub mod yoda;

// Plugin category submodules (prefixed rule names).
pub mod import;
pub mod jest;
pub mod jsdoc;
pub mod jsx_a11y;
pub mod nextjs;
pub mod node;
pub mod promise;
pub mod react;
pub mod react_perf;
pub mod storybook;
pub mod typescript;
pub mod vitest;
pub mod vue;

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::RuleMeta;

use crate::lint_rule::LintRule;

// ---------------------------------------------------------------------------
// Native plugin registry
// ---------------------------------------------------------------------------

/// A named group of native lint rules that functions as a plugin.
pub struct NativePlugin {
    /// Plugin name (e.g., "core", "react", "typescript").
    pub name: &'static str,
    /// Factory function returning all rules in this bundle.
    pub factory: fn() -> Vec<Box<dyn LintRule>>,
}

/// All built-in native plugin bundles.
///
/// Each bundle corresponds to a named plugin in the config:
/// ```toml
/// [plugins]
/// core = true
/// react = true
/// ```
#[must_use]
pub fn native_plugin_registry() -> Vec<NativePlugin> {
    vec![
        NativePlugin {
            name: "core",
            factory: core_rules,
        },
        NativePlugin {
            name: "react",
            factory: || {
                let mut rules = react_rules();
                rules.extend(jsx_a11y_rules());
                rules.extend(react_perf_rules());
                rules
            },
        },
        NativePlugin {
            name: "typescript",
            factory: typescript_rules,
        },
        NativePlugin {
            name: "testing",
            factory: || {
                let mut rules = jest_rules();
                rules.extend(vitest_rules());
                rules
            },
        },
        NativePlugin {
            name: "modules",
            factory: || {
                let mut rules = import_rules();
                rules.extend(node_rules());
                rules.extend(promise_rules());
                rules
            },
        },
        NativePlugin {
            name: "nextjs",
            factory: nextjs_rules,
        },
        NativePlugin {
            name: "vue",
            factory: vue_rules,
        },
        NativePlugin {
            name: "jsdoc",
            factory: jsdoc_rules,
        },
        NativePlugin {
            name: "storybook",
            factory: storybook_rules,
        },
    ]
}

/// Return metadata for all built-in rules.
#[must_use]
pub fn all_rule_metas() -> Vec<RuleMeta> {
    all_lint_rules().iter().map(|r| r.meta()).collect()
}

/// Parse a severity string from config into a [`Severity`].
///
/// Returns `Ok(None)` for "off" (rule disabled).
/// Returns `Err` for unrecognized severity strings.
pub fn parse_severity(s: &str) -> Result<Option<Severity>, String> {
    match s {
        "error" => Ok(Some(Severity::Error)),
        "warn" | "warning" => Ok(Some(Severity::Warning)),
        "off" => Ok(None),
        _ => Err(format!(
            "unknown severity `{s}`; expected \"error\", \"warn\", or \"off\""
        )),
    }
}

// ---------------------------------------------------------------------------
// Rule bundle factory functions
// ---------------------------------------------------------------------------

/// Return all [`LintRule`] implementations across all bundles.
#[must_use]
pub fn all_lint_rules() -> Vec<Box<dyn LintRule>> {
    let mut rules = core_rules();
    rules.extend(react_rules());
    rules.extend(jsx_a11y_rules());
    rules.extend(react_perf_rules());
    rules.extend(typescript_rules());
    rules.extend(jest_rules());
    rules.extend(vitest_rules());
    rules.extend(import_rules());
    rules.extend(node_rules());
    rules.extend(promise_rules());
    rules.extend(nextjs_rules());
    rules.extend(vue_rules());
    rules.extend(jsdoc_rules());
    rules.extend(storybook_rules());
    rules
}

/// Rules for the `core` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn core_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::eqeqeq::Eqeqeq),
        Box::new(self::no_debugger::NoDebugger),
        Box::new(self::no_empty::NoEmpty),
        Box::new(self::no_var::NoVar),
        Box::new(self::accessor_pairs::AccessorPairs),
        Box::new(self::approx_constant::ApproxConstant),
        Box::new(self::array_callback_return::ArrayCallbackReturn),
        Box::new(self::arrow_body_style::ArrowBodyStyle),
        Box::new(self::bad_array_method_on_arguments::BadArrayMethodOnArguments),
        Box::new(self::bad_bitwise_operator::BadBitwiseOperator),
        Box::new(self::bad_char_at_comparison::BadCharAtComparison),
        Box::new(self::bad_comparison_sequence::BadComparisonSequence),
        Box::new(self::bad_min_max_func::BadMinMaxFunc),
        Box::new(self::bad_object_literal_comparison::BadObjectLiteralComparison),
        Box::new(self::bad_replace_all_arg::BadReplaceAllArg),
        Box::new(self::better_regex::BetterRegex),
        Box::new(self::block_scoped_var::BlockScopedVar::new()),
        Box::new(self::branches_sharing_code::BranchesSharingCode),
        Box::new(self::capitalized_comments::CapitalizedComments),
        Box::new(self::catch_error_name::CatchErrorName::new()),
        Box::new(self::class_methods_use_this::ClassMethodsUseThis::new()),
        Box::new(self::consistent_assert::ConsistentAssert),
        Box::new(self::consistent_date_clone::ConsistentDateClone),
        Box::new(self::consistent_destructuring::ConsistentDestructuring::new()),
        Box::new(self::consistent_empty_array_spread::ConsistentEmptyArraySpread),
        Box::new(self::consistent_existence_index_check::ConsistentExistenceIndexCheck),
        Box::new(self::consistent_function_scoping::ConsistentFunctionScoping::new()),
        Box::new(self::consistent_template_literal_escape::ConsistentTemplateLiteralEscape),
        Box::new(self::const_comparisons::ConstComparisons),
        Box::new(self::constructor_super::ConstructorSuper),
        Box::new(self::curly::Curly),
        Box::new(self::default_case::DefaultCase),
        Box::new(self::default_case_last::DefaultCaseLast),
        Box::new(self::default_param_last::DefaultParamLast),
        Box::new(self::double_comparisons::DoubleComparisons),
        Box::new(self::empty_brace_spaces::EmptyBraceSpaces),
        Box::new(self::erasing_op::ErasingOp),
        Box::new(self::error_message::ErrorMessage),
        Box::new(self::escape_case::EscapeCase),
        Box::new(self::expiring_todo_comments::ExpiringTodoComments),
        Box::new(self::explicit_length_check::ExplicitLengthCheck),
        Box::new(self::filename_case::FilenameCase),
        Box::new(self::for_direction::ForDirection),
        Box::new(self::func_names::FuncNames),
        Box::new(self::func_style::FuncStyle),
        Box::new(self::getter_return::GetterReturn),
        Box::new(self::grouped_accessor_pairs::GroupedAccessorPairs),
        Box::new(self::guard_for_in::GuardForIn),
        Box::new(self::id_length::IdLength::new()),
        Box::new(self::init_declarations::InitDeclarations),
        Box::new(self::max_classes_per_file::MaxClassesPerFile::new()),
        Box::new(self::max_complexity::MaxComplexity::new()),
        Box::new(self::max_depth::MaxDepth::new()),
        Box::new(self::max_lines::MaxLines::new()),
        Box::new(self::max_lines_per_function::MaxLinesPerFunction::new()),
        Box::new(self::max_nested_callbacks::MaxNestedCallbacks::new()),
        Box::new(self::max_params::MaxParams::new()),
        Box::new(self::max_statements::MaxStatements::new()),
        Box::new(self::misrefactored_assign_op::MisrefactoredAssignOp),
        Box::new(self::missing_throw::MissingThrow),
        Box::new(self::new_cap::NewCap),
        Box::new(self::new_for_builtins::NewForBuiltins),
        Box::new(self::no_abusive_eslint_disable::NoAbusiveEslintDisable),
        Box::new(self::no_accessor_recursion::NoAccessorRecursion),
        Box::new(self::no_accumulating_spread::NoAccumulatingSpread),
        Box::new(self::no_alert::NoAlert),
        Box::new(self::no_array_callback_reference::NoArrayCallbackReference),
        Box::new(self::no_array_constructor::NoArrayConstructor),
        Box::new(self::no_array_for_each::NoArrayForEach),
        Box::new(self::no_array_method_this_argument::NoArrayMethodThisArgument),
        Box::new(self::no_array_push_push::NoArrayPushPush::new()),
        Box::new(self::no_array_reduce::NoArrayReduce),
        Box::new(self::no_array_reverse::NoArrayReverse),
        Box::new(self::no_array_sort::NoArraySort),
        Box::new(self::no_async_await::NoAsyncAwait),
        Box::new(self::no_async_endpoint_handlers::NoAsyncEndpointHandlers),
        Box::new(self::no_async_promise_executor::NoAsyncPromiseExecutor),
        Box::new(self::no_await_expression_member::NoAwaitExpressionMember),
        Box::new(self::no_await_in_loop::NoAwaitInLoop::new()),
        Box::new(self::no_await_in_promise_methods::NoAwaitInPromiseMethods),
        Box::new(self::no_barrel_file::NoBarrelFile),
        Box::new(self::no_bitwise::NoBitwise),
        Box::new(self::no_caller::NoCaller),
        Box::new(self::no_case_declarations::NoCaseDeclarations),
        Box::new(self::no_class_assign::NoClassAssign),
        Box::new(self::no_compare_neg_zero::NoCompareNegZero),
        Box::new(self::no_cond_assign::NoCondAssign),
        Box::new(self::no_console::NoConsole),
        Box::new(self::no_console_spaces::NoConsoleSpaces),
        Box::new(self::no_const_assign::NoConstAssign),
        Box::new(self::no_const_enum::NoConstEnum),
        Box::new(self::no_constant_binary_expression::NoConstantBinaryExpression),
        Box::new(self::no_constant_condition::NoConstantCondition),
        Box::new(self::no_constructor_return::NoConstructorReturn),
        Box::new(self::no_continue::NoContinue),
        Box::new(self::no_control_regex::NoControlRegex),
        Box::new(self::no_delete_var::NoDeleteVar),
        Box::new(self::no_div_regex::NoDivRegex),
        Box::new(self::no_document_cookie::NoDocumentCookie),
        Box::new(self::no_dupe_class_members::NoDupeClassMembers),
        Box::new(self::no_dupe_else_if::NoDupeElseIf),
        Box::new(self::no_duplicate_case::NoDuplicateCase),
        Box::new(self::no_duplicate_imports::NoDuplicateImports),
        Box::new(self::no_else_return::NoElseReturn),
        Box::new(self::no_empty_character_class::NoEmptyCharacterClass),
        Box::new(self::no_empty_file::NoEmptyFile),
        Box::new(self::no_empty_function::NoEmptyFunction),
        Box::new(self::no_empty_pattern::NoEmptyPattern),
        Box::new(self::no_empty_static_block::NoEmptyStaticBlock),
        Box::new(self::no_eq_null::NoEqNull),
        Box::new(self::no_eval::NoEval),
        Box::new(self::no_ex_assign::NoExAssign),
        Box::new(self::no_extend_native::NoExtendNative),
        Box::new(self::no_extra_bind::NoExtraBind),
        Box::new(self::no_extra_boolean_cast::NoExtraBooleanCast),
        Box::new(self::no_extra_label::NoExtraLabel),
        Box::new(self::no_extra_semi::NoExtraSemi),
        Box::new(self::no_fallthrough::NoFallthrough),
        Box::new(self::no_func_assign::NoFuncAssign),
        Box::new(self::no_global_assign::NoGlobalAssign),
        Box::new(self::no_hex_escape::NoHexEscape),
        Box::new(self::no_immediate_mutation::NoImmediateMutation),
        Box::new(self::no_implicit_coercion::NoImplicitCoercion),
        Box::new(self::no_import_assign::NoImportAssign),
        Box::new(self::no_inline_comments::NoInlineComments),
        Box::new(self::no_inner_declarations::NoInnerDeclarations),
        Box::new(self::no_instanceof_array::NoInstanceofArray),
        Box::new(self::no_instanceof_builtins::NoInstanceofBuiltins),
        Box::new(self::no_invalid_fetch_options::NoInvalidFetchOptions),
        Box::new(self::no_invalid_regexp::NoInvalidRegexp),
        Box::new(self::no_invalid_remove_event_listener::NoInvalidRemoveEventListener),
        Box::new(self::no_irregular_whitespace::NoIrregularWhitespace),
        Box::new(self::no_iterator::NoIterator),
        Box::new(self::no_keyword_prefix::NoKeywordPrefix),
        Box::new(self::no_label_var::NoLabelVar),
        Box::new(self::no_labels::NoLabels),
        Box::new(self::no_length_as_slice_end::NoLengthAsSliceEnd),
        Box::new(self::no_lone_blocks::NoLoneBlocks),
        Box::new(self::no_lonely_if::NoLonelyIf),
        Box::new(self::no_loop_func::NoLoopFunc),
        Box::new(self::no_loss_of_precision::NoLossOfPrecision),
        Box::new(self::no_magic_array_flat_depth::NoMagicArrayFlatDepth),
        Box::new(self::no_magic_numbers::NoMagicNumbers),
        Box::new(self::no_map_spread::NoMapSpread),
        Box::new(self::no_misleading_character_class::NoMisleadingCharacterClass),
        Box::new(self::no_multi_assign::NoMultiAssign),
        Box::new(self::no_multi_str::NoMultiStr),
        Box::new(self::no_negated_condition::NoNegatedCondition),
        Box::new(self::no_negation_in_equality_check::NoNegationInEqualityCheck),
        Box::new(self::no_nested_ternary::NoNestedTernary),
        Box::new(self::no_new::NoNew),
        Box::new(self::no_new_array::NoNewArray),
        Box::new(self::no_new_buffer::NoNewBuffer),
        Box::new(self::no_new_func::NoNewFunc),
        Box::new(self::no_new_native_nonconstructor::NoNewNativeNonconstructor),
        Box::new(self::no_new_wrappers::NoNewWrappers),
        Box::new(self::no_nonoctal_decimal_escape::NoNonoctalDecimalEscape),
        Box::new(self::no_null::NoNull),
        Box::new(self::no_obj_calls::NoObjCalls),
        Box::new(self::no_object_as_default_parameter::NoObjectAsDefaultParameter),
        Box::new(self::no_object_constructor::NoObjectConstructor),
        Box::new(self::no_optional_chaining::NoOptionalChaining),
        Box::new(self::no_param_reassign::NoParamReassign),
        Box::new(self::no_plusplus::NoPlusplus),
        Box::new(self::no_promise_executor_return::NoPromiseExecutorReturn),
        Box::new(self::no_proto::NoProto),
        Box::new(self::no_prototype_builtins::NoPrototypeBuiltins),
        Box::new(self::no_redeclare::NoRedeclare),
        Box::new(self::no_regex_spaces::NoRegexSpaces),
        Box::new(self::no_rest_spread_properties::NoRestSpreadProperties),
        Box::new(self::no_restricted_globals::NoRestrictedGlobals::new()),
        Box::new(self::no_return_assign::NoReturnAssign),
        Box::new(self::no_script_url::NoScriptUrl),
        Box::new(self::no_self_assign::NoSelfAssign),
        Box::new(self::no_self_compare::NoSelfCompare),
        Box::new(self::no_sequences::NoSequences),
        Box::new(self::no_setter_return::NoSetterReturn),
        Box::new(self::no_shadow::NoShadow),
        Box::new(self::no_shadow_restricted_names::NoShadowRestrictedNames),
        Box::new(self::no_single_promise_in_promise_methods::NoSinglePromiseInPromiseMethods),
        Box::new(self::no_sparse_arrays::NoSparseArrays),
        Box::new(self::no_static_only_class::NoStaticOnlyClass),
        Box::new(self::no_template_curly_in_string::NoTemplateCurlyInString),
        Box::new(self::no_ternary::NoTernary),
        Box::new(self::no_thenable::NoThenable),
        Box::new(self::no_this_assignment::NoThisAssignment),
        Box::new(self::no_this_before_super::NoThisBeforeSuper),
        Box::new(self::no_this_in_exported_function::NoThisInExportedFunction),
        Box::new(self::no_throw_literal::NoThrowLiteral),
        Box::new(self::no_typeof_undefined::NoTypeofUndefined),
        Box::new(self::no_unassigned_vars::NoUnassignedVars),
        Box::new(self::no_undef::NoUndef),
        Box::new(self::no_undefined::NoUndefined),
        Box::new(self::no_unexpected_multiline::NoUnexpectedMultiline),
        Box::new(self::no_unmodified_loop_condition::NoUnmodifiedLoopCondition),
        Box::new(self::no_unnecessary_array_flat_depth::NoUnnecessaryArrayFlatDepth),
        Box::new(self::no_unnecessary_array_splice_count::NoUnnecessaryArraySpliceCount),
        Box::new(self::no_unnecessary_await::NoUnnecessaryAwait),
        Box::new(self::no_unnecessary_slice_end::NoUnnecessarySliceEnd),
        Box::new(self::no_unneeded_ternary::NoUnneededTernary),
        Box::new(self::no_unreachable::NoUnreachable),
        Box::new(self::no_unreadable_array_destructuring::NoUnreadableArrayDestructuring),
        Box::new(self::no_unreadable_iife::NoUnreadableIife),
        Box::new(self::no_unsafe_finally::NoUnsafeFinally),
        Box::new(self::no_unsafe_negation::NoUnsafeNegation),
        Box::new(self::no_unsafe_optional_chaining::NoUnsafeOptionalChaining),
        Box::new(self::no_unused_expressions::NoUnusedExpressions),
        Box::new(self::no_unused_labels::NoUnusedLabels),
        Box::new(self::no_unused_private_class_members::NoUnusedPrivateClassMembers),
        Box::new(self::no_unused_vars::NoUnusedVars),
        Box::new(self::no_use_before_define::NoUseBeforeDefine),
        Box::new(self::no_useless_backreference::NoUselessBackreference),
        Box::new(self::no_useless_call::NoUselessCall),
        Box::new(self::no_useless_catch::NoUselessCatch),
        Box::new(self::no_useless_collection_argument::NoUselessCollectionArgument),
        Box::new(self::no_useless_computed_key::NoUselessComputedKey),
        Box::new(self::no_useless_concat::NoUselessConcat),
        Box::new(self::no_useless_constructor::NoUselessConstructor),
        Box::new(self::no_useless_error_capture_stack_trace::NoUselessErrorCaptureStackTrace),
        Box::new(self::no_useless_escape::NoUselessEscape),
        Box::new(self::no_useless_fallback_in_spread::NoUselessFallbackInSpread),
        Box::new(self::no_useless_length_check::NoUselessLengthCheck),
        Box::new(self::no_useless_promise_resolve_reject::NoUselessPromiseResolveReject),
        Box::new(self::no_useless_rename::NoUselessRename),
        Box::new(self::no_useless_return::NoUselessReturn),
        Box::new(self::no_useless_spread::NoUselessSpread),
        Box::new(self::no_useless_switch_case::NoUselessSwitchCase),
        Box::new(self::no_useless_undefined::NoUselessUndefined),
        Box::new(self::no_void::NoVoid),
        Box::new(self::no_warning_comments::NoWarningComments),
        Box::new(self::no_with::NoWith),
        Box::new(self::no_zero_fractions::NoZeroFractions),
        Box::new(self::number_arg_out_of_range::NumberArgOutOfRange),
        Box::new(self::number_literal_case::NumberLiteralCase),
        Box::new(self::numeric_separators_style::NumericSeparatorsStyle),
        Box::new(self::only_used_in_recursion::OnlyUsedInRecursion),
        Box::new(self::operator_assignment::OperatorAssignment),
        Box::new(self::prefer_add_event_listener::PreferAddEventListener),
        Box::new(self::prefer_array_find::PreferArrayFind),
        Box::new(self::prefer_array_flat::PreferArrayFlat),
        Box::new(self::prefer_array_flat_map::PreferArrayFlatMap),
        Box::new(self::prefer_array_index_of::PreferArrayIndexOf),
        Box::new(self::prefer_array_some::PreferArraySome),
        Box::new(self::prefer_at::PreferAt),
        Box::new(self::prefer_bigint_literals::PreferBigintLiterals),
        Box::new(self::prefer_blob_reading_methods::PreferBlobReadingMethods),
        Box::new(self::prefer_class_fields::PreferClassFields),
        Box::new(self::prefer_classlist_toggle::PreferClasslistToggle),
        Box::new(self::prefer_code_point::PreferCodePoint),
        Box::new(self::prefer_const::PreferConst),
        Box::new(self::prefer_date_now::PreferDateNow),
        Box::new(self::prefer_default_parameters::PreferDefaultParameters),
        Box::new(self::prefer_destructuring::PreferDestructuring),
        Box::new(self::prefer_dom_node_append::PreferDomNodeAppend),
        Box::new(self::prefer_dom_node_dataset::PreferDomNodeDataset),
        Box::new(self::prefer_dom_node_remove::PreferDomNodeRemove),
        Box::new(self::prefer_dom_node_text_content::PreferDomNodeTextContent),
        Box::new(self::prefer_event_target::PreferEventTarget),
        Box::new(self::prefer_exponentiation_operator::PreferExponentiationOperator),
        Box::new(self::prefer_global_this::PreferGlobalThis),
        Box::new(self::prefer_keyboard_event_key::PreferKeyboardEventKey),
        Box::new(self::prefer_logical_operator_over_ternary::PreferLogicalOperatorOverTernary),
        Box::new(self::prefer_math_min_max::PreferMathMinMax),
        Box::new(self::prefer_math_trunc::PreferMathTrunc),
        Box::new(self::prefer_modern_dom_apis::PreferModernDomApis),
        Box::new(self::prefer_modern_math_apis::PreferModernMathApis),
        Box::new(self::prefer_module::PreferModule),
        Box::new(self::prefer_native_coercion_functions::PreferNativeCoercionFunctions),
        Box::new(self::prefer_negative_index::PreferNegativeIndex),
        Box::new(self::prefer_node_protocol::PreferNodeProtocol),
        Box::new(self::prefer_number_properties::PreferNumberProperties),
        Box::new(self::prefer_numeric_literals::PreferNumericLiterals),
        Box::new(self::prefer_object_from_entries::PreferObjectFromEntries),
        Box::new(self::prefer_object_has_own::PreferObjectHasOwn),
        Box::new(self::prefer_object_spread::PreferObjectSpread),
        Box::new(self::prefer_optional_catch_binding::PreferOptionalCatchBinding),
        Box::new(self::prefer_prototype_methods::PreferPrototypeMethods),
        Box::new(self::prefer_query_selector::PreferQuerySelector),
        Box::new(self::prefer_reflect_apply::PreferReflectApply),
        Box::new(self::prefer_regexp_test::PreferRegexpTest),
        Box::new(self::prefer_response_static_json::PreferResponseStaticJson),
        Box::new(self::prefer_rest_params::PreferRestParams),
        Box::new(self::prefer_set_has::PreferSetHas),
        Box::new(self::prefer_set_size::PreferSetSize),
        Box::new(self::prefer_spread::PreferSpread),
        Box::new(self::prefer_string_raw::PreferStringRaw),
        Box::new(self::prefer_string_replace_all::PreferStringReplaceAll),
        Box::new(self::prefer_string_slice::PreferStringSlice),
        Box::new(self::prefer_string_trim_start_end::PreferStringTrimStartEnd),
        Box::new(self::prefer_structured_clone::PreferStructuredClone),
        Box::new(self::prefer_switch::PreferSwitch),
        Box::new(self::prefer_template::PreferTemplate),
        Box::new(self::prefer_ternary::PreferTernary),
        Box::new(self::prefer_top_level_await::PreferTopLevelAwait),
        Box::new(self::prefer_type_error::PreferTypeError),
        Box::new(self::preserve_caught_error::PreserveCaughtError),
        Box::new(self::prevent_abbreviations::PreventAbbreviations),
        Box::new(self::radix::Radix),
        Box::new(self::relative_url_style::RelativeUrlStyle),
        Box::new(self::require_array_join_separator::RequireArrayJoinSeparator),
        Box::new(self::require_module_attributes::RequireModuleAttributes),
        Box::new(self::require_module_specifiers::RequireModuleSpecifiers),
        Box::new(self::require_number_to_fixed_digits_argument::RequireNumberToFixedDigitsArgument),
        Box::new(self::require_post_message_target_origin::RequirePostMessageTargetOrigin),
        Box::new(self::require_yield::RequireYield),
        Box::new(self::sort_imports::SortImports),
        Box::new(self::sort_keys::SortKeys),
        Box::new(self::sort_vars::SortVars),
        Box::new(self::switch_case_braces::SwitchCaseBraces),
        Box::new(self::symbol_description::SymbolDescription),
        Box::new(self::text_encoding_identifier_case::TextEncodingIdentifierCase),
        Box::new(self::throw_new_error::ThrowNewError),
        Box::new(self::unicode_bom::UnicodeBom),
        Box::new(self::uninvoked_array_callback::UninvokedArrayCallback),
        Box::new(self::use_isnan::UseIsnan),
        Box::new(self::valid_typeof::ValidTypeof),
        Box::new(self::vars_on_top::VarsOnTop),
        Box::new(self::yoda::Yoda),
    ]
}

/// Rules for the `react` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn react_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::react::button_has_type::ButtonHasType),
        Box::new(
            self::react::checked_requires_onchange_or_readonly::CheckedRequiresOnchangeOrReadonly,
        ),
        Box::new(self::react::display_name::DisplayName),
        Box::new(self::react::exhaustive_deps::ExhaustiveDeps),
        Box::new(self::react::forbid_dom_props::ForbidDomProps),
        Box::new(self::react::forbid_elements::ForbidElements),
        Box::new(self::react::forward_ref_uses_ref::ForwardRefUsesRef),
        Box::new(self::react::iframe_missing_sandbox::IframeMissingSandbox),
        Box::new(self::react::jsx_boolean_value::JsxBooleanValue),
        Box::new(self::react::jsx_curly_brace_presence::JsxCurlyBracePresence),
        Box::new(self::react::jsx_filename_extension::JsxFilenameExtension),
        Box::new(self::react::jsx_fragments::JsxFragments),
        Box::new(self::react::jsx_handler_names::JsxHandlerNames),
        Box::new(self::react::jsx_key::JsxKey),
        Box::new(self::react::jsx_max_depth::JsxMaxDepth),
        Box::new(self::react::jsx_no_comment_textnodes::JsxNoCommentTextnodes),
        Box::new(self::react::jsx_no_constructed_context_values::JsxNoConstructedContextValues),
        Box::new(self::react::jsx_no_duplicate_props::JsxNoDuplicateProps),
        Box::new(self::react::jsx_no_script_url::JsxNoScriptUrl),
        Box::new(self::react::jsx_no_target_blank::JsxNoTargetBlank),
        Box::new(self::react::jsx_no_undef::JsxNoUndef),
        Box::new(self::react::jsx_no_useless_fragment::JsxNoUselessFragment),
        Box::new(self::react::jsx_pascal_case::JsxPascalCase),
        Box::new(self::react::jsx_props_no_spread_multi::JsxPropsNoSpreadMulti),
        Box::new(self::react::jsx_props_no_spreading::JsxPropsNoSpreading),
        Box::new(self::react::no_array_index_key::NoArrayIndexKey),
        Box::new(self::react::no_children_prop::NoChildrenProp),
        Box::new(self::react::no_danger::NoDanger),
        Box::new(self::react::no_danger_with_children::NoDangerWithChildren),
        Box::new(self::react::no_did_mount_set_state::NoDidMountSetState),
        Box::new(self::react::no_direct_mutation_state::NoDirectMutationState),
        Box::new(self::react::no_find_dom_node::NoFindDomNode),
        Box::new(self::react::no_is_mounted::NoIsMounted),
        Box::new(self::react::no_multi_comp::NoMultiComp),
        Box::new(
            self::react::no_redundant_should_component_update::NoRedundantShouldComponentUpdate,
        ),
        Box::new(self::react::no_render_return_value::NoRenderReturnValue),
        Box::new(self::react::no_set_state::NoSetState),
        Box::new(self::react::no_string_refs::NoStringRefs),
        Box::new(self::react::no_this_in_sfc::NoThisInSfc),
        Box::new(self::react::no_unescaped_entities::NoUnescapedEntities),
        Box::new(self::react::no_unknown_property::NoUnknownProperty),
        Box::new(self::react::no_unsafe::NoUnsafe),
        Box::new(self::react::no_will_update_set_state::NoWillUpdateSetState),
        Box::new(self::react::only_export_components::OnlyExportComponents),
        Box::new(self::react::prefer_es6_class::PreferEs6Class),
        Box::new(self::react::react_in_jsx_scope::ReactInJsxScope),
        Box::new(self::react::require_render_return::RequireRenderReturn),
        Box::new(self::react::rules_of_hooks::RulesOfHooks::new()),
        Box::new(self::react::self_closing_comp::SelfClosingComp),
        Box::new(self::react::state_in_constructor::StateInConstructor),
        Box::new(self::react::style_prop_object::StylePropObject),
        Box::new(self::react::void_dom_elements_no_children::VoidDomElementsNoChildren),
    ]
}

/// Rules for the `jsx_a11y` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn jsx_a11y_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::jsx_a11y::alt_text::AltText),
        Box::new(self::jsx_a11y::anchor_ambiguous_text::AnchorAmbiguousText),
        Box::new(self::jsx_a11y::anchor_has_content::AnchorHasContent),
        Box::new(self::jsx_a11y::anchor_is_valid::AnchorIsValid),
        Box::new(
            self::jsx_a11y::aria_activedescendant_has_tabindex::AriaActivedescendantHasTabindex,
        ),
        Box::new(self::jsx_a11y::aria_props::AriaProps),
        Box::new(self::jsx_a11y::aria_proptypes::AriaProptypes),
        Box::new(self::jsx_a11y::aria_role::AriaRole),
        Box::new(self::jsx_a11y::aria_unsupported_elements::AriaUnsupportedElements),
        Box::new(self::jsx_a11y::autocomplete_valid::AutocompleteValid),
        Box::new(self::jsx_a11y::click_events_have_key_events::ClickEventsHaveKeyEvents),
        Box::new(self::jsx_a11y::heading_has_content::HeadingHasContent),
        Box::new(self::jsx_a11y::html_has_lang::HtmlHasLang),
        Box::new(self::jsx_a11y::iframe_has_title::IframeHasTitle),
        Box::new(self::jsx_a11y::img_redundant_alt::ImgRedundantAlt),
        Box::new(self::jsx_a11y::label_has_associated_control::LabelHasAssociatedControl),
        Box::new(self::jsx_a11y::lang::Lang),
        Box::new(self::jsx_a11y::media_has_caption::MediaHasCaption),
        Box::new(self::jsx_a11y::mouse_events_have_key_events::MouseEventsHaveKeyEvents),
        Box::new(self::jsx_a11y::no_access_key::NoAccessKey),
        Box::new(self::jsx_a11y::no_aria_hidden_on_focusable::NoAriaHiddenOnFocusable),
        Box::new(self::jsx_a11y::no_autofocus::NoAutofocus),
        Box::new(self::jsx_a11y::no_distracting_elements::NoDistractingElements),
        Box::new(self::jsx_a11y::no_noninteractive_tabindex::NoNoninteractiveTabindex),
        Box::new(self::jsx_a11y::no_redundant_roles::NoRedundantRoles),
        Box::new(self::jsx_a11y::no_static_element_interactions::NoStaticElementInteractions),
        Box::new(self::jsx_a11y::prefer_tag_over_role::PreferTagOverRole),
        Box::new(self::jsx_a11y::role_has_required_aria_props::RoleHasRequiredAriaProps),
        Box::new(self::jsx_a11y::role_supports_aria_props::RoleSupportAriaProps),
        Box::new(self::jsx_a11y::scope::Scope),
        Box::new(self::jsx_a11y::tabindex_no_positive::TabindexNoPositive),
    ]
}

/// Rules for the `react_perf` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn react_perf_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::react_perf::jsx_no_jsx_as_prop::JsxNoJsxAsProp),
        Box::new(self::react_perf::jsx_no_new_array_as_prop::JsxNoNewArrayAsProp),
        Box::new(self::react_perf::jsx_no_new_function_as_prop::JsxNoNewFunctionAsProp),
        Box::new(self::react_perf::jsx_no_new_object_as_prop::JsxNoNewObjectAsProp),
    ]
}

/// Rules for the `typescript` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn typescript_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::typescript::adjacent_overload_signatures::AdjacentOverloadSignatures),
        Box::new(self::typescript::array_type::ArrayType),
        Box::new(self::typescript::await_thenable::AwaitThenable),
        Box::new(self::typescript::ban_ts_comment::BanTsComment),
        Box::new(self::typescript::ban_tslint_comment::BanTslintComment),
        Box::new(self::typescript::ban_types::BanTypes),
        Box::new(self::typescript::consistent_generic_constructors::ConsistentGenericConstructors),
        Box::new(self::typescript::consistent_indexed_object_style::ConsistentIndexedObjectStyle),
        Box::new(self::typescript::consistent_return::ConsistentReturn),
        Box::new(self::typescript::consistent_type_assertions::ConsistentTypeAssertions),
        Box::new(self::typescript::consistent_type_definitions::ConsistentTypeDefinitions),
        Box::new(self::typescript::consistent_type_exports::ConsistentTypeExports),
        Box::new(self::typescript::consistent_type_imports::ConsistentTypeImports),
        Box::new(self::typescript::dot_notation::DotNotation),
        Box::new(self::typescript::explicit_function_return_type::ExplicitFunctionReturnType),
        Box::new(self::typescript::explicit_module_boundary_types::ExplicitModuleBoundaryTypes),
        Box::new(self::typescript::no_array_delete::NoArrayDelete),
        Box::new(self::typescript::no_base_to_string::NoBaseToString),
        Box::new(self::typescript::no_confusing_non_null_assertion::NoConfusingNonNullAssertion),
        Box::new(self::typescript::no_confusing_void_expression::NoConfusingVoidExpression),
        Box::new(self::typescript::no_deprecated::NoDeprecated),
        Box::new(self::typescript::no_duplicate_enum_values::NoDuplicateEnumValues),
        Box::new(self::typescript::no_dynamic_delete::NoDynamicDelete),
        Box::new(self::typescript::no_empty_interface::NoEmptyInterface),
        Box::new(self::typescript::no_empty_object_type::NoEmptyObjectType),
        Box::new(self::typescript::no_explicit_any::NoExplicitAny),
        Box::new(self::typescript::no_extra_non_null_assertion::NoExtraNonNullAssertion),
        Box::new(self::typescript::no_extraneous_class::NoExtraneousClass),
        Box::new(self::typescript::no_floating_promises::NoFloatingPromises),
        Box::new(self::typescript::no_for_in_array::NoForInArray),
        Box::new(self::typescript::no_implied_eval::NoImpliedEval),
        Box::new(self::typescript::no_inferrable_types::NoInferrableTypes),
        Box::new(self::typescript::no_invalid_void_type::NoInvalidVoidType::new()),
        Box::new(self::typescript::no_misused_new::NoMisusedNew),
        Box::new(self::typescript::no_misused_promises::NoMisusedPromises),
        Box::new(self::typescript::no_misused_spread::NoMisusedSpread),
        Box::new(self::typescript::no_mixed_enums::NoMixedEnums),
        Box::new(self::typescript::no_non_null_asserted_optional_chain::NoNonNullAssertedOptionalChain),
        Box::new(self::typescript::no_non_null_assertion::NoNonNullAssertion),
        Box::new(self::typescript::no_require_imports::NoRequireImports),
        Box::new(self::typescript::no_restricted_types::NoRestrictedTypes),
        Box::new(self::typescript::no_this_alias::NoThisAlias),
        Box::new(self::typescript::no_unnecessary_boolean_literal_compare::NoUnnecessaryBooleanLiteralCompare),
        Box::new(self::typescript::no_unnecessary_condition::NoUnnecessaryCondition),
        Box::new(self::typescript::no_unnecessary_parameter_property_assignment::NoUnnecessaryParameterPropertyAssignment),
        Box::new(self::typescript::no_unnecessary_qualifier::NoUnnecessaryQualifier),
        Box::new(self::typescript::no_unnecessary_template_expression::NoUnnecessaryTemplateExpression),
        Box::new(self::typescript::no_unnecessary_type_arguments::NoUnnecessaryTypeArguments),
        Box::new(self::typescript::no_unnecessary_type_assertion::NoUnnecessaryTypeAssertion),
        Box::new(self::typescript::no_unnecessary_type_constraint::NoUnnecessaryTypeConstraint),
        Box::new(self::typescript::no_unnecessary_type_parameters::NoUnnecessaryTypeParameters),
        Box::new(self::typescript::no_unsafe_argument::NoUnsafeArgument),
        Box::new(self::typescript::no_unsafe_assignment::NoUnsafeAssignment),
        Box::new(self::typescript::no_unsafe_call::NoUnsafeCall),
        Box::new(self::typescript::no_unsafe_declaration_merging::NoUnsafeDeclarationMerging),
        Box::new(self::typescript::no_unsafe_enum_comparison::NoUnsafeEnumComparison),
        Box::new(self::typescript::no_unsafe_function_type::NoUnsafeFunctionType),
        Box::new(self::typescript::no_unsafe_member_access::NoUnsafeMemberAccess),
        Box::new(self::typescript::no_unsafe_return::NoUnsafeReturn),
        Box::new(self::typescript::no_unsafe_type_assertion::NoUnsafeTypeAssertion),
        Box::new(self::typescript::no_unsafe_unary_minus::NoUnsafeUnaryMinus),
        Box::new(self::typescript::no_useless_empty_export::NoUselessEmptyExport),
        Box::new(self::typescript::no_var_requires::NoVarRequires),
        Box::new(self::typescript::no_wrapper_object_types::NoWrapperObjectTypes),
        Box::new(self::typescript::non_nullable_type_assertion_style::NonNullableTypeAssertionStyle),
        Box::new(self::typescript::only_throw_error::OnlyThrowError),
        Box::new(self::typescript::parameter_properties::ParameterProperties),
        Box::new(self::typescript::prefer_as_const::PreferAsConst),
        Box::new(self::typescript::prefer_enum_initializers::PreferEnumInitializers),
        Box::new(self::typescript::prefer_find::PreferFind),
        Box::new(self::typescript::prefer_for_of::PreferForOf),
        Box::new(self::typescript::prefer_function_type::PreferFunctionType),
        Box::new(self::typescript::prefer_includes::PreferIncludes),
        Box::new(self::typescript::prefer_literal_enum_member::PreferLiteralEnumMember),
        Box::new(self::typescript::prefer_namespace_keyword::PreferNamespaceKeyword),
        Box::new(self::typescript::prefer_nullish_coalescing::PreferNullishCoalescing),
        Box::new(self::typescript::prefer_optional_chain::PreferOptionalChain),
        Box::new(self::typescript::prefer_promise_reject_errors::PreferPromiseRejectErrors),
        Box::new(self::typescript::prefer_readonly::PreferReadonly),
        Box::new(self::typescript::prefer_readonly_parameter_types::PreferReadonlyParameterTypes),
        Box::new(self::typescript::prefer_reduce_type_parameter::PreferReduceTypeParameter),
        Box::new(self::typescript::prefer_regexp_exec::PreferRegexpExec),
        Box::new(self::typescript::prefer_return_this_type::PreferReturnThisType),
        Box::new(self::typescript::prefer_string_starts_ends_with::PreferStringStartsEndsWith),
        Box::new(self::typescript::promise_function_async::PromiseFunctionAsync),
        Box::new(self::typescript::related_getter_setter_pairs::RelatedGetterSetterPairs),
        Box::new(self::typescript::require_array_sort_compare::RequireArraySortCompare),
        Box::new(self::typescript::require_await::RequireAwait),
        Box::new(self::typescript::restrict_plus_operands::RestrictPlusOperands),
        Box::new(self::typescript::restrict_template_expressions::RestrictTemplateExpressions),
        Box::new(self::typescript::return_await::ReturnAwait),
        Box::new(self::typescript::strict_boolean_expressions::StrictBooleanExpressions),
        Box::new(self::typescript::strict_void_return::StrictVoidReturn),
        Box::new(self::typescript::switch_exhaustiveness_check::SwitchExhaustivenessCheck),
        Box::new(self::typescript::triple_slash_reference::TripleSlashReference),
        Box::new(self::typescript::unbound_method::UnboundMethod),
        Box::new(self::typescript::unified_signatures::UnifiedSignatures),
        Box::new(self::typescript::use_unknown_in_catch_callback_variable::UseUnknownInCatchCallbackVariable),
    ]
}

/// Rules for the `jest` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn jest_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::jest::consistent_test_it::ConsistentTestIt),
        Box::new(self::jest::expect_expect::ExpectExpect),
        Box::new(self::jest::max_expects::MaxExpects::new()),
        Box::new(self::jest::max_nested_describe::MaxNestedDescribe),
        Box::new(self::jest::no_alias_methods::NoAliasMethods),
        Box::new(self::jest::no_commented_out_tests::NoCommentedOutTests),
        Box::new(self::jest::no_conditional_expect::NoConditionalExpect),
        Box::new(self::jest::no_conditional_in_test::NoConditionalInTest),
        Box::new(self::jest::no_confusing_set_timeout::NoConfusingSetTimeout),
        Box::new(self::jest::no_deprecated_functions::NoDeprecatedFunctions),
        Box::new(self::jest::no_disabled_tests::NoDisabledTests),
        Box::new(self::jest::no_done_callback::NoDoneCallback),
        Box::new(self::jest::no_duplicate_hooks::NoDuplicateHooks),
        Box::new(self::jest::no_export::NoExport),
        Box::new(self::jest::no_focused_tests::NoFocusedTests),
        Box::new(self::jest::no_hooks::NoHooks),
        Box::new(self::jest::no_identical_title::NoIdenticalTitle),
        Box::new(self::jest::no_interpolation_in_snapshots::NoInterpolationInSnapshots),
        Box::new(self::jest::no_jasmine_globals::NoJasmineGlobals),
        Box::new(self::jest::no_large_snapshots::NoLargeSnapshots),
        Box::new(self::jest::no_mocks_import::NoMocksImport),
        Box::new(self::jest::no_restricted_jest_methods::NoRestrictedJestMethods),
        Box::new(self::jest::no_restricted_matchers::NoRestrictedMatchers),
        Box::new(self::jest::no_standalone_expect::NoStandaloneExpect),
        Box::new(self::jest::no_test_prefixes::NoTestPrefixes),
        Box::new(self::jest::no_test_return_statement::NoTestReturnStatement),
        Box::new(self::jest::no_unneeded_async_expect_function::NoUnneededAsyncExpectFunction),
        Box::new(self::jest::no_untyped_mock_factory::NoUntypedMockFactory),
        Box::new(self::jest::padding_around_test_blocks::PaddingAroundTestBlocks),
        Box::new(self::jest::prefer_called_with::PreferCalledWith),
        Box::new(self::jest::prefer_comparison_matcher::PreferComparisonMatcher),
        Box::new(self::jest::prefer_each::PreferEach),
        Box::new(self::jest::prefer_equality_matcher::PreferEqualityMatcher),
        Box::new(self::jest::prefer_expect_resolves::PreferExpectResolves),
        Box::new(self::jest::prefer_hooks_in_order::PreferHooksInOrder),
        Box::new(self::jest::prefer_hooks_on_top::PreferHooksOnTop),
        Box::new(self::jest::prefer_jest_mocked::PreferJestMocked),
        Box::new(self::jest::prefer_lowercase_title::PreferLowercaseTitle),
        Box::new(self::jest::prefer_mock_promise_shorthand::PreferMockPromiseShorthand),
        Box::new(self::jest::prefer_mock_return_shorthand::PreferMockReturnShorthand),
        Box::new(self::jest::prefer_spy_on::PreferSpyOn),
        Box::new(self::jest::prefer_strict_equal::PreferStrictEqual),
        Box::new(self::jest::prefer_to_be::PreferToBe),
        Box::new(self::jest::prefer_to_contain::PreferToContain),
        Box::new(self::jest::prefer_to_have_been_called::PreferToHaveBeenCalled),
        Box::new(self::jest::prefer_to_have_been_called_times::PreferToHaveBeenCalledTimes),
        Box::new(self::jest::prefer_to_have_length::PreferToHaveLength),
        Box::new(self::jest::prefer_todo::PreferTodo),
        Box::new(self::jest::require_hook::RequireHook),
        Box::new(self::jest::require_to_throw_message::RequireToThrowMessage),
        Box::new(self::jest::require_top_level_describe::RequireTopLevelDescribe),
        Box::new(self::jest::valid_describe_callback::ValidDescribeCallback),
        Box::new(self::jest::valid_expect::ValidExpect),
        Box::new(self::jest::valid_title::ValidTitle),
    ]
}

/// Rules for the `vitest` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn vitest_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::vitest::consistent_each_for::ConsistentEachFor),
        Box::new(self::vitest::consistent_test_filename::ConsistentTestFilename),
        Box::new(self::vitest::consistent_vitest_vi::ConsistentVitestVi),
        Box::new(self::vitest::hoisted_apis_on_top::HoistedApisOnTop),
        Box::new(self::vitest::no_conditional_tests::NoConditionalTests),
        Box::new(self::vitest::no_import_node_test::NoImportNodeTest),
        Box::new(self::vitest::no_importing_vitest_globals::NoImportingVitestGlobals),
        Box::new(self::vitest::prefer_called_once::PreferCalledOnce),
        Box::new(self::vitest::prefer_called_times::PreferCalledTimes),
        Box::new(self::vitest::prefer_describe_function_title::PreferDescribeFunctionTitle),
        Box::new(self::vitest::prefer_expect_type_of::PreferExpectTypeOf),
        Box::new(self::vitest::prefer_import_in_mock::PreferImportInMock),
        Box::new(self::vitest::prefer_to_be_falsy::PreferToBeFalsy),
        Box::new(self::vitest::prefer_to_be_object::PreferToBeObject),
        Box::new(self::vitest::prefer_to_be_truthy::PreferToBeTruthy),
        Box::new(self::vitest::require_local_test_context_for_concurrent_snapshots::RequireLocalTestContextForConcurrentSnapshots),
        Box::new(self::vitest::warn_todo::WarnTodo),
    ]
}

/// Rules for the `import` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn import_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::import::consistent_type_specifier_style::ConsistentTypeSpecifierStyle),
        Box::new(self::import::default::DefaultExport),
        Box::new(self::import::export::ExportRule),
        Box::new(self::import::exports_last::ExportsLast),
        Box::new(self::import::extensions::Extensions),
        Box::new(self::import::first::First),
        Box::new(self::import::group_exports::GroupExports),
        Box::new(self::import::max_dependencies::MaxDependencies::new()),
        Box::new(self::import::named::NamedExport),
        Box::new(self::import::namespace::NamespaceImport),
        Box::new(self::import::no_absolute_path::NoAbsolutePath),
        Box::new(self::import::no_amd::NoAmd),
        Box::new(self::import::no_anonymous_default_export::NoAnonymousDefaultExport),
        Box::new(self::import::no_commonjs::NoCommonjs),
        Box::new(self::import::no_cycle::NoCycle),
        Box::new(self::import::no_default_export::NoDefaultExport),
        Box::new(self::import::no_duplicates::NoDuplicates),
        Box::new(self::import::no_dynamic_require::NoDynamicRequire),
        Box::new(self::import::no_empty_named_blocks::NoEmptyNamedBlocks),
        Box::new(self::import::no_mutable_exports::NoMutableExports),
        Box::new(self::import::no_named_as_default::NoNamedAsDefault),
        Box::new(self::import::no_named_as_default_member::NoNamedAsDefaultMember),
        Box::new(self::import::no_named_default::NoNamedDefault),
        Box::new(self::import::no_named_export::NoNamedExport),
        Box::new(self::import::no_namespace::NoNamespace),
        Box::new(self::import::no_nodejs_modules::NoNodejsModules),
        Box::new(self::import::no_relative_parent_imports::NoRelativeParentImports),
        Box::new(self::import::no_restricted_imports::NoRestrictedImports),
        Box::new(self::import::no_self_import::NoSelfImport),
        Box::new(self::import::no_unassigned_import::NoUnassignedImport),
        Box::new(self::import::no_webpack_loader_syntax::NoWebpackLoaderSyntax),
        Box::new(self::import::prefer_default_export::PreferDefaultExport),
        Box::new(self::import::unambiguous::Unambiguous),
    ]
}

/// Rules for the `node` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn node_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::node::global_require::GlobalRequire::new()),
        Box::new(self::node::no_exports_assign::NoExportsAssign),
        Box::new(self::node::no_new_require::NoNewRequire),
        Box::new(self::node::no_path_concat::NoPathConcat),
        Box::new(self::node::no_process_env::NoProcessEnv),
        Box::new(self::node::no_process_exit::NoProcessExit),
    ]
}

/// Rules for the `promise` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn promise_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::promise::always_return::AlwaysReturn),
        Box::new(self::promise::avoid_new::AvoidNew),
        Box::new(self::promise::catch_or_return::CatchOrReturn),
        Box::new(self::promise::no_callback_in_promise::NoCallbackInPromise),
        Box::new(self::promise::no_multiple_resolved::NoMultipleResolved),
        Box::new(self::promise::no_native::NoNative),
        Box::new(self::promise::no_nesting::NoNesting),
        Box::new(self::promise::no_new_statics::NoNewStatics),
        Box::new(self::promise::no_promise_in_callback::NoPromiseInCallback),
        Box::new(self::promise::no_return_in_finally::NoReturnInFinally),
        Box::new(self::promise::no_return_wrap::NoReturnWrap),
        Box::new(self::promise::param_names::ParamNames),
        Box::new(self::promise::prefer_await_to_callbacks::PreferAwaitToCallbacks),
        Box::new(self::promise::prefer_await_to_then::PreferAwaitToThen),
        Box::new(self::promise::spec_only::SpecOnly),
        Box::new(self::promise::valid_params::ValidParams),
    ]
}

/// Rules for the `nextjs` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn nextjs_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::nextjs::google_font_display::GoogleFontDisplay),
        Box::new(self::nextjs::google_font_preconnect::GoogleFontPreconnect),
        Box::new(self::nextjs::inline_script_id::InlineScriptId),
        Box::new(self::nextjs::next_script_for_ga::NextScriptForGa),
        Box::new(self::nextjs::no_assign_module_variable::NoAssignModuleVariable),
        Box::new(self::nextjs::no_async_client_component::NoAsyncClientComponent),
        Box::new(self::nextjs::no_before_interactive_script_outside_document::NoBeforeInteractiveScriptOutsideDocument),
        Box::new(self::nextjs::no_css_tags::NoCssTags),
        Box::new(self::nextjs::no_document_import_in_page::NoDocumentImportInPage),
        Box::new(self::nextjs::no_duplicate_head::NoDuplicateHead),
        Box::new(self::nextjs::no_head_element::NoHeadElement),
        Box::new(self::nextjs::no_head_import_in_document::NoHeadImportInDocument),
        Box::new(self::nextjs::no_html_link_for_pages::NoHtmlLinkForPages),
        Box::new(self::nextjs::no_img_element::NoImgElement),
        Box::new(self::nextjs::no_page_custom_font::NoPageCustomFont),
        Box::new(self::nextjs::no_script_component_in_head::NoScriptComponentInHead),
        Box::new(self::nextjs::no_styled_jsx_in_document::NoStyledJsxInDocument),
        Box::new(self::nextjs::no_sync_scripts::NoSyncScripts),
        Box::new(self::nextjs::no_title_in_document_head::NoTitleInDocumentHead),
        Box::new(self::nextjs::no_typos::NoTypos),
        Box::new(self::nextjs::no_unwanted_polyfillio::NoUnwantedPolyfillio),
    ]
}

/// Rules for the `vue` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn vue_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::vue::component_definition_name_casing::ComponentDefinitionNameCasing),
        Box::new(self::vue::custom_event_name_casing::CustomEventNameCasing),
        Box::new(self::vue::html_closing_bracket_newline::HtmlClosingBracketNewline),
        Box::new(self::vue::html_self_closing::HtmlSelfClosing),
        Box::new(self::vue::no_arrow_functions_in_watch::NoArrowFunctionsInWatch),
        Box::new(self::vue::no_async_in_computed_properties::NoAsyncInComputedProperties),
        Box::new(self::vue::no_child_content::NoChildContent),
        Box::new(self::vue::no_component_options_typo::NoComponentOptionsTypo),
        Box::new(self::vue::no_dupe_keys::NoDupeKeys),
        Box::new(self::vue::no_expose_after_await::NoExposeAfterAwait),
        Box::new(self::vue::no_lifecycle_after_await::NoLifecycleAfterAwait),
        Box::new(self::vue::no_ref_object_reactivity_loss::NoRefObjectReactivityLoss),
        Box::new(self::vue::no_reserved_component_names::NoReservedComponentNames),
        Box::new(self::vue::no_setup_props_reactivity_loss::NoSetupPropsReactivityLoss),
        Box::new(self::vue::no_watch_after_await::NoWatchAfterAwait),
        Box::new(self::vue::prefer_define_options::PreferDefineOptions),
        Box::new(self::vue::require_prop_comment::RequirePropComment),
    ]
}

/// Rules for the `jsdoc` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn jsdoc_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::jsdoc::check_access::CheckAccess),
        Box::new(self::jsdoc::check_param_names::CheckParamNames),
        Box::new(self::jsdoc::check_property_names::CheckPropertyNames),
        Box::new(self::jsdoc::check_tag_names::CheckTagNames),
        Box::new(self::jsdoc::check_types::CheckTypes),
        Box::new(self::jsdoc::check_values::CheckValues),
        Box::new(self::jsdoc::empty_tags::EmptyTags),
        Box::new(self::jsdoc::implements_on_classes::ImplementsOnClasses),
        Box::new(self::jsdoc::match_description::MatchDescription),
        Box::new(self::jsdoc::match_name::MatchName),
        Box::new(self::jsdoc::no_defaults::NoDefaults),
        Box::new(self::jsdoc::no_multi_asterisks::NoMultiAsterisks),
        Box::new(self::jsdoc::no_restricted_syntax::NoRestrictedSyntax),
        Box::new(self::jsdoc::require_description::RequireDescription),
        Box::new(self::jsdoc::require_param::RequireParam),
        Box::new(self::jsdoc::require_param_description::RequireParamDescription),
        Box::new(self::jsdoc::require_param_type::RequireParamType),
        Box::new(self::jsdoc::require_returns::RequireReturns),
    ]
}

/// Rules for the `storybook` plugin bundle.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn storybook_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(self::storybook::await_interactions::AwaitInteractions),
        Box::new(self::storybook::context_in_play_function::ContextInPlayFunction),
        Box::new(self::storybook::csf_component::CsfComponent),
        Box::new(self::storybook::default_exports::DefaultExports),
        Box::new(self::storybook::hierarchy_separator::HierarchySeparator),
        Box::new(self::storybook::meta_inline_properties::MetaInlineProperties),
        Box::new(self::storybook::meta_satisfies_type::MetaSatisfiesType),
        Box::new(self::storybook::no_redundant_story_name::NoRedundantStoryName),
        Box::new(self::storybook::no_stories_of::NoStoriesOf),
        Box::new(self::storybook::no_title_property_in_meta::NoTitlePropertyInMeta),
        Box::new(self::storybook::no_uninstalled_addons::NoUninstalledAddons),
        Box::new(self::storybook::prefer_pascal_case::PreferPascalCase),
        Box::new(self::storybook::story_exports::StoryExports),
        Box::new(self::storybook::use_storybook_expect::UseStorybookExpect),
        Box::new(self::storybook::use_storybook_testing_library::UseStorybookTestingLibrary),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_returns_builtin_rules() {
        let rules = all_lint_rules();
        assert!(
            rules.len() >= 2,
            "should have at least 2 built-in lint rules"
        );

        let names: Vec<String> = rules.iter().map(|r| r.meta().name).collect();
        assert!(
            names.contains(&"for-direction".to_owned()),
            "should contain for-direction"
        );
        assert!(
            names.contains(&"no-console".to_owned()),
            "should contain no-console"
        );
    }

    #[test]
    fn test_all_rule_metas() {
        let metas = all_rule_metas();
        assert!(metas.len() >= 2, "should have metadata for all rules");
    }

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("error"), Ok(Some(Severity::Error)));
        assert_eq!(parse_severity("warn"), Ok(Some(Severity::Warning)));
        assert_eq!(parse_severity("warning"), Ok(Some(Severity::Warning)));
        assert_eq!(parse_severity("off"), Ok(None));
    }

    #[test]
    fn test_parse_severity_rejects_unknown() {
        assert!(
            parse_severity("errror").is_err(),
            "typo should be rejected, not silently treated as error"
        );
        assert!(
            parse_severity("whatever").is_err(),
            "unknown severity should be rejected"
        );
    }

    #[test]
    fn test_all_rules_contain_prefixed_rules() {
        let rules = all_lint_rules();
        let names: Vec<String> = rules.iter().map(|r| r.meta().name).collect();
        assert!(
            names.iter().any(|n| n.starts_with("node/")),
            "rules should contain node/ prefixed rules"
        );
        assert!(
            names.contains(&"node/global-require".to_owned()),
            "rules should contain node/global-require"
        );
    }

    #[test]
    fn test_all_rule_metas_includes_prefixed() {
        let metas = all_rule_metas();
        assert!(
            metas.iter().any(|m| m.name.starts_with("node/")),
            "all_rule_metas should include node/ prefixed rules"
        );
    }

    #[test]
    fn test_native_plugin_registry_covers_all_rules() {
        let registry_count: usize = native_plugin_registry()
            .into_iter()
            .map(|np| (np.factory)().len())
            .sum();
        let all_count = all_lint_rules().len();
        assert_eq!(
            registry_count, all_count,
            "native plugin registry should cover all rules"
        );
    }
}
