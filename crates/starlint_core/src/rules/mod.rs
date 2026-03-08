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
            factory: crate::lint_rules::core_rules,
        },
        NativePlugin {
            name: "react",
            factory: || {
                let mut rules = crate::lint_rules::react_rules();
                rules.extend(crate::lint_rules::jsx_a11y_rules());
                rules.extend(crate::lint_rules::react_perf_rules());
                rules
            },
        },
        NativePlugin {
            name: "typescript",
            factory: crate::lint_rules::typescript_rules,
        },
        NativePlugin {
            name: "testing",
            factory: || {
                let mut rules = crate::lint_rules::jest_rules();
                rules.extend(crate::lint_rules::vitest_rules());
                rules
            },
        },
        NativePlugin {
            name: "modules",
            factory: || {
                let mut rules = crate::lint_rules::import_rules();
                rules.extend(crate::lint_rules::node_rules());
                rules.extend(crate::lint_rules::promise_rules());
                rules
            },
        },
        NativePlugin {
            name: "nextjs",
            factory: crate::lint_rules::nextjs_rules,
        },
        NativePlugin {
            name: "vue",
            factory: crate::lint_rules::vue_rules,
        },
        NativePlugin {
            name: "jsdoc",
            factory: crate::lint_rules::jsdoc_rules,
        },
        NativePlugin {
            name: "storybook",
            factory: crate::lint_rules::storybook_rules,
        },
    ]
}

/// Return metadata for all built-in rules.
#[must_use]
pub fn all_rule_metas() -> Vec<RuleMeta> {
    crate::lint_rules::all_lint_rules()
        .iter()
        .map(|r| r.meta())
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_returns_builtin_rules() {
        let rules = crate::lint_rules::all_lint_rules();
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
        let rules = crate::lint_rules::all_lint_rules();
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
        let all_count = crate::lint_rules::all_lint_rules().len();
        assert_eq!(
            registry_count, all_count,
            "native plugin registry should cover all rules"
        );
    }
}
