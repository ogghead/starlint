//! Declarative macros for reducing plugin and rule boilerplate.

/// Declare a plugin with its rules, generating `create_plugin()`, `all_rules()`,
/// and standard tests.
///
/// Eliminates ~59 lines of identical boilerplate per plugin crate.
///
/// # Usage
///
/// ```ignore
/// declare_plugin! {
///     name: "storybook",
///     rules: [
///         crate::rules::storybook::await_interactions::AwaitInteractions,
///         crate::rules::storybook::default_exports::DefaultExports,
///     ]
/// }
/// ```
///
/// Expands to:
/// - `pub fn create_plugin() -> Box<dyn Plugin>`
/// - `pub fn all_rules() -> Vec<Box<dyn LintRule>>`
/// - A test module verifying rule count and plugin creation
#[macro_export]
macro_rules! declare_plugin {
    (
        name: $plugin_name:expr,
        rules: [ $( $rule_expr:expr ),* $(,)? ]
    ) => {
        /// Create this plugin with all its rules.
        #[must_use]
        pub fn create_plugin() -> Box<dyn $crate::Plugin> {
            Box::new($crate::LintRulePlugin::new(all_rules()))
        }

        /// Return all lint rules in this plugin.
        #[must_use]
        #[allow(clippy::too_many_lines)]
        pub fn all_rules() -> Vec<Box<dyn $crate::LintRule>> {
            vec![
                $( Box::new($rule_expr), )*
            ]
        }

        #[cfg(test)]
        mod plugin_tests {
            use super::*;

            #[test]
            fn test_create_plugin_returns_rules() {
                let plugin = create_plugin();
                let rules = plugin.rules();
                assert!(
                    !rules.is_empty(),
                    concat!($plugin_name, " plugin should provide at least one rule")
                );
            }

            #[test]
            fn test_all_rules_count() {
                let rules = all_rules();
                let expected = [ $( stringify!($rule_expr), )* ].len();
                assert_eq!(
                    rules.len(),
                    expected,
                    concat!($plugin_name, " rule count mismatch")
                );
            }
        }
    };
}
