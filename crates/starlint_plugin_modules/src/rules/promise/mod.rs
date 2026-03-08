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
