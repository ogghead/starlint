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
