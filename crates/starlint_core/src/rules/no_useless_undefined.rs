//! Rule: `no-useless-undefined` (unicorn)
//!
//! Disallow useless `undefined`. Using `undefined` as a default value,
//! return value, or argument is usually unnecessary since JavaScript
//! provides it implicitly.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags useless uses of `undefined`.
#[derive(Debug)]
pub struct NoUselessUndefined;

impl NativeRule for NoUselessUndefined {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-undefined".to_owned(),
            description: "Disallow useless undefined".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ReturnStatement, AstType::VariableDeclarator])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            // `let x = undefined;` -> `let x;`
            AstKind::VariableDeclarator(decl) => {
                if let Some(init) = &decl.init {
                    if is_undefined(init) {
                        // Remove from end of binding id to end of init (` = undefined`)
                        let remove_span = Span::new(decl.id.span().end, init.span().end);
                        ctx.report(Diagnostic {
                            rule_name: "no-useless-undefined".to_owned(),
                            message: "Do not use useless `undefined`".to_owned(),
                            span: Span::new(decl.span.start, decl.span.end),
                            severity: Severity::Warning,
                            help: Some("Remove `= undefined`".to_owned()),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: "Remove `= undefined`".to_owned(),
                                edits: vec![Edit {
                                    span: remove_span,
                                    replacement: String::new(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            // `return undefined;` -> `return;`
            AstKind::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    if is_undefined(arg) {
                        // Remove from after `return` keyword to end of argument (` undefined`)
                        // `return` is 6 chars, so the keyword ends at ret.span.start + 6
                        let return_keyword_end = ret.span.start.saturating_add(6);
                        let remove_span = Span::new(return_keyword_end, arg.span().end);
                        ctx.report(Diagnostic {
                            rule_name: "no-useless-undefined".to_owned(),
                            message: "Do not use useless `undefined`".to_owned(),
                            span: Span::new(ret.span.start, ret.span.end),
                            severity: Severity::Warning,
                            help: Some("Remove `undefined` from return".to_owned()),
                            fix: Some(Fix {
                                kind: FixKind::SafeFix,
                                message: "Remove `undefined` from return".to_owned(),
                                edits: vec![Edit {
                                    span: remove_span,
                                    replacement: String::new(),
                                }],
                                is_snippet: false,
                            }),
                            labels: vec![],
                        });
                    }
                }
            }
            // `void 0` is a different pattern (intentional), skip it
            _ => {}
        }
    }
}

/// Check if an expression is `undefined` (the identifier).
fn is_undefined(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Identifier(ident) if ident.name == "undefined")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessUndefined)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_let_undefined() {
        let diags = lint("let x = undefined;");
        assert_eq!(diags.len(), 1, "let x = undefined should be flagged");
    }

    #[test]
    fn test_flags_return_undefined() {
        let diags = lint("function foo() { return undefined; }");
        assert_eq!(diags.len(), 1, "return undefined should be flagged");
    }

    #[test]
    fn test_allows_let_with_value() {
        let diags = lint("let x = 1;");
        assert!(diags.is_empty(), "let with value should not be flagged");
    }

    #[test]
    fn test_allows_return_nothing() {
        let diags = lint("function foo() { return; }");
        assert!(diags.is_empty(), "bare return should not be flagged");
    }

    #[test]
    fn test_allows_let_no_init() {
        let diags = lint("let x;");
        assert!(diags.is_empty(), "let without init should not be flagged");
    }
}
