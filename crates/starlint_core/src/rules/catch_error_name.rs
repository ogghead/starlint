//! Rule: `catch-error-name`
//!
//! Enforce a consistent parameter name in catch clauses. By default, the
//! expected name is `error`. This improves grep-ability and consistency
//! across a codebase.

use oxc_ast::AstKind;
use oxc_ast::ast::BindingPattern;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default expected catch parameter name.
const DEFAULT_NAME: &str = "error";

/// Flags catch clauses whose parameter name doesn't match the expected name.
#[derive(Debug)]
pub struct CatchErrorName {
    /// The expected catch parameter name.
    expected_name: String,
}

impl Default for CatchErrorName {
    fn default() -> Self {
        Self::new()
    }
}

impl CatchErrorName {
    /// Create a new rule with the default expected name (`error`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            expected_name: DEFAULT_NAME.to_owned(),
        }
    }
}

impl NativeRule for CatchErrorName {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "catch-error-name".to_owned(),
            description: "Enforce a consistent parameter name in catch clauses".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(name) = config.get("name").and_then(serde_json::Value::as_str) {
            name.clone_into(&mut self.expected_name);
        }
        Ok(())
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CatchClause])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CatchClause(clause) = kind else {
            return;
        };

        // No param → nothing to check (`catch {}`)
        let Some(param) = &clause.param else {
            return;
        };

        // Only check simple identifier params, skip destructured patterns
        let BindingPattern::BindingIdentifier(id) = &param.pattern else {
            return;
        };

        let name = id.name.as_str();

        // Allow `_` (intentionally unused convention)
        if name == "_" {
            return;
        }

        if name == self.expected_name {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "catch-error-name".to_owned(),
            message: format!("Catch parameter should be named `{}`", self.expected_name),
            span: Span::new(id.span.start, id.span.end),
            severity: Severity::Warning,
            help: Some(format!("Rename `{name}` to `{}`", self.expected_name)),
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CatchErrorName::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    fn lint_with_name(source: &str, expected: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(CatchErrorName {
                expected_name: expected.to_owned(),
            })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_short_name() {
        let diags = lint("try {} catch (e) {}");
        assert_eq!(diags.len(), 1, "should flag 'e'");
        assert!(
            diags.first().is_some_and(|d| d.message.contains("`error`")),
            "message should suggest 'error'"
        );
    }

    #[test]
    fn test_flags_err() {
        let diags = lint("try {} catch (err) {}");
        assert_eq!(diags.len(), 1, "should flag 'err'");
    }

    #[test]
    fn test_flags_ex() {
        let diags = lint("try {} catch (ex) {}");
        assert_eq!(diags.len(), 1, "should flag 'ex'");
    }

    #[test]
    fn test_allows_error() {
        let diags = lint("try {} catch (error) {}");
        assert!(diags.is_empty(), "'error' should not be flagged");
    }

    #[test]
    fn test_allows_underscore() {
        let diags = lint("try {} catch (_) {}");
        assert!(diags.is_empty(), "'_' should not be flagged");
    }

    #[test]
    fn test_allows_no_param() {
        let diags = lint("try {} catch {}");
        assert!(diags.is_empty(), "no param should not be flagged");
    }

    #[test]
    fn test_allows_destructured() {
        let diags = lint("try {} catch ({ message }) {}");
        assert!(diags.is_empty(), "destructured should not be flagged");
    }

    #[test]
    fn test_configure_custom_name_allows() {
        let diags = lint_with_name("try {} catch (err) {}", "err");
        assert!(diags.is_empty(), "'err' should pass when configured");
    }

    #[test]
    fn test_configure_custom_name_flags() {
        let diags = lint_with_name("try {} catch (error) {}", "err");
        assert_eq!(
            diags.len(),
            1,
            "'error' should fail when 'err' is configured"
        );
    }

    #[test]
    fn test_configure_via_method() {
        let mut rule = CatchErrorName::new();
        let config = serde_json::json!({ "name": "ex" });
        assert!(rule.configure(&config).is_ok());
        assert_eq!(rule.expected_name, "ex", "name should be updated");
    }
}
