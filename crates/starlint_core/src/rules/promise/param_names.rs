//! Rule: `promise/param-names`
//!
//! Enforce standard `resolve`/`reject` parameter names in `new Promise()`
//! executors. Consistent naming improves readability.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `new Promise()` executors whose parameters are not named
/// `resolve` and `reject`.
#[derive(Debug)]
pub struct ParamNames;

impl NativeRule for ParamNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "promise/param-names".to_owned(),
            description: "Enforce standard `resolve`/`reject` parameter names".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::NewExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::NewExpression(new_expr) = kind else {
            return;
        };

        let Expression::Identifier(ident) = &new_expr.callee else {
            return;
        };

        if ident.name.as_str() != "Promise" {
            return;
        }

        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        let arg_expr = match first_arg {
            oxc_ast::ast::Argument::SpreadElement(_) => return,
            _ => first_arg.to_expression(),
        };

        // Extract parameter names from the executor function
        let params = match arg_expr {
            Expression::ArrowFunctionExpression(arrow) => &arrow.params,
            Expression::FunctionExpression(func) => &func.params,
            _ => return,
        };

        let items = &params.items;

        // Check first parameter (resolve)
        if let Some(first) = items.first() {
            if let BindingPattern::BindingIdentifier(id) = &first.pattern {
                let name = id.name.as_str();
                if name != "resolve" && name != "_resolve" && name != "_" {
                    ctx.report(Diagnostic {
                        rule_name: "promise/param-names".to_owned(),
                        message: format!(
                            "Promise executor first parameter should be named `resolve`, found `{name}`"
                        ),
                        span: Span::new(id.span.start, id.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: "Rename to `resolve`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(id.span.start, id.span.end),
                                replacement: "resolve".to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
        }

        // Check second parameter (reject)
        if let Some(second) = items.get(1) {
            if let BindingPattern::BindingIdentifier(id) = &second.pattern {
                let name = id.name.as_str();
                if name != "reject" && name != "_reject" && name != "_" {
                    ctx.report(Diagnostic {
                        rule_name: "promise/param-names".to_owned(),
                        message: format!(
                            "Promise executor second parameter should be named `reject`, found `{name}`"
                        ),
                        span: Span::new(id.span.start, id.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: Some(Fix {
                            kind: FixKind::SuggestionFix,
                            message: "Rename to `reject`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(id.span.start, id.span.end),
                                replacement: "reject".to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ParamNames)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_non_standard_names() {
        let diags = lint("new Promise((yes, no) => { yes(1); });");
        assert_eq!(diags.len(), 2, "should flag both non-standard param names");
    }

    #[test]
    fn test_allows_standard_names() {
        let diags = lint("new Promise((resolve, reject) => { resolve(1); });");
        assert!(diags.is_empty(), "standard names should be allowed");
    }

    #[test]
    fn test_allows_underscore_prefix() {
        let diags = lint("new Promise((_resolve, _reject) => { });");
        assert!(
            diags.is_empty(),
            "underscore-prefixed names should be allowed"
        );
    }
}
