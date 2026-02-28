//! TypeScript-specific lint rules.
//!
//! Rules are prefixed with `typescript/` in config and output.

pub mod adjacent_overload_signatures;
pub mod array_type;
pub mod await_thenable;
pub mod ban_ts_comment;
pub mod ban_tslint_comment;
pub mod ban_types;
pub mod consistent_generic_constructors;
pub mod consistent_indexed_object_style;
pub mod consistent_return;
pub mod consistent_type_assertions;
pub mod consistent_type_definitions;
pub mod consistent_type_exports;
pub mod consistent_type_imports;
pub mod dot_notation;
pub mod explicit_function_return_type;
pub mod explicit_module_boundary_types;
pub mod no_array_delete;
pub mod no_base_to_string;
pub mod no_confusing_non_null_assertion;
pub mod no_confusing_void_expression;
pub mod no_deprecated;
pub mod no_duplicate_enum_values;
pub mod no_dynamic_delete;
pub mod no_empty_interface;
pub mod no_empty_object_type;
pub mod no_explicit_any;
pub mod no_extra_non_null_assertion;
pub mod no_extraneous_class;
pub mod no_floating_promises;
pub mod no_for_in_array;
pub mod no_implied_eval;
pub mod no_inferrable_types;
pub mod no_invalid_void_type;
pub mod no_misused_new;
pub mod no_misused_promises;
pub mod no_misused_spread;
pub mod no_mixed_enums;
pub mod no_namespace;
pub mod no_non_null_asserted_optional_chain;
pub mod no_non_null_assertion;
pub mod no_require_imports;
pub mod no_restricted_types;
pub mod no_this_alias;
pub mod no_unnecessary_boolean_literal_compare;
pub mod no_unnecessary_parameter_property_assignment;
pub mod no_unnecessary_qualifier;
pub mod no_unnecessary_template_expression;
pub mod no_unnecessary_condition;
pub mod no_unnecessary_type_arguments;
pub mod no_unnecessary_type_assertion;
pub mod no_unnecessary_type_constraint;
pub mod no_unnecessary_type_parameters;
pub mod no_unsafe_argument;
pub mod no_unsafe_assignment;
pub mod no_unsafe_call;
pub mod no_unsafe_declaration_merging;
pub mod no_unsafe_enum_comparison;
pub mod no_unsafe_function_type;
pub mod no_unsafe_member_access;
pub mod no_unsafe_return;
pub mod no_unsafe_type_assertion;
pub mod no_unsafe_unary_minus;
pub mod no_useless_empty_export;
pub mod no_var_requires;
pub mod no_wrapper_object_types;
pub mod non_nullable_type_assertion_style;
pub mod only_throw_error;
pub mod parameter_properties;
pub mod prefer_as_const;
pub mod prefer_enum_initializers;
pub mod prefer_find;
pub mod prefer_for_of;
pub mod prefer_function_type;
pub mod prefer_includes;
pub mod prefer_literal_enum_member;
pub mod prefer_namespace_keyword;
pub mod prefer_nullish_coalescing;
pub mod prefer_optional_chain;
pub mod prefer_promise_reject_errors;
pub mod prefer_readonly;
pub mod prefer_reduce_type_parameter;
pub mod prefer_regexp_exec;
pub mod prefer_readonly_parameter_types;
pub mod prefer_return_this_type;
pub mod prefer_string_starts_ends_with;
pub mod promise_function_async;
pub mod restrict_plus_operands;
pub mod return_await;
pub mod strict_boolean_expressions;
pub mod strict_void_return;
pub mod switch_exhaustiveness_check;
pub mod unbound_method;
pub mod related_getter_setter_pairs;
pub mod require_array_sort_compare;
pub mod require_await;
pub mod restrict_template_expressions;
pub mod triple_slash_reference;
pub mod unified_signatures;
pub mod use_unknown_in_catch_callback_variable;

use crate::rule::NativeRule;

