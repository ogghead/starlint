//! Built-in native lint rules and rule registry.
//!
//! All rules are registered in [`all_rules`]. The [`rules_for_config`] function
//! filters and configures rules based on a rule config map.

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
pub mod typescript;
pub mod vitest;
pub mod vue;

use std::collections::{HashMap, HashSet};

use starlint_plugin_sdk::diagnostic::Severity;
use starlint_plugin_sdk::rule::RuleMeta;

use crate::rule::NativeRule;

/// Return all built-in native rules with their default configuration.
#[must_use]
#[allow(clippy::too_many_lines, clippy::large_stack_frames)]
pub fn all_rules() -> Vec<Box<dyn NativeRule>> {
    let mut rules: Vec<Box<dyn NativeRule>> = vec![
        Box::new(accessor_pairs::AccessorPairs),
        Box::new(approx_constant::ApproxConstant),
        Box::new(array_callback_return::ArrayCallbackReturn),
        Box::new(arrow_body_style::ArrowBodyStyle),
        Box::new(bad_array_method_on_arguments::BadArrayMethodOnArguments),
        Box::new(block_scoped_var::BlockScopedVar::new()),
        Box::new(bad_bitwise_operator::BadBitwiseOperator),
        Box::new(bad_char_at_comparison::BadCharAtComparison),
        Box::new(bad_comparison_sequence::BadComparisonSequence),
        Box::new(bad_min_max_func::BadMinMaxFunc),
        Box::new(bad_object_literal_comparison::BadObjectLiteralComparison),
        Box::new(bad_replace_all_arg::BadReplaceAllArg),
        Box::new(better_regex::BetterRegex),
        Box::new(branches_sharing_code::BranchesSharingCode),
        Box::new(capitalized_comments::CapitalizedComments),
        Box::new(catch_error_name::CatchErrorName::new()),
        Box::new(consistent_assert::ConsistentAssert),
        Box::new(consistent_date_clone::ConsistentDateClone),
        Box::new(consistent_destructuring::ConsistentDestructuring::new()),
        Box::new(consistent_empty_array_spread::ConsistentEmptyArraySpread),
        Box::new(consistent_existence_index_check::ConsistentExistenceIndexCheck),
        Box::new(consistent_function_scoping::ConsistentFunctionScoping::new()),
        Box::new(consistent_template_literal_escape::ConsistentTemplateLiteralEscape),
        Box::new(class_methods_use_this::ClassMethodsUseThis::new()),
        Box::new(const_comparisons::ConstComparisons),
        Box::new(constructor_super::ConstructorSuper),
        Box::new(curly::Curly),
        Box::new(default_case::DefaultCase),
        Box::new(default_case_last::DefaultCaseLast),
        Box::new(default_param_last::DefaultParamLast),
        Box::new(double_comparisons::DoubleComparisons),
        Box::new(empty_brace_spaces::EmptyBraceSpaces),
        Box::new(eqeqeq::Eqeqeq),
        Box::new(explicit_length_check::ExplicitLengthCheck),
        Box::new(erasing_op::ErasingOp),
        Box::new(error_message::ErrorMessage),
        Box::new(escape_case::EscapeCase),
        Box::new(expiring_todo_comments::ExpiringTodoComments),
        Box::new(filename_case::FilenameCase),
        Box::new(for_direction::ForDirection),
        Box::new(func_names::FuncNames),
        Box::new(func_style::FuncStyle),
        Box::new(getter_return::GetterReturn),
        Box::new(grouped_accessor_pairs::GroupedAccessorPairs),
        Box::new(guard_for_in::GuardForIn),
        Box::new(id_length::IdLength::new()),
        Box::new(init_declarations::InitDeclarations),
        Box::new(max_classes_per_file::MaxClassesPerFile::new()),
        Box::new(max_complexity::MaxComplexity::new()),
        Box::new(max_depth::MaxDepth::new()),
        Box::new(max_lines::MaxLines::new()),
        Box::new(max_lines_per_function::MaxLinesPerFunction::new()),
        Box::new(max_nested_callbacks::MaxNestedCallbacks::new()),
        Box::new(max_params::MaxParams::new()),
        Box::new(max_statements::MaxStatements::new()),
        Box::new(misrefactored_assign_op::MisrefactoredAssignOp),
        Box::new(missing_throw::MissingThrow),
        Box::new(new_cap::NewCap),
        Box::new(new_for_builtins::NewForBuiltins),
        Box::new(no_abusive_eslint_disable::NoAbusiveEslintDisable),
        Box::new(no_accessor_recursion::NoAccessorRecursion),
        Box::new(no_accumulating_spread::NoAccumulatingSpread),
        Box::new(no_alert::NoAlert),
        Box::new(no_anonymous_default_export::NoAnonymousDefaultExport),
        Box::new(no_array_reverse::NoArrayReverse),
        Box::new(no_array_callback_reference::NoArrayCallbackReference),
        Box::new(no_array_constructor::NoArrayConstructor),
        Box::new(no_array_for_each::NoArrayForEach),
        Box::new(no_array_method_this_argument::NoArrayMethodThisArgument),
        Box::new(no_array_push_push::NoArrayPushPush::new()),
        Box::new(no_array_reduce::NoArrayReduce),
        Box::new(no_array_sort::NoArraySort),
        Box::new(no_async_await::NoAsyncAwait),
        Box::new(no_async_endpoint_handlers::NoAsyncEndpointHandlers),
        Box::new(no_async_promise_executor::NoAsyncPromiseExecutor),
        Box::new(no_await_in_loop::NoAwaitInLoop::new()),
        Box::new(no_await_in_promise_methods::NoAwaitInPromiseMethods),
        Box::new(no_await_expression_member::NoAwaitExpressionMember),
        Box::new(no_barrel_file::NoBarrelFile),
        Box::new(no_bitwise::NoBitwise),
        Box::new(no_caller::NoCaller),
        Box::new(no_case_declarations::NoCaseDeclarations),
        Box::new(no_class_assign::NoClassAssign),
        Box::new(no_compare_neg_zero::NoCompareNegZero),
        Box::new(no_cond_assign::NoCondAssign),
        Box::new(no_console::NoConsole),
        Box::new(no_console_spaces::NoConsoleSpaces),
        Box::new(no_const_assign::NoConstAssign),
        Box::new(no_const_enum::NoConstEnum),
        Box::new(no_constant_binary_expression::NoConstantBinaryExpression),
        Box::new(no_constant_condition::NoConstantCondition),
        Box::new(no_constructor_return::NoConstructorReturn),
        Box::new(no_continue::NoContinue),
        Box::new(no_control_regex::NoControlRegex),
        Box::new(no_debugger::NoDebugger),
        Box::new(no_delete_var::NoDeleteVar),
        Box::new(no_div_regex::NoDivRegex),
        Box::new(no_document_cookie::NoDocumentCookie),
        Box::new(no_dupe_class_members::NoDupeClassMembers),
        Box::new(no_dupe_else_if::NoDupeElseIf),
        Box::new(no_dupe_keys::NoDupeKeys),
        Box::new(no_duplicate_case::NoDuplicateCase),
        Box::new(no_duplicate_imports::NoDuplicateImports),
        Box::new(no_else_return::NoElseReturn),
        Box::new(no_empty::NoEmpty),
        Box::new(no_empty_character_class::NoEmptyCharacterClass),
        Box::new(no_empty_file::NoEmptyFile),
        Box::new(no_empty_function::NoEmptyFunction),
        Box::new(no_empty_pattern::NoEmptyPattern),
        Box::new(no_empty_static_block::NoEmptyStaticBlock),
        Box::new(no_eq_null::NoEqNull),
        Box::new(no_eval::NoEval),
        Box::new(no_ex_assign::NoExAssign),
        Box::new(no_extend_native::NoExtendNative),
        Box::new(no_extra_bind::NoExtraBind),
        Box::new(no_extra_boolean_cast::NoExtraBooleanCast),
        Box::new(no_extra_label::NoExtraLabel),
        Box::new(no_extra_semi::NoExtraSemi),
        Box::new(no_fallthrough::NoFallthrough),
        Box::new(no_func_assign::NoFuncAssign),
        Box::new(no_global_assign::NoGlobalAssign),
        Box::new(no_hex_escape::NoHexEscape),
        Box::new(no_immediate_mutation::NoImmediateMutation),
        Box::new(no_implicit_coercion::NoImplicitCoercion),
        Box::new(no_import_assign::NoImportAssign),
        Box::new(no_inline_comments::NoInlineComments),
        Box::new(no_inner_declarations::NoInnerDeclarations),
        Box::new(no_instanceof_array::NoInstanceofArray),
        Box::new(no_instanceof_builtins::NoInstanceofBuiltins),
        Box::new(no_invalid_fetch_options::NoInvalidFetchOptions),
        Box::new(no_invalid_regexp::NoInvalidRegexp),
        Box::new(no_invalid_remove_event_listener::NoInvalidRemoveEventListener),
        Box::new(no_irregular_whitespace::NoIrregularWhitespace),
        Box::new(no_iterator::NoIterator),
        Box::new(no_keyword_prefix::NoKeywordPrefix),
        Box::new(no_label_var::NoLabelVar),
        Box::new(no_labels::NoLabels),
        Box::new(no_length_as_slice_end::NoLengthAsSliceEnd),
        Box::new(no_lone_blocks::NoLoneBlocks),
        Box::new(no_lonely_if::NoLonelyIf),
        Box::new(no_loop_func::NoLoopFunc),
        Box::new(no_magic_array_flat_depth::NoMagicArrayFlatDepth),
        Box::new(no_magic_numbers::NoMagicNumbers),
        Box::new(no_map_spread::NoMapSpread),
        Box::new(no_loss_of_precision::NoLossOfPrecision),
        Box::new(no_misleading_character_class::NoMisleadingCharacterClass),
        Box::new(no_multi_assign::NoMultiAssign),
        Box::new(no_multi_str::NoMultiStr),
        Box::new(no_negated_condition::NoNegatedCondition),
        Box::new(no_negation_in_equality_check::NoNegationInEqualityCheck),
        Box::new(no_nested_ternary::NoNestedTernary),
        Box::new(no_new::NoNew),
        Box::new(no_new_array::NoNewArray),
        Box::new(no_new_buffer::NoNewBuffer),
        Box::new(no_new_func::NoNewFunc),
        Box::new(no_new_native_nonconstructor::NoNewNativeNonconstructor),
        Box::new(no_new_wrappers::NoNewWrappers),
        Box::new(no_nonoctal_decimal_escape::NoNonoctalDecimalEscape),
        Box::new(no_null::NoNull),
        Box::new(no_obj_calls::NoObjCalls),
        Box::new(no_object_as_default_parameter::NoObjectAsDefaultParameter),
        Box::new(no_object_constructor::NoObjectConstructor),
        Box::new(no_optional_chaining::NoOptionalChaining),
        Box::new(no_param_reassign::NoParamReassign),
        Box::new(no_plusplus::NoPlusplus),
        Box::new(no_process_exit::NoProcessExit),
        Box::new(no_promise_executor_return::NoPromiseExecutorReturn),
        Box::new(no_proto::NoProto),
        Box::new(no_prototype_builtins::NoPrototypeBuiltins),
        Box::new(no_redeclare::NoRedeclare),
        Box::new(no_regex_spaces::NoRegexSpaces),
        Box::new(no_restricted_globals::NoRestrictedGlobals::new()),
        Box::new(no_restricted_imports::NoRestrictedImports::new()),
        Box::new(no_rest_spread_properties::NoRestSpreadProperties),
        Box::new(no_return_assign::NoReturnAssign),
        Box::new(no_script_url::NoScriptUrl),
        Box::new(no_self_assign::NoSelfAssign),
        Box::new(no_self_compare::NoSelfCompare),
        Box::new(no_sequences::NoSequences),
        Box::new(no_setter_return::NoSetterReturn),
        Box::new(no_shadow::NoShadow),
        Box::new(no_shadow_restricted_names::NoShadowRestrictedNames),
        Box::new(no_single_promise_in_promise_methods::NoSinglePromiseInPromiseMethods),
        Box::new(no_sparse_arrays::NoSparseArrays),
        Box::new(no_static_only_class::NoStaticOnlyClass),
        Box::new(no_template_curly_in_string::NoTemplateCurlyInString),
        Box::new(no_ternary::NoTernary),
        Box::new(no_thenable::NoThenable),
        Box::new(no_this_assignment::NoThisAssignment),
        Box::new(no_this_before_super::NoThisBeforeSuper),
        Box::new(no_this_in_exported_function::NoThisInExportedFunction),
        Box::new(no_throw_literal::NoThrowLiteral),
        Box::new(no_typeof_undefined::NoTypeofUndefined),
        Box::new(no_unassigned_vars::NoUnassignedVars),
        Box::new(no_undef::NoUndef),
        Box::new(no_undefined::NoUndefined),
        Box::new(no_unmodified_loop_condition::NoUnmodifiedLoopCondition),
        Box::new(no_unexpected_multiline::NoUnexpectedMultiline),
        Box::new(no_unnecessary_array_flat_depth::NoUnnecessaryArrayFlatDepth),
        Box::new(no_unnecessary_array_splice_count::NoUnnecessaryArraySpliceCount),
        Box::new(no_unnecessary_await::NoUnnecessaryAwait),
        Box::new(no_unnecessary_slice_end::NoUnnecessarySliceEnd),
        Box::new(no_unneeded_ternary::NoUnneededTernary),
        Box::new(no_unreachable::NoUnreachable),
        Box::new(no_unreadable_array_destructuring::NoUnreadableArrayDestructuring),
        Box::new(no_unreadable_iife::NoUnreadableIife),
        Box::new(no_unsafe_finally::NoUnsafeFinally),
        Box::new(no_unsafe_negation::NoUnsafeNegation),
        Box::new(no_unsafe_optional_chaining::NoUnsafeOptionalChaining),
        Box::new(no_unused_expressions::NoUnusedExpressions),
        Box::new(no_unused_labels::NoUnusedLabels),
        Box::new(no_unused_private_class_members::NoUnusedPrivateClassMembers),
        Box::new(no_unused_vars::NoUnusedVars),
        Box::new(no_use_before_define::NoUseBeforeDefine),
        Box::new(no_useless_backreference::NoUselessBackreference),
        Box::new(no_useless_call::NoUselessCall),
        Box::new(no_useless_catch::NoUselessCatch),
        Box::new(no_useless_collection_argument::NoUselessCollectionArgument),
        Box::new(no_useless_computed_key::NoUselessComputedKey),
        Box::new(no_useless_concat::NoUselessConcat),
        Box::new(no_useless_constructor::NoUselessConstructor),
        Box::new(no_useless_error_capture_stack_trace::NoUselessErrorCaptureStackTrace),
        Box::new(no_useless_escape::NoUselessEscape),
        Box::new(no_useless_fallback_in_spread::NoUselessFallbackInSpread),
        Box::new(no_useless_length_check::NoUselessLengthCheck),
        Box::new(no_useless_promise_resolve_reject::NoUselessPromiseResolveReject),
        Box::new(no_useless_rename::NoUselessRename),
        Box::new(no_useless_return::NoUselessReturn),
        Box::new(no_useless_spread::NoUselessSpread),
        Box::new(no_useless_switch_case::NoUselessSwitchCase),
        Box::new(no_useless_undefined::NoUselessUndefined),
        Box::new(no_var::NoVar),
        Box::new(no_void::NoVoid),
        Box::new(no_warning_comments::NoWarningComments),
        Box::new(no_with::NoWith),
        Box::new(no_zero_fractions::NoZeroFractions),
        Box::new(number_arg_out_of_range::NumberArgOutOfRange),
        Box::new(number_literal_case::NumberLiteralCase),
        Box::new(numeric_separators_style::NumericSeparatorsStyle),
        Box::new(only_used_in_recursion::OnlyUsedInRecursion),
        Box::new(operator_assignment::OperatorAssignment),
        Box::new(prefer_array_find::PreferArrayFind),
        Box::new(prefer_array_flat::PreferArrayFlat),
        Box::new(prefer_array_flat_map::PreferArrayFlatMap),
        Box::new(prefer_array_index_of::PreferArrayIndexOf),
        Box::new(prefer_array_some::PreferArraySome),
        Box::new(prefer_add_event_listener::PreferAddEventListener),
        Box::new(prefer_at::PreferAt),
        Box::new(prefer_bigint_literals::PreferBigintLiterals),
        Box::new(prefer_blob_reading_methods::PreferBlobReadingMethods),
        Box::new(prefer_class_fields::PreferClassFields),
        Box::new(prefer_classlist_toggle::PreferClasslistToggle),
        Box::new(prefer_code_point::PreferCodePoint),
        Box::new(prefer_const::PreferConst),
        Box::new(prefer_date_now::PreferDateNow),
        Box::new(prefer_default_parameters::PreferDefaultParameters),
        Box::new(prefer_destructuring::PreferDestructuring),
        Box::new(prefer_dom_node_append::PreferDomNodeAppend),
        Box::new(prefer_dom_node_dataset::PreferDomNodeDataset),
        Box::new(prefer_dom_node_remove::PreferDomNodeRemove),
        Box::new(prefer_dom_node_text_content::PreferDomNodeTextContent),
        Box::new(prefer_event_target::PreferEventTarget),
        Box::new(prefer_exponentiation_operator::PreferExponentiationOperator),
        Box::new(prefer_global_this::PreferGlobalThis),
        Box::new(prefer_includes::PreferIncludes),
        Box::new(prefer_keyboard_event_key::PreferKeyboardEventKey),
        Box::new(prefer_logical_operator_over_ternary::PreferLogicalOperatorOverTernary),
        Box::new(prefer_math_min_max::PreferMathMinMax),
        Box::new(prefer_math_trunc::PreferMathTrunc),
        Box::new(prefer_modern_dom_apis::PreferModernDomApis),
        Box::new(prefer_modern_math_apis::PreferModernMathApis),
        Box::new(prefer_module::PreferModule),
        Box::new(prefer_native_coercion_functions::PreferNativeCoercionFunctions),
        Box::new(prefer_negative_index::PreferNegativeIndex),
        Box::new(prefer_node_protocol::PreferNodeProtocol),
        Box::new(prefer_number_properties::PreferNumberProperties),
        Box::new(prefer_numeric_literals::PreferNumericLiterals),
        Box::new(prefer_object_from_entries::PreferObjectFromEntries),
        Box::new(prefer_object_has_own::PreferObjectHasOwn),
        Box::new(prefer_object_spread::PreferObjectSpread),
        Box::new(prefer_optional_catch_binding::PreferOptionalCatchBinding),
        Box::new(prefer_promise_reject_errors::PreferPromiseRejectErrors),
        Box::new(prefer_prototype_methods::PreferPrototypeMethods),
        Box::new(prefer_query_selector::PreferQuerySelector),
        Box::new(prefer_reflect_apply::PreferReflectApply),
        Box::new(prefer_regexp_test::PreferRegexpTest),
        Box::new(prefer_response_static_json::PreferResponseStaticJson),
        Box::new(prefer_rest_params::PreferRestParams),
        Box::new(prefer_set_has::PreferSetHas),
        Box::new(prefer_set_size::PreferSetSize),
        Box::new(prefer_spread::PreferSpread),
        Box::new(prefer_string_raw::PreferStringRaw),
        Box::new(prefer_string_replace_all::PreferStringReplaceAll),
        Box::new(prefer_string_slice::PreferStringSlice),
        Box::new(prefer_string_starts_ends_with::PreferStringStartsEndsWith),
        Box::new(prefer_string_trim_start_end::PreferStringTrimStartEnd),
        Box::new(prefer_structured_clone::PreferStructuredClone),
        Box::new(prefer_switch::PreferSwitch),
        Box::new(prefer_template::PreferTemplate),
        Box::new(prefer_ternary::PreferTernary),
        Box::new(prefer_top_level_await::PreferTopLevelAwait),
        Box::new(prefer_type_error::PreferTypeError),
        Box::new(preserve_caught_error::PreserveCaughtError),
        Box::new(prevent_abbreviations::PreventAbbreviations),
        Box::new(radix::Radix),
        Box::new(relative_url_style::RelativeUrlStyle),
        Box::new(require_array_join_separator::RequireArrayJoinSeparator),
        Box::new(require_await::RequireAwait),
        Box::new(require_module_attributes::RequireModuleAttributes),
        Box::new(require_module_specifiers::RequireModuleSpecifiers),
        Box::new(require_number_to_fixed_digits_argument::RequireNumberToFixedDigitsArgument),
        Box::new(require_post_message_target_origin::RequirePostMessageTargetOrigin),
        Box::new(require_yield::RequireYield),
        Box::new(sort_imports::SortImports),
        Box::new(sort_keys::SortKeys),
        Box::new(sort_vars::SortVars),
        Box::new(switch_case_braces::SwitchCaseBraces),
        Box::new(symbol_description::SymbolDescription),
        Box::new(text_encoding_identifier_case::TextEncodingIdentifierCase),
        Box::new(throw_new_error::ThrowNewError),
        Box::new(unicode_bom::UnicodeBom),
        Box::new(uninvoked_array_callback::UninvokedArrayCallback),
        Box::new(use_isnan::UseIsnan),
        Box::new(valid_typeof::ValidTypeof),
        Box::new(vars_on_top::VarsOnTop),
        Box::new(yoda::Yoda),
    ];

    // Append prefixed plugin-category rules.
    rules.extend(import::category_rules());
    rules.extend(jest::category_rules());
    rules.extend(jsdoc::category_rules());
    rules.extend(jsx_a11y::category_rules());
    rules.extend(nextjs::category_rules());
    rules.extend(node::category_rules());
    rules.extend(promise::category_rules());
    rules.extend(react::category_rules());
    rules.extend(react_perf::category_rules());
    rules.extend(typescript::category_rules());
    rules.extend(vitest::category_rules());
    rules.extend(vue::category_rules());

    rules
}

