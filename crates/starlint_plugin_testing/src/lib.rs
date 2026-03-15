//! Testing lint rules for starlint.
//!
//! Provides [`create_plugin`] to construct a [`Plugin`] containing all
//! testing rules.

pub mod rules;

/// Check whether a file path looks like a test file based on naming conventions.
///
/// Matches common patterns: `*.test.*`, `*.spec.*`, `__tests__/`, `__test__/`,
/// `test.js`, `spec.ts`, etc.
pub(crate) fn is_test_file(file_path: &std::path::Path) -> bool {
    let path_str = file_path.to_string_lossy();
    path_str.contains(".test.")
        || path_str.contains(".spec.")
        || path_str.contains("__tests__")
        || path_str.contains("__test__")
        || path_str.ends_with(".test")
        || path_str.ends_with(".spec")
        || file_path.file_stem().is_some_and(|stem| {
            let name = stem.to_string_lossy();
            name == "test"
                || name == "spec"
                || name.starts_with("test_")
                || name.starts_with("spec_")
        })
}

starlint_rule_framework::declare_plugin! {
    name: "testing",
    rules: [
        crate::rules::jest::consistent_test_it::ConsistentTestIt,
        crate::rules::jest::expect_expect::ExpectExpect,
        crate::rules::jest::max_expects::MaxExpects::new(),
        crate::rules::jest::max_nested_describe::MaxNestedDescribe,
        crate::rules::jest::no_alias_methods::NoAliasMethods,
        crate::rules::jest::no_commented_out_tests::NoCommentedOutTests,
        crate::rules::jest::no_conditional_expect::NoConditionalExpect,
        crate::rules::jest::no_conditional_in_test::NoConditionalInTest,
        crate::rules::jest::no_confusing_set_timeout::NoConfusingSetTimeout,
        crate::rules::jest::no_deprecated_functions::NoDeprecatedFunctions,
        crate::rules::jest::no_disabled_tests::NoDisabledTests,
        crate::rules::jest::no_done_callback::NoDoneCallback,
        crate::rules::jest::no_duplicate_hooks::NoDuplicateHooks,
        crate::rules::jest::no_export::NoExport,
        crate::rules::jest::no_focused_tests::NoFocusedTests,
        crate::rules::jest::no_hooks::NoHooks,
        crate::rules::jest::no_identical_title::NoIdenticalTitle,
        crate::rules::jest::no_interpolation_in_snapshots::NoInterpolationInSnapshots,
        crate::rules::jest::no_jasmine_globals::NoJasmineGlobals,
        crate::rules::jest::no_large_snapshots::NoLargeSnapshots,
        crate::rules::jest::no_mocks_import::NoMocksImport,
        crate::rules::jest::no_restricted_jest_methods::NoRestrictedJestMethods,
        crate::rules::jest::no_restricted_matchers::NoRestrictedMatchers,
        crate::rules::jest::no_standalone_expect::NoStandaloneExpect,
        crate::rules::jest::no_test_prefixes::NoTestPrefixes,
        crate::rules::jest::no_test_return_statement::NoTestReturnStatement,
        crate::rules::jest::no_unneeded_async_expect_function::NoUnneededAsyncExpectFunction,
        crate::rules::jest::no_untyped_mock_factory::NoUntypedMockFactory,
        crate::rules::jest::padding_around_test_blocks::PaddingAroundTestBlocks,
        crate::rules::jest::prefer_called_with::PreferCalledWith,
        crate::rules::jest::prefer_comparison_matcher::PreferComparisonMatcher,
        crate::rules::jest::prefer_each::PreferEach,
        crate::rules::jest::prefer_equality_matcher::PreferEqualityMatcher,
        crate::rules::jest::prefer_expect_resolves::PreferExpectResolves,
        crate::rules::jest::prefer_hooks_in_order::PreferHooksInOrder,
        crate::rules::jest::prefer_hooks_on_top::PreferHooksOnTop,
        crate::rules::jest::prefer_jest_mocked::PreferJestMocked,
        crate::rules::jest::prefer_lowercase_title::PreferLowercaseTitle,
        crate::rules::jest::prefer_mock_promise_shorthand::PreferMockPromiseShorthand,
        crate::rules::jest::prefer_mock_return_shorthand::PreferMockReturnShorthand,
        crate::rules::jest::prefer_spy_on::PreferSpyOn,
        crate::rules::jest::prefer_strict_equal::PreferStrictEqual,
        crate::rules::jest::prefer_to_be::PreferToBe,
        crate::rules::jest::prefer_to_contain::PreferToContain,
        crate::rules::jest::prefer_to_have_been_called::PreferToHaveBeenCalled,
        crate::rules::jest::prefer_to_have_been_called_times::PreferToHaveBeenCalledTimes,
        crate::rules::jest::prefer_to_have_length::PreferToHaveLength,
        crate::rules::jest::prefer_todo::PreferTodo,
        crate::rules::jest::require_hook::RequireHook,
        crate::rules::jest::require_to_throw_message::RequireToThrowMessage,
        crate::rules::jest::require_top_level_describe::RequireTopLevelDescribe,
        crate::rules::jest::valid_describe_callback::ValidDescribeCallback,
        crate::rules::jest::valid_expect::ValidExpect,
        crate::rules::jest::valid_title::ValidTitle,
        crate::rules::vitest::consistent_each_for::ConsistentEachFor,
        crate::rules::vitest::consistent_test_filename::ConsistentTestFilename,
        crate::rules::vitest::consistent_vitest_vi::ConsistentVitestVi,
        crate::rules::vitest::hoisted_apis_on_top::HoistedApisOnTop,
        crate::rules::vitest::no_conditional_tests::NoConditionalTests,
        crate::rules::vitest::no_import_node_test::NoImportNodeTest,
        crate::rules::vitest::no_importing_vitest_globals::NoImportingVitestGlobals,
        crate::rules::vitest::prefer_called_once::PreferCalledOnce,
        crate::rules::vitest::prefer_called_times::PreferCalledTimes,
        crate::rules::vitest::prefer_describe_function_title::PreferDescribeFunctionTitle,
        crate::rules::vitest::prefer_expect_type_of::PreferExpectTypeOf,
        crate::rules::vitest::prefer_import_in_mock::PreferImportInMock,
        crate::rules::vitest::prefer_to_be_falsy::PreferToBeFalsy,
        crate::rules::vitest::prefer_to_be_object::PreferToBeObject,
        crate::rules::vitest::prefer_to_be_truthy::PreferToBeTruthy,
        crate::rules::vitest::require_local_test_context_for_concurrent_snapshots::RequireLocalTestContextForConcurrentSnapshots,
        crate::rules::vitest::warn_todo::WarnTodo,
    ]
}
