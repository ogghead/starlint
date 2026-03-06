//! Rule: `typescript/no-invalid-void-type`
//!
//! Disallow `void` type outside of return types and generic type parameters.
//! The `void` type is only meaningful as a function return type, indicating that
//! a function does not return a value. Using `void` as a variable type,
//! parameter type, or union member is almost always a mistake — prefer
//! `undefined` in those contexts.

use std::sync::RwLock;

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `void` type annotations that appear outside of return type positions.
///
/// Tracks function return type annotation spans during traversal so that
/// `void` used as a return type is correctly allowed.
#[derive(Debug)]
pub struct NoInvalidVoidType {
    /// Span ranges of active function return type annotations.
    /// When a `TSVoidKeyword` falls within one of these ranges, it is allowed.
    return_type_spans: RwLock<Vec<(u32, u32)>>,
}

impl NoInvalidVoidType {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            return_type_spans: RwLock::new(Vec::new()),
        }
    }
}

impl Default for NoInvalidVoidType {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for NoInvalidVoidType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-invalid-void-type".to_owned(),
            description: "Disallow `void` type outside of return types and generic type parameters"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::Function,
            AstType::TSVoidKeyword,
        ])
    }

    fn leave_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::Function,
            AstType::TSVoidKeyword,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::Function(func) => {
                // Track the span of the return type annotation so void inside it
                // is allowed.
                if let Some(ret) = &func.return_type {
                    if let Ok(mut spans) = self.return_type_spans.write() {
                        spans.push((ret.span.start, ret.span.end));
                    }
                }
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if let Some(ret) = &arrow.return_type {
                    if let Ok(mut spans) = self.return_type_spans.write() {
                        spans.push((ret.span.start, ret.span.end));
                    }
                }
            }
            AstKind::TSVoidKeyword(keyword) => {
                let void_start = keyword.span.start;
                let void_end = keyword.span.end;

                // Allow void in return type positions.
                let in_return_type = self
                    .return_type_spans
                    .read()
                    .map(|spans| {
                        spans
                            .iter()
                            .any(|&(start, end)| void_start >= start && void_end <= end)
                    })
                    .unwrap_or(false);

                if !in_return_type {
                    ctx.report(Diagnostic {
                        rule_name: "typescript/no-invalid-void-type".to_owned(),
                        message: "`void` is only valid as a return type — use `undefined` instead"
                            .to_owned(),
                        span: Span::new(void_start, void_end),
                        severity: Severity::Warning,
                        help: Some("Replace `void` with `undefined`".to_owned()),
                        fix: Some(Fix {
                            message: "Replace with `undefined`".to_owned(),
                            edits: vec![Edit {
                                span: Span::new(void_start, void_end),
                                replacement: "undefined".to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }

    fn leave(&self, kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::Function(func) => {
                if let Some(ret) = &func.return_type {
                    if let Ok(mut spans) = self.return_type_spans.write() {
                        spans
                            .retain(|&(start, end)| start != ret.span.start || end != ret.span.end);
                    }
                }
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if let Some(ret) = &arrow.return_type {
                    if let Ok(mut spans) = self.return_type_spans.write() {
                        spans
                            .retain(|&(start, end)| start != ret.span.start || end != ret.span.end);
                    }
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoInvalidVoidType::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_void_return_type() {
        let diags = lint("function f(): void {}");
        assert!(
            diags.is_empty(),
            "`void` as function return type should not be flagged"
        );
    }

    #[test]
    fn test_allows_void_arrow_return_type() {
        let diags = lint("const f = (): void => {};");
        assert!(
            diags.is_empty(),
            "`void` as arrow function return type should not be flagged"
        );
    }

    #[test]
    fn test_flags_void_variable_type() {
        let diags = lint("let x: void;");
        assert_eq!(diags.len(), 1, "`void` as variable type should be flagged");
    }

    #[test]
    fn test_flags_void_parameter_type() {
        let diags = lint("function f(x: void) {}");
        assert_eq!(diags.len(), 1, "`void` as parameter type should be flagged");
    }

    #[test]
    fn test_allows_void_return_with_void_param_flagged() {
        let diags = lint("function f(x: void): void {}");
        assert_eq!(
            diags.len(),
            1,
            "only the parameter `void` should be flagged, not the return type"
        );
    }
}
