//! Promise-related lint rules.
//!
//! Rules are prefixed with `promise/` in config and output.

pub mod always_return;
pub mod avoid_new;
pub mod catch_or_return;
pub mod no_callback_in_promise;
pub mod no_multiple_resolved;
pub mod no_native;
pub mod no_nesting;
pub mod no_new_statics;
pub mod no_promise_in_callback;
pub mod no_return_in_finally;
pub mod no_return_wrap;
pub mod param_names;
pub mod prefer_await_to_callbacks;
pub mod prefer_await_to_then;
pub mod spec_only;
pub mod valid_params;

use crate::rule::NativeRule;

/// Return all Promise rules.
#[must_use]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(always_return::AlwaysReturn),
        Box::new(avoid_new::AvoidNew),
        Box::new(catch_or_return::CatchOrReturn),
        Box::new(no_callback_in_promise::NoCallbackInPromise),
        Box::new(no_multiple_resolved::NoMultipleResolved),
        Box::new(no_nesting::NoNesting),
        Box::new(no_new_statics::NoNewStatics),
        Box::new(no_promise_in_callback::NoPromiseInCallback),
        Box::new(no_return_in_finally::NoReturnInFinally),
        Box::new(no_return_wrap::NoReturnWrap),
        Box::new(param_names::ParamNames),
        Box::new(prefer_await_to_callbacks::PreferAwaitToCallbacks),
        Box::new(prefer_await_to_then::PreferAwaitToThen),
        Box::new(spec_only::SpecOnly),
        Box::new(valid_params::ValidParams),
    ]
}
