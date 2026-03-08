//! Node.js-specific lint rules.
//!
//! Rules are prefixed with `node/` in config and output.

pub mod global_require;
pub mod no_exports_assign;
pub mod no_new_require;
pub mod no_path_concat;
pub mod no_process_env;
pub mod no_process_exit;
