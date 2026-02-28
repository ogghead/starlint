//! Node.js-specific lint rules.
//!
//! Rules are prefixed with `node/` in config and output.

pub mod global_require;
pub mod no_exports_assign;
pub mod no_new_require;
pub mod no_path_concat;
pub mod no_process_env;
pub mod no_process_exit;

use crate::rule::NativeRule;

/// Return all Node.js rules.
#[must_use]
pub fn category_rules() -> Vec<Box<dyn NativeRule>> {
    vec![
        Box::new(global_require::GlobalRequire::new()),
        Box::new(no_exports_assign::NoExportsAssign),
        Box::new(no_new_require::NoNewRequire),
        Box::new(no_path_concat::NoPathConcat),
        Box::new(no_process_env::NoProcessEnv),
        Box::new(no_process_exit::NoProcessExit),
    ]
}
