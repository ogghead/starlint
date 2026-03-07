//! React performance lint rules.
//!
//! Rules are prefixed with `react-perf/` in config and output.

pub mod jsx_no_jsx_as_prop;
pub mod jsx_no_new_array_as_prop;
pub mod jsx_no_new_function_as_prop;
pub mod jsx_no_new_object_as_prop;

use crate::rule::NativeRule;

/// Return all react-perf rules.
#[must_use]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![]
}
