//! Rule: `typescript/no-inferrable-types`
//!
//! Disallow explicit type annotations on variables where the type can be
//! trivially inferred from the initializer. For example, `let x: number = 5`
//! is redundant because TypeScript already infers `number` from the literal `5`.
//!
//! Note: Since `starlint_ast::VariableDeclaratorNode` does not have a
//! `type_annotation` field, this rule uses a source-text heuristic to detect
//! `: type` annotations between the binding identifier and the `=` sign.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags explicit type annotations that match trivially inferred types.
#[derive(Debug)]
pub struct NoInferrableTypes;

impl LintRule for NoInferrableTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-inferrable-types".to_owned(),
            description:
                "Disallow explicit type annotations on variables with trivially inferred types"
                    .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::VariableDeclarator])
    }

    #[allow(clippy::as_conversions)]
    #[allow(clippy::arithmetic_side_effects, clippy::cast_possible_truncation)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::VariableDeclarator(decl) = node else {
            return;
        };

        // Must have an initializer
        let Some(init_id) = decl.init else {
            return;
        };

        // Must be a simple binding identifier (not destructuring)
        let Some(AstNode::BindingIdentifier(binding)) = ctx.node(decl.id) else {
            return;
        };

        // Get the initializer node to determine what type it infers
        let Some(init_node) = ctx.node(init_id) else {
            return;
        };

        // Determine the inferred type from the initializer literal
        let inferred_type = match init_node {
            AstNode::NumericLiteral(_) => "number",
            AstNode::StringLiteral(_) => "string",
            AstNode::BooleanLiteral(_) => "boolean",
            _ => return,
        };

        // Use source text to detect if there's a type annotation between the
        // binding identifier and the initializer.
        // Pattern: `identifier: type = value`
        let source = ctx.source_text();
        let binding_end = binding.span.end as usize;
        let init_start = init_node.span().start as usize;

        // Get the text between binding end and init start
        let between = source.get(binding_end..init_start).unwrap_or("");

        // Look for `: type` pattern -- should contain `:` followed by the type keyword,
        // then `=`
        let colon_pos = between.find(':');
        let Some(colon_offset) = colon_pos else {
            return;
        };

        // Extract the type name from between colon and `=`
        let after_colon = &between[colon_offset + 1..];
        let eq_pos = after_colon.find('=').unwrap_or(after_colon.len());
        let type_text = after_colon[..eq_pos].trim();

        // Check if the annotated type matches the inferred type
        if type_text != inferred_type {
            return;
        }

        // Calculate the span of `: type` to remove it
        let ann_start = binding_end + colon_offset;
        let ann_end = binding_end + colon_offset + 1 + eq_pos;
        // Trim trailing whitespace before `=`
        let ann_text = source.get(ann_start..ann_end).unwrap_or("");
        let trimmed_end = ann_start + ann_text.trim_end().len();

        ctx.report(Diagnostic {
            rule_name: "typescript/no-inferrable-types".to_owned(),
            message: format!("Type `{inferred_type}` is trivially inferred from the initializer"),
            span: Span::new(decl.span.start, decl.span.end),
            severity: Severity::Warning,
            help: Some(format!("Remove the `{inferred_type}` type annotation")),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: format!("Remove the `{inferred_type}` type annotation"),
                edits: vec![Edit {
                    span: Span::new(ann_start as u32, trimmed_end as u32),
                    replacement: String::new(),
                }],
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
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoInferrableTypes)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_number_with_numeric_literal() {
        let diags = lint("let x: number = 5;");
        assert_eq!(
            diags.len(),
            1,
            "`let x: number = 5` should be flagged as inferrable"
        );
    }

    #[test]
    fn test_flags_string_with_string_literal() {
        let diags = lint(r#"let x: string = "hello";"#);
        assert_eq!(
            diags.len(),
            1,
            r#"`let x: string = "hello"` should be flagged as inferrable"#
        );
    }

    #[test]
    fn test_flags_boolean_with_boolean_literal() {
        let diags = lint("let x: boolean = true;");
        assert_eq!(
            diags.len(),
            1,
            "`let x: boolean = true` should be flagged as inferrable"
        );
    }

    #[test]
    fn test_allows_type_annotation_without_init() {
        let diags = lint("let x: number;");
        assert!(
            diags.is_empty(),
            "type annotation without initializer should not be flagged"
        );
    }

    #[test]
    fn test_allows_init_without_type_annotation() {
        let diags = lint("let x = 5;");
        assert!(
            diags.is_empty(),
            "initializer without type annotation should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_trivial_type() {
        let diags = lint("let x: Foo = new Foo();");
        assert!(
            diags.is_empty(),
            "non-trivial type annotation should not be flagged"
        );
    }
}