/// Return metadata for all built-in native rules.
#[must_use]
pub fn all_rule_metas() -> Vec<RuleMeta> {
    all_rules().iter().map(|r| r.meta()).collect()
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

/// Rules and their configured severity overrides.
pub struct ConfiguredRules {
    /// Enabled native rules.
    pub rules: Vec<Box<dyn NativeRule>>,
    /// Severity overrides from config (rule name → configured severity).
    pub severity_overrides: HashMap<String, Severity>,
    /// Rules loaded but disabled by default (only active via file-pattern overrides).
    pub disabled_rules: HashSet<String>,
}

/// Build a rule set from config, including rules referenced in overrides.
///
/// If `rule_configs` is empty, returns all rules with their default severity.
/// Otherwise, **only** enables rules that appear in `rule_configs` or
/// `override_configs` (unless "off" in base and not in overrides).
///
/// Rules referenced only in overrides are loaded but added to
/// [`ConfiguredRules::disabled_rules`] — their diagnostics are suppressed
/// by default and only activated for files matching an override pattern.
///
/// Configured severities are returned in [`ConfiguredRules::severity_overrides`]
/// so the engine can apply them to diagnostics.
#[must_use]
pub fn rules_for_config<S: ::std::hash::BuildHasher>(
    rule_configs: &HashMap<String, starlint_config::RuleConfig, S>,
    override_configs: &[starlint_config::Override],
) -> ConfiguredRules {
    // Collect rule names referenced in any override block.
    let override_rule_names: HashSet<String> = override_configs
        .iter()
        .flat_map(|ov| ov.rules.keys().cloned())
        .collect();

    if rule_configs.is_empty() {
        return ConfiguredRules {
            rules: all_rules(),
            severity_overrides: HashMap::new(),
            disabled_rules: HashSet::new(),
        };
    }

    let available = all_rules();
    let mut enabled: Vec<Box<dyn NativeRule>> = Vec::new();
    let mut severity_overrides: HashMap<String, Severity> = HashMap::new();
    let mut disabled_rules: HashSet<String> = HashSet::new();

    // Separate configs into exact matches and glob patterns (e.g. "typescript/*").
    let mut glob_configs: Vec<(&str, &starlint_config::RuleConfig)> = Vec::new();
    for (key, config) in rule_configs {
        if let Some(prefix) = key.strip_suffix("/*") {
            glob_configs.push((prefix, config));
        }
    }

    for mut rule in available {
        let meta = rule.meta();
        // Exact match takes priority; fall back to glob pattern.
        let in_base = rule_configs.get(&meta.name).or_else(|| {
            meta.name.split_once('/').and_then(|(prefix, _)| {
                glob_configs
                    .iter()
                    .find(|(p, _)| *p == prefix)
                    .map(|(_, config)| *config)
            })
        });
        let in_overrides = override_rule_names.contains(&meta.name);

        match in_base {
            Some(config) => match config {
                starlint_config::RuleConfig::Severity(sev) => match parse_severity(sev) {
                    Ok(Some(severity)) => {
                        if severity != meta.default_severity {
                            severity_overrides.insert(meta.name, severity);
                        }
                        enabled.push(rule);
                    }
                    Ok(None) => {
                        // "off" in base config — load only if referenced in overrides
                        if in_overrides {
                            disabled_rules.insert(meta.name);
                            enabled.push(rule);
                        }
                    }
                    Err(err) => {
                        tracing::warn!("rule `{}`: {err}", meta.name);
                    }
                },
                starlint_config::RuleConfig::Detailed(detailed) => {
                    match parse_severity(&detailed.severity) {
                        Ok(Some(severity)) => {
                            if severity != meta.default_severity {
                                severity_overrides.insert(meta.name.clone(), severity);
                            }
                            let options_value = serde_json::Value::Object(
                                detailed
                                    .options
                                    .iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect(),
                            );
                            if let Err(err) = rule.configure(&options_value) {
                                tracing::warn!("failed to configure rule `{}`: {err}", meta.name);
                            }
                            enabled.push(rule);
                        }
                        Ok(None) => {
                            // "off" in base config — load only if referenced in overrides
                            if in_overrides {
                                disabled_rules.insert(meta.name);
                                enabled.push(rule);
                            }
                        }
                        Err(err) => {
                            tracing::warn!("rule `{}`: {err}", meta.name);
                        }
                    }
                }
            },
            None => {
                // Not in base config — load as disabled if referenced in overrides
                if in_overrides {
                    disabled_rules.insert(meta.name);
                    enabled.push(rule);
                }
            }
        }
    }

    ConfiguredRules {
        rules: enabled,
        severity_overrides,
        disabled_rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_returns_builtin_rules() {
        let rules = all_rules();
        assert!(rules.len() >= 2, "should have at least 2 built-in rules");

        let names: Vec<String> = rules.iter().map(|r| r.meta().name).collect();
        assert!(
            names.contains(&"no-debugger".to_owned()),
            "should contain no-debugger"
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
    fn test_rules_for_empty_config() {
        let configs = HashMap::new();
        let configured = rules_for_config(&configs, &[]);
        assert!(
            configured.rules.len() >= 2,
            "empty config should return all default rules"
        );
        assert!(
            configured.severity_overrides.is_empty(),
            "empty config should have no severity overrides"
        );
    }

    #[test]
    fn test_rules_for_config_filters() {
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("error".to_owned()),
        );
        let configured = rules_for_config(&configs, &[]);
        assert_eq!(
            configured.rules.len(),
            1,
            "should only enable configured rules"
        );
        assert_eq!(
            configured.rules.first().map(|r| r.meta().name),
            Some("no-debugger".to_owned()),
            "should be no-debugger rule"
        );
    }

    #[test]
    fn test_rules_for_config_off() {
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("off".to_owned()),
        );
        let configured = rules_for_config(&configs, &[]);
        assert!(
            configured.rules.is_empty(),
            "rule set to 'off' should not be enabled"
        );
    }

    #[test]
    fn test_severity_override_applied() {
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("warn".to_owned()),
        );
        let configured = rules_for_config(&configs, &[]);
        assert_eq!(configured.rules.len(), 1, "should enable the rule");
        assert_eq!(
            configured.severity_overrides.get("no-debugger"),
            Some(&Severity::Warning),
            "no-debugger should be overridden to Warning"
        );
    }

    #[test]
    fn test_no_override_when_severity_matches_default() {
        let mut configs = HashMap::new();
        // no-debugger default is Error, so setting "error" should not create an override
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("error".to_owned()),
        );
        let configured = rules_for_config(&configs, &[]);
        assert!(
            configured.severity_overrides.is_empty(),
            "no override when severity matches default"
        );
    }

    #[test]
    fn test_empty_overrides_no_disabled_rules() {
        let configs = HashMap::new();
        let configured = rules_for_config(&configs, &[]);
        assert!(
            configured.disabled_rules.is_empty(),
            "empty overrides should produce no disabled rules"
        );
    }

    #[test]
    fn test_override_only_rule_loaded_as_disabled() {
        // Base config enables only no-debugger; override references no-console
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("error".to_owned()),
        );
        let overrides = vec![starlint_config::Override {
            files: vec!["**/*.test.ts".to_owned()],
            rules: std::iter::once((
                "no-console".to_owned(),
                starlint_config::RuleConfig::Severity("warn".to_owned()),
            ))
            .collect(),
        }];
        let configured = rules_for_config(&configs, &overrides);

        let names: Vec<String> = configured.rules.iter().map(|r| r.meta().name).collect();
        assert!(
            names.contains(&"no-debugger".to_owned()),
            "base rule should be loaded"
        );
        assert!(
            names.contains(&"no-console".to_owned()),
            "override-only rule should be loaded"
        );
        assert!(
            configured.disabled_rules.contains("no-console"),
            "override-only rule should be in disabled_rules"
        );
        assert!(
            !configured.disabled_rules.contains("no-debugger"),
            "base rule should not be disabled"
        );
    }

    #[test]
    fn test_off_rule_loaded_when_in_override() {
        // Base: no-debugger = "off", override references no-debugger
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("off".to_owned()),
        );
        let overrides = vec![starlint_config::Override {
            files: vec!["**/*.test.ts".to_owned()],
            rules: std::iter::once((
                "no-debugger".to_owned(),
                starlint_config::RuleConfig::Severity("error".to_owned()),
            ))
            .collect(),
        }];
        let configured = rules_for_config(&configs, &overrides);

        assert_eq!(
            configured.rules.len(),
            1,
            "off rule referenced in override should be loaded"
        );
        assert!(
            configured.disabled_rules.contains("no-debugger"),
            "off rule should be in disabled_rules"
        );
    }

    #[test]
    fn test_off_rule_skipped_when_not_in_override() {
        // Base: no-debugger = "off", no overrides reference it
        let mut configs = HashMap::new();
        configs.insert(
            "no-debugger".to_owned(),
            starlint_config::RuleConfig::Severity("off".to_owned()),
        );
        let configured = rules_for_config(&configs, &[]);
        assert!(
            configured.rules.is_empty(),
            "off rule with no override should be skipped"
        );
    }

    #[test]
    fn test_glob_config_enables_category() {
        // "node/*" = "error" should enable all node/ prefixed rules
        let mut configs = HashMap::new();
        configs.insert(
            "node/*".to_owned(),
            starlint_config::RuleConfig::Severity("error".to_owned()),
        );
        let configured = rules_for_config(&configs, &[]);
        let names: Vec<String> = configured.rules.iter().map(|r| r.meta().name).collect();
        assert!(
            names.iter().any(|n| n.starts_with("node/")),
            "glob config should enable node/ rules"
        );
        // Should NOT enable unprefixed rules
        assert!(
            !names.iter().any(|n| n == "no-debugger"),
            "glob config should not enable unrelated rules"
        );
    }

    #[test]
    fn test_exact_match_overrides_glob() {
        // "node/*" = "error" but "node/no-process-env" = "off"
        let mut configs = HashMap::new();
        configs.insert(
            "node/*".to_owned(),
            starlint_config::RuleConfig::Severity("error".to_owned()),
        );
        configs.insert(
            "node/no-process-env".to_owned(),
            starlint_config::RuleConfig::Severity("off".to_owned()),
        );
        let configured = rules_for_config(&configs, &[]);
        let names: Vec<String> = configured.rules.iter().map(|r| r.meta().name).collect();
        assert!(
            !names.contains(&"node/no-process-env".to_owned()),
            "exact 'off' should override glob 'error'"
        );
        assert!(
            names.contains(&"node/global-require".to_owned()),
            "other node rules should still be enabled"
        );
    }

    #[test]
    fn test_prefixed_rules_in_all_rules() {
        let rules = all_rules();
        assert!(
            rules.iter().any(|r| r.meta().name == "node/global-require"),
            "all_rules should include prefixed node rules"
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
}
