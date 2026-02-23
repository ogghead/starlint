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

use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

/// Create the testing plugin with all its rules.
#[must_use]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(LintRulePlugin::new(all_rules()))
}

/// Return all testing lint rules.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(crate::rules::jest::consistent_test_it::ConsistentTestIt),
        Box::new(crate::rules::jest::expect_expect::ExpectExpect),
        Box::new(crate::rules::jest::max_expects::MaxExpects::new()),
        Box::new(crate::rules::jest::max_nested_describe::MaxNestedDescribe),
        Box::new(crate::rules::jest::no_alias_methods::NoAliasMethods),
        Box::new(crate::rules::jest::no_commented_out_tests::NoCommentedOutTests),
        Box::new(crate::rules::jest::no_conditional_expect::NoConditionalExpect),
        Box::new(crate::rules::jest::no_conditional_in_test::NoConditionalInTest),
        Box::new(crate::rules::jest::no_confusing_set_timeout::NoConfusingSetTimeout),
        Box::new(crate::rules::jest::no_deprecated_functions::NoDeprecatedFunctions),
        Box::new(crate::rules::jest::no_disabled_tests::NoDisabledTests),
        Box::new(crate::rules::jest::no_done_callback::NoDoneCallback),
        Box::new(crate::rules::jest::no_duplicate_hooks::NoDuplicateHooks),
        Box::new(crate::rules::jest::no_export::NoExport),
        Box::new(crate::rules::jest::no_focused_tests::NoFocusedTests),
        Box::new(crate::rules::jest::no_hooks::NoHooks),
        Box::new(crate::rules::jest::no_identical_title::NoIdenticalTitle),
        Box::new(crate::rules::jest::no_interpolation_in_snapshots::NoInterpolationInSnapshots),
        Box::new(crate::rules::jest::no_jasmine_globals::NoJasmineGlobals),
        Box::new(crate::rules::jest::no_large_snapshots::NoLargeSnapshots),
        Box::new(crate::rules::jest::no_mocks_import::NoMocksImport),
        Box::new(crate::rules::jest::no_restricted_jest_methods::NoRestrictedJestMethods),
        Box::new(crate::rules::jest::no_restricted_matchers::NoRestrictedMatchers),
        Box::new(crate::rules::jest::no_standalone_expect::NoStandaloneExpect),
        Box::new(crate::rules::jest::no_test_prefixes::NoTestPrefixes),
        Box::new(crate::rules::jest::no_test_return_statement::NoTestReturnStatement),
        Box::new(crate::rules::jest::no_unneeded_async_expect_function::NoUnneededAsyncExpectFunction),
        Box::new(crate::rules::jest::no_untyped_mock_factory::NoUntypedMockFactory),
        Box::new(crate::rules::jest::padding_around_test_blocks::PaddingAroundTestBlocks),
        Box::new(crate::rules::jest::prefer_called_with::PreferCalledWith),
        Box::new(crate::rules::jest::prefer_comparison_matcher::PreferComparisonMatcher),
        Box::new(crate::rules::jest::prefer_each::PreferEach),
        Box::new(crate::rules::jest::prefer_equality_matcher::PreferEqualityMatcher),
        Box::new(crate::rules::jest::prefer_expect_resolves::PreferExpectResolves),
        Box::new(crate::rules::jest::prefer_hooks_in_order::PreferHooksInOrder),
        Box::new(crate::rules::jest::prefer_hooks_on_top::PreferHooksOnTop),
        Box::new(crate::rules::jest::prefer_jest_mocked::PreferJestMocked),
        Box::new(crate::rules::jest::prefer_lowercase_title::PreferLowercaseTitle),
        Box::new(crate::rules::jest::prefer_mock_promise_shorthand::PreferMockPromiseShorthand),
        Box::new(crate::rules::jest::prefer_mock_return_shorthand::PreferMockReturnShorthand),
        Box::new(crate::rules::jest::prefer_spy_on::PreferSpyOn),
        Box::new(crate::rules::jest::prefer_strict_equal::PreferStrictEqual),
        Box::new(crate::rules::jest::prefer_to_be::PreferToBe),
        Box::new(crate::rules::jest::prefer_to_contain::PreferToContain),
        Box::new(crate::rules::jest::prefer_to_have_been_called::PreferToHaveBeenCalled),
        Box::new(crate::rules::jest::prefer_to_have_been_called_times::PreferToHaveBeenCalledTimes),
        Box::new(crate::rules::jest::prefer_to_have_length::PreferToHaveLength),
        Box::new(crate::rules::jest::prefer_todo::PreferTodo),
        Box::new(crate::rules::jest::require_hook::RequireHook),
        Box::new(crate::rules::jest::require_to_throw_message::RequireToThrowMessage),
        Box::new(crate::rules::jest::require_top_level_describe::RequireTopLevelDescribe),
        Box::new(crate::rules::jest::valid_describe_callback::ValidDescribeCallback),
        Box::new(crate::rules::jest::valid_expect::ValidExpect),
        Box::new(crate::rules::jest::valid_title::ValidTitle),
        Box::new(crate::rules::vitest::consistent_each_for::ConsistentEachFor),
        Box::new(crate::rules::vitest::consistent_test_filename::ConsistentTestFilename),
        Box::new(crate::rules::vitest::consistent_vitest_vi::ConsistentVitestVi),
        Box::new(crate::rules::vitest::hoisted_apis_on_top::HoistedApisOnTop),
        Box::new(crate::rules::vitest::no_conditional_tests::NoConditionalTests),
        Box::new(crate::rules::vitest::no_import_node_test::NoImportNodeTest),
        Box::new(crate::rules::vitest::no_importing_vitest_globals::NoImportingVitestGlobals),
        Box::new(crate::rules::vitest::prefer_called_once::PreferCalledOnce),
        Box::new(crate::rules::vitest::prefer_called_times::PreferCalledTimes),
        Box::new(crate::rules::vitest::prefer_describe_function_title::PreferDescribeFunctionTitle),
        Box::new(crate::rules::vitest::prefer_expect_type_of::PreferExpectTypeOf),
        Box::new(crate::rules::vitest::prefer_import_in_mock::PreferImportInMock),
        Box::new(crate::rules::vitest::prefer_to_be_falsy::PreferToBeFalsy),
        Box::new(crate::rules::vitest::prefer_to_be_object::PreferToBeObject),
        Box::new(crate::rules::vitest::prefer_to_be_truthy::PreferToBeTruthy),
        Box::new(crate::rules::vitest::require_local_test_context_for_concurrent_snapshots::RequireLocalTestContextForConcurrentSnapshots),
        Box::new(crate::rules::vitest::warn_todo::WarnTodo),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_plugin_returns_rules() {
        let plugin = create_plugin();
        let rules = plugin.rules();
        assert!(
            !rules.is_empty(),
            "testing plugin should provide at least one rule"
        );
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 71, "testing should have 71 rules");
    }
}
