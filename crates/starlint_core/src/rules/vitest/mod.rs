//! Vitest-specific lint rules.
//!
//! Rules are prefixed with `vitest/` in config and output.

pub mod consistent_each_for;
pub mod consistent_test_filename;
pub mod consistent_vitest_vi;
pub mod hoisted_apis_on_top;
pub mod no_conditional_tests;
pub mod no_import_node_test;
pub mod no_importing_vitest_globals;
pub mod prefer_called_once;
pub mod prefer_called_times;
pub mod prefer_describe_function_title;
pub mod prefer_expect_type_of;
pub mod prefer_import_in_mock;
pub mod prefer_to_be_falsy;
pub mod prefer_to_be_object;
pub mod prefer_to_be_truthy;
pub mod require_local_test_context_for_concurrent_snapshots;
pub mod warn_todo;

use crate::rule::NativeRule;

/// Return all Vitest rules.
#[must_use]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(consistent_each_for::ConsistentEachFor),
        Box::new(consistent_test_filename::ConsistentTestFilename),
        Box::new(consistent_vitest_vi::ConsistentVitestVi),
        Box::new(hoisted_apis_on_top::HoistedApisOnTop),
        Box::new(no_conditional_tests::NoConditionalTests),
        Box::new(no_import_node_test::NoImportNodeTest),
        Box::new(no_importing_vitest_globals::NoImportingVitestGlobals),
        Box::new(prefer_called_once::PreferCalledOnce),
        Box::new(prefer_called_times::PreferCalledTimes),
        Box::new(prefer_describe_function_title::PreferDescribeFunctionTitle),
        Box::new(prefer_expect_type_of::PreferExpectTypeOf),
        Box::new(prefer_import_in_mock::PreferImportInMock),
        Box::new(prefer_to_be_falsy::PreferToBeFalsy),
        Box::new(prefer_to_be_object::PreferToBeObject),
        Box::new(prefer_to_be_truthy::PreferToBeTruthy),
        Box::new(require_local_test_context_for_concurrent_snapshots::RequireLocalTestContextForConcurrentSnapshots),
        Box::new(warn_todo::WarnTodo),
    ]
}
