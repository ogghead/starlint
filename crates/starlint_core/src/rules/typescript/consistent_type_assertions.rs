//! Rule: `typescript/consistent-type-assertions`
//!
//! Prefer `as` syntax over angle-bracket syntax for type assertions. The
//! angle-bracket form (`<Type>expr`) is ambiguous with JSX in `.tsx` files
//! and is less common in modern TypeScript codebases. Using `as` consistently
//! avoids confusion and ensures compatibility with JSX.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags angle-bracket type assertions (`<Type>expr`), preferring `as` syntax.
#[derive(Debug)]
pub struct ConsistentTypeAssertions;

impl LintRule for ConsistentTypeAssertions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/consistent-type-assertions".to_owned(),
            description: "Prefer `as` syntax for type assertions".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSTypeAssertion])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSTypeAssertion(expr) = node else {
            return;
        };

        // Build fix: rewrite <Type>expression to expression as Type
        // TSTypeAssertionNode only has span + expression (NodeId).
        // Extract the type text from the source: it's the text between `<` and `>`
        // at the start of the assertion span.
        let source = ctx.source_text();
        let expr_span = ctx.node(expr.expression).map(AstNode::span);
        let (type_text, expr_text) = {
            let full = source
                .get(expr.span.start as usize..expr.span.end as usize)
                .unwrap_or("");
            // The angle-bracket assertion looks like: <Type>expression
            // Find the closing `>` to split type from expression
            let type_t = full
                .strip_prefix('<')
                .and_then(|rest| rest.find('>').map(|pos| &rest[..pos]))
                .unwrap_or("")
                .to_owned();
            let expr_t = expr_span
                .map_or("", |sp| {
                    source.get(sp.start as usize..sp.end as usize).unwrap_or("")
                })
                .to_owned();
            (type_t, expr_t)
        };

        let fix = (!type_text.is_empty() && !expr_text.is_empty()).then(|| Fix {
            kind: FixKind::SafeFix,
            message: format!("Rewrite to `{expr_text} as {type_text}`"),
            edits: vec![Edit {
                span: Span::new(expr.span.start, expr.span.end),
                replacement: format!("{expr_text} as {type_text}"),
            }],
            is_snippet: false,
        });

        ctx.report(Diagnostic {
            rule_name: "typescript/consistent-type-assertions".to_owned(),
            message: "Use `as` syntax instead of angle-bracket syntax for type assertions"
                .to_owned(),
            span: Span::new(expr.span.start, expr.span.end),
            severity: Severity::Warning,
            help: Some("Use `as` syntax instead".to_owned()),
            fix,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ConsistentTypeAssertions)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_angle_bracket_assertion() {
        let diags = lint("let x = <string>someValue;");
        assert_eq!(
            diags.len(),
            1,
            "angle-bracket type assertion should be flagged"
        );
    }

    #[test]
    fn test_flags_angle_bracket_with_complex_type() {
        let diags = lint("let x = <Array<number>>someValue;");
        assert_eq!(
            diags.len(),
            1,
            "angle-bracket assertion with generic type should be flagged"
        );
    }

    #[test]
    fn test_allows_as_syntax() {
        let diags = lint("let x = someValue as string;");
        assert!(
            diags.is_empty(),
            "`as` syntax assertion should not be flagged"
        );
    }

    #[test]
    fn test_allows_as_const() {
        let diags = lint(r#"let x = "hello" as const;"#);
        assert!(diags.is_empty(), "`as const` should not be flagged");
    }

    #[test]
    fn test_allows_no_assertion() {
        let diags = lint("let x: string = someValue;");
        assert!(
            diags.is_empty(),
            "type annotation without assertion should not be flagged"
        );
    }
}
