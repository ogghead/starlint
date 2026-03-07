//! Rule: `typescript/no-wrapper-object-types`
//!
//! Disallow wrapper object types (`String`, `Number`, `Boolean`, `BigInt`,
//! `Symbol`) in type annotations. These uppercase types refer to the boxed
//! object wrappers, not the primitive types. Almost all TypeScript code should
//! use the lowercase primitive forms (`string`, `number`, `boolean`, `bigint`,
//! `symbol`) instead.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "typescript/no-wrapper-object-types";

/// Wrapper object types mapped to their primitive equivalents.
const WRAPPER_TYPES: &[(&str, &str)] = &[
    ("String", "string"),
    ("Number", "number"),
    ("Boolean", "boolean"),
    ("BigInt", "bigint"),
    ("Symbol", "symbol"),
];

/// Flags `TSTypeReference` nodes that refer to wrapper object types instead
/// of their lowercase primitive equivalents.
#[derive(Debug)]
pub struct NoWrapperObjectTypes;

impl LintRule for NoWrapperObjectTypes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow wrapper object types in type annotations".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
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

        for &(wrapper, primitive) in WRAPPER_TYPES {
            if name == wrapper {
                let message = format!("Use lowercase `{primitive}` instead of `{wrapper}`");
                let type_span = Span::new(type_ref.span.start, type_ref.span.end);
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: message.clone(),
                    span: type_span,
                    severity: Severity::Error,
                    help: Some(message),
                    fix: Some(Fix {
                        kind: FixKind::SafeFix,
                        message: format!("Replace `{wrapper}` with `{primitive}`"),
                        edits: vec![Edit {
                            span: type_span,
                            replacement: primitive.to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoWrapperObjectTypes)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_string_wrapper() {
        let diags = lint("let x: String;");
        assert_eq!(diags.len(), 1, "`String` wrapper type should be flagged");
        assert!(
            diags.first().is_some_and(|d| d.message.contains("string")),
            "message should suggest lowercase `string`"
        );
    }

    #[test]
    fn test_flags_number_wrapper() {
        let diags = lint("function f(n: Number): void {}");
        assert_eq!(diags.len(), 1, "`Number` wrapper type should be flagged");
    }

    #[test]
    fn test_flags_boolean_wrapper() {
        let diags = lint("const b: Boolean = true;");
        assert_eq!(diags.len(), 1, "`Boolean` wrapper type should be flagged");
    }

    #[test]
    fn test_flags_bigint_and_symbol_wrappers() {
        let diags = lint("type Pair = { a: BigInt; b: Symbol };");
        assert_eq!(
            diags.len(),
            2,
            "both `BigInt` and `Symbol` wrapper types should be flagged"
        );
    }

    #[test]
    fn test_allows_lowercase_primitives() {
        let diags = lint("let x: string; let y: number; let z: boolean;");
        assert!(
            diags.is_empty(),
            "lowercase primitive types should not be flagged"
        );
    }

    #[test]
    fn test_allows_custom_types() {
        let diags = lint("interface MyString {} let x: MyString;");
        assert!(
            diags.is_empty(),
            "custom types that are not wrapper types should not be flagged"
        );
    }
}