/// Return all TypeScript rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(adjacent_overload_signatures::AdjacentOverloadSignatures),
        Box::new(array_type::ArrayType),
        Box::new(await_thenable::AwaitThenable),
        Box::new(ban_ts_comment::BanTsComment),
        Box::new(ban_tslint_comment::BanTslintComment),
        Box::new(ban_types::BanTypes),
        Box::new(consistent_generic_constructors::ConsistentGenericConstructors),
        Box::new(consistent_indexed_object_style::ConsistentIndexedObjectStyle),
        Box::new(consistent_return::ConsistentReturn),
        Box::new(consistent_type_assertions::ConsistentTypeAssertions),
        Box::new(consistent_type_definitions::ConsistentTypeDefinitions),
        Box::new(consistent_type_exports::ConsistentTypeExports),
        Box::new(consistent_type_imports::ConsistentTypeImports),
        Box::new(dot_notation::DotNotation),
        Box::new(explicit_function_return_type::ExplicitFunctionReturnType),
        Box::new(explicit_module_boundary_types::ExplicitModuleBoundaryTypes),
        Box::new(no_array_delete::NoArrayDelete),
        Box::new(no_base_to_string::NoBaseToString),
        Box::new(no_confusing_non_null_assertion::NoConfusingNonNullAssertion),
        Box::new(no_confusing_void_expression::NoConfusingVoidExpression),
        Box::new(no_deprecated::NoDeprecated),
        Box::new(no_duplicate_enum_values::NoDuplicateEnumValues),
        Box::new(no_dynamic_delete::NoDynamicDelete),
        Box::new(no_empty_interface::NoEmptyInterface),
        Box::new(no_empty_object_type::NoEmptyObjectType),
        Box::new(no_explicit_any::NoExplicitAny),
        Box::new(no_extra_non_null_assertion::NoExtraNonNullAssertion),
        Box::new(no_extraneous_class::NoExtraneousClass),
        Box::new(no_floating_promises::NoFloatingPromises),
        Box::new(no_for_in_array::NoForInArray),
        Box::new(no_implied_eval::NoImpliedEval),
        Box::new(no_inferrable_types::NoInferrableTypes),
        Box::new(no_invalid_void_type::NoInvalidVoidType::new()),
        Box::new(no_misused_new::NoMisusedNew),
        Box::new(no_misused_promises::NoMisusedPromises),
        Box::new(no_misused_spread::NoMisusedSpread),
        Box::new(no_mixed_enums::NoMixedEnums),
        Box::new(no_namespace::NoNamespace),
        Box::new(no_non_null_asserted_optional_chain::NoNonNullAssertedOptionalChain),
        Box::new(no_non_null_assertion::NoNonNullAssertion),
        Box::new(no_require_imports::NoRequireImports),
        Box::new(no_restricted_types::NoRestrictedTypes),
        Box::new(no_this_alias::NoThisAlias),
        Box::new(no_unnecessary_boolean_literal_compare::NoUnnecessaryBooleanLiteralCompare),
        Box::new(no_unnecessary_parameter_property_assignment::NoUnnecessaryParameterPropertyAssignment),
        Box::new(no_unnecessary_qualifier::NoUnnecessaryQualifier),
        Box::new(no_unnecessary_template_expression::NoUnnecessaryTemplateExpression),
        Box::new(no_unnecessary_condition::NoUnnecessaryCondition),
        Box::new(no_unnecessary_type_arguments::NoUnnecessaryTypeArguments),
        Box::new(no_unnecessary_type_assertion::NoUnnecessaryTypeAssertion),
        Box::new(no_unnecessary_type_constraint::NoUnnecessaryTypeConstraint),
        Box::new(no_unnecessary_type_parameters::NoUnnecessaryTypeParameters),
        Box::new(no_unsafe_argument::NoUnsafeArgument),
        Box::new(no_unsafe_assignment::NoUnsafeAssignment),
        Box::new(no_unsafe_call::NoUnsafeCall),
        Box::new(no_unsafe_declaration_merging::NoUnsafeDeclarationMerging),
        Box::new(no_unsafe_enum_comparison::NoUnsafeEnumComparison),
        Box::new(no_unsafe_function_type::NoUnsafeFunctionType),
        Box::new(no_unsafe_member_access::NoUnsafeMemberAccess),
        Box::new(no_unsafe_return::NoUnsafeReturn),
        Box::new(no_unsafe_type_assertion::NoUnsafeTypeAssertion),
        Box::new(no_unsafe_unary_minus::NoUnsafeUnaryMinus),
        Box::new(no_useless_empty_export::NoUselessEmptyExport),
        Box::new(no_var_requires::NoVarRequires),
        Box::new(no_wrapper_object_types::NoWrapperObjectTypes),
        Box::new(non_nullable_type_assertion_style::NonNullableTypeAssertionStyle),
        Box::new(only_throw_error::OnlyThrowError),
        Box::new(parameter_properties::ParameterProperties),
        Box::new(prefer_as_const::PreferAsConst),
        Box::new(prefer_enum_initializers::PreferEnumInitializers),
        Box::new(prefer_find::PreferFind),
        Box::new(prefer_for_of::PreferForOf),
        Box::new(prefer_function_type::PreferFunctionType),
        Box::new(prefer_includes::PreferIncludes),
        Box::new(prefer_literal_enum_member::PreferLiteralEnumMember),
        Box::new(prefer_namespace_keyword::PreferNamespaceKeyword),
        Box::new(prefer_nullish_coalescing::PreferNullishCoalescing),
        Box::new(prefer_optional_chain::PreferOptionalChain),
        Box::new(prefer_promise_reject_errors::PreferPromiseRejectErrors),
        Box::new(prefer_readonly::PreferReadonly),
        Box::new(prefer_readonly_parameter_types::PreferReadonlyParameterTypes),
        Box::new(prefer_reduce_type_parameter::PreferReduceTypeParameter),
        Box::new(prefer_regexp_exec::PreferRegexpExec),
        Box::new(prefer_return_this_type::PreferReturnThisType),
        Box::new(prefer_string_starts_ends_with::PreferStringStartsEndsWith),
        Box::new(promise_function_async::PromiseFunctionAsync),
        Box::new(restrict_plus_operands::RestrictPlusOperands),
        Box::new(related_getter_setter_pairs::RelatedGetterSetterPairs),
        Box::new(require_array_sort_compare::RequireArraySortCompare),
        Box::new(require_await::RequireAwait),
        Box::new(restrict_template_expressions::RestrictTemplateExpressions),
        Box::new(return_await::ReturnAwait),
        Box::new(strict_boolean_expressions::StrictBooleanExpressions),
        Box::new(strict_void_return::StrictVoidReturn),
        Box::new(switch_exhaustiveness_check::SwitchExhaustivenessCheck),
        Box::new(triple_slash_reference::TripleSlashReference),
        Box::new(unbound_method::UnboundMethod),
        Box::new(unified_signatures::UnifiedSignatures),
        Box::new(use_unknown_in_catch_callback_variable::UseUnknownInCatchCallbackVariable),
    ]
}
