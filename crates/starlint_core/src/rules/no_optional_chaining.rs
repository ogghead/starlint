//! Rule: `no-optional-chaining`
//!
//! Flag use of optional chaining (`?.`). Some codebases prefer explicit
//! null checks over optional chaining for clarity or compatibility.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags any optional chaining expression (`?.`).
#[derive(Debug)]
pub struct NoOptionalChaining;

impl NativeRule for NoOptionalChaining {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-optional-chaining".to_owned(),
            description: "Disallow optional chaining (`?.`)".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ChainExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ChainExpression(chain) = kind else {
            return;
        };

        ctx.report(Diagnostic {
            rule_name: "no-optional-chaining".to_owned(),
            message: "Unexpected use of optional chaining (`?.`)".to_owned(),
            span: Span::new(chain.span.start, chain.span.end),
            severity: Severity::Warning,
            help: None,
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoOptionalChaining)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_optional_member_access() {
        let diags = lint("foo?.bar;");
        assert_eq!(diags.len(), 1, "optional member access should be flagged");
    }

    #[test]
    fn test_flags_optional_call() {
        let diags = lint("foo?.();");
        assert_eq!(diags.len(), 1, "optional call should be flagged");
    }

    #[test]
    fn test_allows_regular_member_access() {
        let diags = lint("foo.bar;");
        assert!(
            diags.is_empty(),
            "regular member access should not be flagged"
        );
    }

    #[test]
    fn test_allows_regular_call() {
        let diags = lint("foo();");
        assert!(diags.is_empty(), "regular call should not be flagged");
    }

    #[test]
    fn test_flags_chained_optional() {
        let diags = lint("foo?.bar?.baz;");
        // A deeply chained `?.` expression is a single ChainExpression
        assert!(
            !diags.is_empty(),
            "chained optional access should be flagged"
        );
    }
}
