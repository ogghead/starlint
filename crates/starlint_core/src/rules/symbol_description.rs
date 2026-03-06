//! Rule: `symbol-description`
//!
//! Require a description when creating a `Symbol`. Providing a description
//! makes debugging easier since it appears in `toString()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `Symbol()` calls without a description argument.
#[derive(Debug)]
pub struct SymbolDescription;

impl NativeRule for SymbolDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "symbol-description".to_owned(),
            description: "Require a description when creating a Symbol".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::Identifier(id) = &call.callee else {
            return;
        };

        if id.name.as_str() != "Symbol" {
            return;
        }

        if call.arguments.is_empty() {
            // Fix: Symbol() → Symbol('') — insert empty description
            // Find the closing paren and insert '' before it
            #[allow(clippy::as_conversions)]
            let fix = {
                let source = ctx.source_text();
                source
                    .get(call.span.start as usize..call.span.end as usize)
                    .and_then(|text| {
                        text.rfind(')').map(|paren_pos| {
                            let insert_pos = call
                                .span
                                .start
                                .saturating_add(u32::try_from(paren_pos).unwrap_or(0));
                            Fix {
                                message: "Add empty description `''`".to_owned(),
                                edits: vec![Edit {
                                    span: Span::new(insert_pos, insert_pos),
                                    replacement: "''".to_owned(),
                                }],
                                is_snippet: false,
                            }
                        })
                    })
            };

            ctx.report(Diagnostic {
                rule_name: "symbol-description".to_owned(),
                message: "Provide a description for `Symbol()` to aid debugging".to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(SymbolDescription)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_symbol_without_description() {
        let diags = lint("var s = Symbol();");
        assert_eq!(
            diags.len(),
            1,
            "Symbol() without description should be flagged"
        );
    }

    #[test]
    fn test_allows_symbol_with_description() {
        let diags = lint("var s = Symbol('mySymbol');");
        assert!(
            diags.is_empty(),
            "Symbol with description should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_symbol_call() {
        let diags = lint("var x = foo();");
        assert!(diags.is_empty(), "non-Symbol call should not be flagged");
    }
}
