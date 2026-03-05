//! Rule: `no-rest-spread-properties`
//!
//! Flag use of object rest/spread properties (`{...obj}` and
//! `const {a, ...rest} = obj`). Some codebases prefer avoiding these
//! for compatibility or clarity.

use oxc_ast::AstKind;
use oxc_ast::ast::ObjectPropertyKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags object spread (`{...obj}`) and object rest (`const {a, ...rest} = obj`).
#[derive(Debug)]
pub struct NoRestSpreadProperties;

impl NativeRule for NoRestSpreadProperties {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-rest-spread-properties".to_owned(),
            description: "Disallow object rest/spread properties".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ObjectExpression, AstType::ObjectPattern])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::ObjectExpression(obj) => {
                for property in &obj.properties {
                    if let ObjectPropertyKind::SpreadProperty(spread) = property {
                        ctx.report(Diagnostic {
                            rule_name: "no-rest-spread-properties".to_owned(),
                            message: "Unexpected object spread property".to_owned(),
                            span: Span::new(spread.span.start, spread.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            AstKind::ObjectPattern(pat) => {
                if let Some(rest) = &pat.rest {
                    ctx.report(Diagnostic {
                        rule_name: "no-rest-spread-properties".to_owned(),
                        message: "Unexpected object rest property".to_owned(),
                        span: Span::new(rest.span.start, rest.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRestSpreadProperties)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_object_spread() {
        let diags = lint("const x = {...obj};");
        assert_eq!(diags.len(), 1, "object spread should be flagged");
    }

    #[test]
    fn test_flags_object_rest() {
        let diags = lint("const {a, ...rest} = obj;");
        assert_eq!(diags.len(), 1, "object rest should be flagged");
    }

    #[test]
    fn test_allows_array_spread() {
        let diags = lint("const x = [1, 2, 3];");
        assert!(diags.is_empty(), "array literal should not be flagged");
    }

    #[test]
    fn test_allows_array_rest() {
        let diags = lint("const [a, ...rest] = arr;");
        assert!(diags.is_empty(), "array rest should not be flagged");
    }

    #[test]
    fn test_allows_plain_object() {
        let diags = lint("const x = { a: 1, b: 2 };");
        assert!(diags.is_empty(), "plain object should not be flagged");
    }

    #[test]
    fn test_flags_multiple_spreads() {
        let diags = lint("const x = {...a, ...b};");
        assert_eq!(
            diags.len(),
            2,
            "two spread properties should produce two diagnostics"
        );
    }
}
