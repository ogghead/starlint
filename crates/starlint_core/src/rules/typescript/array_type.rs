//! Rule: `typescript/array-type`
//!
//! Enforce consistent array type style. By default, enforces the "array" style
//! where `Array<T>` should be written as `T[]` and `ReadonlyArray<T>` should be
//! written as `readonly T[]`. Generic wrapper types like `Array<T>` are more
//! verbose and less conventional in most `TypeScript` codebases.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Type names that should use shorthand array syntax instead.
const ARRAY_TYPE_NAMES: &[&str] = &["Array", "ReadonlyArray"];

/// Flags `Array<T>` and `ReadonlyArray<T>` type references, preferring `T[]`
/// and `readonly T[]` shorthand syntax.
#[derive(Debug)]
pub struct ArrayType;

impl LintRule for ArrayType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/array-type".to_owned(),
            description: "Enforce consistent array type style (`T[]` instead of `Array<T>`)"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSTypeReference])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSTypeReference(type_ref) = node else {
            return;
        };

        let name = type_ref.type_name.as_str();

        if !ARRAY_TYPE_NAMES.contains(&name) {
            return;
        }

        let suggestion = if name == "ReadonlyArray" {
            "Use `readonly T[]` instead of `ReadonlyArray<T>`"
        } else {
            "Use `T[]` instead of `Array<T>`"
        };

        // Extract the type parameter text from inside the angle brackets
        let source = ctx.source_text();
        let ref_start = usize::try_from(type_ref.span.start).unwrap_or(0);
        let ref_end = usize::try_from(type_ref.span.end).unwrap_or(0);
        let ref_text = source.get(ref_start..ref_end).unwrap_or("");

        // Find the content between `<` and `>` for the type argument
        let fix = ref_text.find('<').and_then(|open| {
            ref_text.rfind('>').map(|close| {
                let inner = ref_text.get(open.saturating_add(1)..close).unwrap_or("");
                if name == "ReadonlyArray" {
                    format!("readonly {inner}[]")
                } else {
                    // If the type arg contains `|` or `&`, wrap in parens for correctness
                    if inner.contains('|') || inner.contains('&') {
                        format!("({inner})[]")
                    } else {
                        format!("{inner}[]")
                    }
                }
            })
        });

        let span = Span::new(type_ref.span.start, type_ref.span.end);

        ctx.report(Diagnostic {
            rule_name: "typescript/array-type".to_owned(),
            message: suggestion.to_owned(),
            span,
            severity: Severity::Warning,
            help: Some(suggestion.to_owned()),
            fix: fix.map(|replacement| Fix {
                kind: FixKind::SafeFix,
                message: suggestion.to_owned(),
                edits: vec![Edit { span, replacement }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ArrayType)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_array_generic() {
        let diags = lint("let x: Array<number>;");
        assert_eq!(diags.len(), 1, "`Array<number>` should be flagged");
    }

    #[test]
    fn test_flags_readonly_array_generic() {
        let diags = lint("let x: ReadonlyArray<string>;");
        assert_eq!(diags.len(), 1, "`ReadonlyArray<string>` should be flagged");
    }

    #[test]
    fn test_allows_shorthand_array() {
        let diags = lint("let x: number[];");
        assert!(
            diags.is_empty(),
            "`number[]` shorthand should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_generic_types() {
        let diags = lint("let x: Map<string, number>;");
        assert!(diags.is_empty(), "`Map<K, V>` should not be flagged");
    }

    #[test]
    fn test_flags_nested_array_generic() {
        let diags = lint("let x: Array<Array<number>>;");
        assert_eq!(
            diags.len(),
            2,
            "both nested `Array<>` references should be flagged"
        );
    }
}
