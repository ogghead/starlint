//! Rule: `typescript/no-unsafe-function-type`
//!
//! Disallow the `Function` type. The `Function` type accepts any function-like
//! value and provides no type safety for calling the value — arguments and
//! return type are all `any`. Prefer specific function signatures like
//! `() => void`, `(arg: string) => number`, or the `(...args: any[]) => any`
//! escape hatch when the signature is truly unknown.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{FixBuilder, LintContext, LintRule};

/// Flags usage of the `Function` type in type annotations.
#[derive(Debug)]
pub struct NoUnsafeFunctionType;

impl LintRule for NoUnsafeFunctionType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unsafe-function-type".to_owned(),
            description: "Disallow the `Function` type — use a specific function signature instead"
                .to_owned(),
            category: Category::Correctness,
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

        if type_ref.type_name != "Function" {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "typescript/no-unsafe-function-type".to_owned(),
            message: "The `Function` type is unsafe — use a specific function type like `() => void` instead".to_owned(),
            span: Span::new(type_ref.span.start, type_ref.span.end),
            severity: Severity::Warning,
            help: Some("Replace with `(...args: any[]) => any`".to_owned()),
            fix: FixBuilder::new("Replace with `(...args: any[]) => any`", FixKind::SafeFix)
                .replace(Span::new(type_ref.span.start, type_ref.span.end), "(...args: any[]) => any")
                .build(),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoUnsafeFunctionType, "test.ts");

    #[test]
    fn test_flags_function_variable() {
        let diags = lint("let f: Function;");
        assert_eq!(diags.len(), 1, "`Function` type should be flagged");
    }

    #[test]
    fn test_flags_function_parameter() {
        let diags = lint("function run(cb: Function) {}");
        assert_eq!(
            diags.len(),
            1,
            "`Function` as parameter type should be flagged"
        );
    }

    #[test]
    fn test_flags_function_return_type() {
        let diags = lint("function factory(): Function { return () => {}; }");
        assert_eq!(
            diags.len(),
            1,
            "`Function` as return type should be flagged"
        );
    }

    #[test]
    fn test_allows_specific_function_type() {
        let diags = lint("let f: () => void;");
        assert!(
            diags.is_empty(),
            "specific function type should not be flagged"
        );
    }

    #[test]
    fn test_allows_function_with_args() {
        let diags = lint("let f: (x: number) => string;");
        assert!(
            diags.is_empty(),
            "typed function signature should not be flagged"
        );
    }
}
