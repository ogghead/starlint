//! Jest-specific lint rules.
//!
//! Rules are prefixed with `jest/` in config and output.

pub mod consistent_test_it;
pub mod expect_expect;
pub mod max_expects;
pub mod max_nested_describe;
pub mod no_alias_methods;
pub mod no_commented_out_tests;
pub mod no_conditional_expect;
pub mod no_conditional_in_test;
pub mod no_confusing_set_timeout;
pub mod no_deprecated_functions;
pub mod no_disabled_tests;
pub mod no_done_callback;
pub mod no_duplicate_hooks;
pub mod no_export;
pub mod no_focused_tests;
pub mod no_hooks;
pub mod no_identical_title;
pub mod no_interpolation_in_snapshots;
pub mod no_jasmine_globals;
pub mod no_large_snapshots;
pub mod no_mocks_import;
pub mod no_restricted_jest_methods;
pub mod no_restricted_matchers;
pub mod no_standalone_expect;
pub mod no_test_prefixes;
pub mod no_test_return_statement;
pub mod no_unneeded_async_expect_function;
pub mod no_untyped_mock_factory;
pub mod padding_around_test_blocks;
pub mod prefer_called_with;
pub mod prefer_comparison_matcher;
pub mod prefer_each;
pub mod prefer_equality_matcher;
pub mod prefer_expect_resolves;
pub mod prefer_hooks_in_order;
pub mod prefer_hooks_on_top;
pub mod prefer_jest_mocked;
pub mod prefer_lowercase_title;
pub mod prefer_mock_promise_shorthand;
pub mod prefer_mock_return_shorthand;
pub mod prefer_spy_on;
pub mod prefer_strict_equal;
pub mod prefer_to_be;
pub mod prefer_to_contain;
pub mod prefer_to_have_been_called;
pub mod prefer_to_have_been_called_times;
pub mod prefer_to_have_length;
pub mod prefer_todo;
pub mod require_hook;
pub mod require_to_throw_message;
pub mod require_top_level_describe;
pub mod valid_describe_callback;
pub mod valid_expect;
pub mod valid_title;

use crate::rule::NativeRule;

/// Return all Jest rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(expect_expect::ExpectExpect),
        Box::new(no_alias_methods::NoAliasMethods),
        Box::new(no_conditional_expect::NoConditionalExpect),
        Box::new(no_deprecated_functions::NoDeprecatedFunctions),
        Box::new(no_disabled_tests::NoDisabledTests),
        Box::new(no_done_callback::NoDoneCallback),
        Box::new(no_focused_tests::NoFocusedTests),
        Box::new(no_hooks::NoHooks),
        Box::new(no_interpolation_in_snapshots::NoInterpolationInSnapshots),
        Box::new(no_jasmine_globals::NoJasmineGlobals),
        Box::new(no_large_snapshots::NoLargeSnapshots),        Box::new(no_restricted_jest_methods::NoRestrictedJestMethods),
        Box::new(no_restricted_matchers::NoRestrictedMatchers),
        Box::new(no_standalone_expect::NoStandaloneExpect),
        Box::new(no_test_prefixes::NoTestPrefixes),
        Box::new(no_untyped_mock_factory::NoUntypedMockFactory),
        Box::new(prefer_called_with::PreferCalledWith),
        Box::new(prefer_comparison_matcher::PreferComparisonMatcher),
        Box::new(prefer_each::PreferEach),
        Box::new(prefer_equality_matcher::PreferEqualityMatcher),
        Box::new(prefer_expect_resolves::PreferExpectResolves),
        Box::new(prefer_hooks_in_order::PreferHooksInOrder),
        Box::new(prefer_hooks_on_top::PreferHooksOnTop),
        Box::new(prefer_lowercase_title::PreferLowercaseTitle),
        Box::new(prefer_mock_promise_shorthand::PreferMockPromiseShorthand),
        Box::new(prefer_mock_return_shorthand::PreferMockReturnShorthand),
        Box::new(prefer_spy_on::PreferSpyOn),
        Box::new(prefer_strict_equal::PreferStrictEqual),
        Box::new(prefer_to_be::PreferToBe),
        Box::new(prefer_to_contain::PreferToContain),
        Box::new(prefer_to_have_been_called::PreferToHaveBeenCalled),
        Box::new(prefer_to_have_been_called_times::PreferToHaveBeenCalledTimes),
        Box::new(prefer_to_have_length::PreferToHaveLength),
        Box::new(prefer_todo::PreferTodo),
        Box::new(require_to_throw_message::RequireToThrowMessage),
        Box::new(valid_describe_callback::ValidDescribeCallback),
        Box::new(valid_expect::ValidExpect),
        Box::new(valid_title::ValidTitle),
    ]
}
