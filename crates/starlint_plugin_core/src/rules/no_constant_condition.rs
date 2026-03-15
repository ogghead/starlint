//! Rule: `no-constant-condition`
//!
//! Disallow constant expressions in conditions. A condition that always evaluates
//! to the same value is almost certainly a bug or dead code.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags conditions that are always truthy or falsy due to being a literal value.
#[derive(Debug)]
pub struct NoConstantCondition;

/// Returns `true` if the expression is a literal (boolean, numeric, null, string,
/// or a template literal with no interpolations).
fn is_constant_expression(expr: &AstNode) -> bool {
    match expr {
        AstNode::BooleanLiteral(_)
        | AstNode::NumericLiteral(_)
        | AstNode::NullLiteral(_)
        | AstNode::StringLiteral(_) => true,
        AstNode::TemplateLiteral(tpl) => tpl.expressions.is_empty(),
        _ => false,
    }
}

/// Determine the JS truthiness of a constant literal expression.
///
/// Returns `Some(true)` for truthy literals, `Some(false)` for falsy, `None` if
/// truthiness cannot be determined.
fn is_truthy_literal(expr: &AstNode) -> Option<bool> {
    match expr {
        AstNode::BooleanLiteral(lit) => Some(lit.value),
        AstNode::NumericLiteral(lit) => Some(lit.value != 0.0 && !lit.value.is_nan()),
        AstNode::NullLiteral(_) => Some(false),
        AstNode::StringLiteral(lit) => Some(!lit.value.is_empty()),
        AstNode::TemplateLiteral(tpl) => {
            if !tpl.expressions.is_empty() {
                return None;
            }
            // Empty template `` is falsy (empty string), non-empty is truthy.
            let is_empty = tpl.quasis.iter().all(std::string::String::is_empty);
            Some(!is_empty)
        }
        _ => None,
    }
}

impl LintRule for NoConstantCondition {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-constant-condition".to_owned(),
            description: "Disallow constant expressions in conditions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    #[allow(clippy::too_many_lines)] // Five AstKind arms with similar structure
    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ConditionalExpression,
            AstNodeType::DoWhileStatement,
            AstNodeType::ForStatement,
            AstNodeType::IfStatement,
            AstNodeType::WhileStatement,
        ])
    }

    #[allow(clippy::too_many_lines)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::IfStatement(stmt) => {
                let Some(test_node) = ctx.node(stmt.test) else {
                    return;
                };
                if is_constant_expression(test_node) {
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in `if` statement".to_owned(),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::WhileStatement(stmt) => {
                let Some(test_node) = ctx.node(stmt.test) else {
                    return;
                };
                if is_constant_expression(test_node) {
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in `while` statement".to_owned(),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::DoWhileStatement(stmt) => {
                let Some(test_node) = ctx.node(stmt.test) else {
                    return;
                };
                if is_constant_expression(test_node) {
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in `do-while` statement".to_owned(),
                        span: Span::new(stmt.span.start, stmt.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            AstNode::ForStatement(stmt) => {
                if let Some(test_id) = stmt.test {
                    let Some(test_node) = ctx.node(test_id) else {
                        return;
                    };
                    if is_constant_expression(test_node) {
                        ctx.report(Diagnostic {
                            rule_name: "no-constant-condition".to_owned(),
                            message: "Unexpected constant condition in `for` statement".to_owned(),
                            span: Span::new(stmt.span.start, stmt.span.end),
                            severity: Severity::Error,
                            help: Some(
                                "Replace the constant condition with a dynamic expression"
                                    .to_owned(),
                            ),
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            AstNode::ConditionalExpression(expr) => {
                let Some(test_node) = ctx.node(expr.test) else {
                    return;
                };
                if is_constant_expression(test_node) {
                    let source = ctx.source_text();
                    let fix = is_truthy_literal(test_node).and_then(|truthy| {
                        let branch_id = if truthy {
                            expr.consequent
                        } else {
                            expr.alternate
                        };
                        let branch_span = ctx.node(branch_id)?.span();
                        let start = usize::try_from(branch_span.start).ok()?;
                        let end = usize::try_from(branch_span.end).ok()?;
                        let branch_text = source.get(start..end)?;
                        Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!(
                                "Replace with {} branch",
                                if truthy { "consequent" } else { "alternate" }
                            ),
                            edits: vec![Edit {
                                span: Span::new(expr.span.start, expr.span.end),
                                replacement: branch_text.to_owned(),
                            }],
                            is_snippet: false,
                        })
                    });
                    ctx.report(Diagnostic {
                        rule_name: "no-constant-condition".to_owned(),
                        message: "Unexpected constant condition in ternary expression".to_owned(),
                        span: Span::new(expr.span.start, expr.span.end),
                        severity: Severity::Error,
                        help: Some(
                            "Replace the constant condition with a dynamic expression".to_owned(),
                        ),
                        fix,
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

    use super::*;

    starlint_rule_framework::lint_rule_test!(NoConstantCondition);

    #[test]
    fn test_flags_if_true() {
        let diags = lint("if (true) {}");
        assert_eq!(diags.len(), 1, "should flag constant condition");
        assert_eq!(
            diags.first().map(|d| d.rule_name.as_str()),
            Some("no-constant-condition"),
            "rule name should match"
        );
    }

    #[test]
    fn test_flags_while_false() {
        let diags = lint("while (false) {}");
        assert_eq!(diags.len(), 1, "should flag constant condition");
        assert_eq!(
            diags.first().map(|d| d.rule_name.as_str()),
            Some("no-constant-condition"),
            "rule name should match"
        );
    }

    #[test]
    fn test_allows_variable_condition() {
        let diags = lint("if (x) {}");
        assert!(diags.is_empty(), "variable condition should not be flagged");
    }

    #[test]
    fn test_flags_ternary_literal() {
        let diags = lint("var r = true ? 1 : 2;");
        assert_eq!(diags.len(), 1, "should flag constant ternary condition");
        assert_eq!(
            diags.first().map(|d| d.rule_name.as_str()),
            Some("no-constant-condition"),
            "rule name should match"
        );
    }

    #[test]
    fn test_flags_if_zero() {
        let diags = lint("if (0) { doSomething(); }");
        assert_eq!(diags.len(), 1, "should flag numeric literal condition");
    }

    #[test]
    fn test_flags_if_null() {
        let diags = lint("if (null) { doSomething(); }");
        assert_eq!(diags.len(), 1, "should flag null literal condition");
    }

    #[test]
    fn test_flags_if_string() {
        let diags = lint(r#"if ("yes") { doSomething(); }"#);
        assert_eq!(diags.len(), 1, "should flag string literal condition");
    }

    #[test]
    fn test_flags_do_while_constant() {
        let diags = lint("do { x++; } while (true);");
        assert_eq!(diags.len(), 1, "should flag constant do-while condition");
    }

    #[test]
    fn test_flags_for_constant_test() {
        let diags = lint("for (let i = 0; true; i++) {}");
        assert_eq!(diags.len(), 1, "should flag constant for-loop test");
    }

    #[test]
    fn test_allows_for_no_test() {
        let diags = lint("for (;;) { break; }");
        assert!(
            diags.is_empty(),
            "for loop with no test should not be flagged"
        );
    }

    #[test]
    fn test_flags_template_literal_no_interpolation() {
        let diags = lint("if (`constant`) {}");
        assert_eq!(
            diags.len(),
            1,
            "template literal without interpolation is constant"
        );
    }

    #[test]
    fn test_allows_template_literal_with_interpolation() {
        let diags = lint("if (`hello ${x}`) {}");
        assert!(
            diags.is_empty(),
            "template literal with interpolation is not constant"
        );
    }

    #[test]
    fn test_ternary_true_fix_replaces_with_consequent() {
        let diags = lint("var r = true ? 1 : 2;");
        assert_eq!(diags.len(), 1);
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "ternary should have a fix");
        let edit = fix.and_then(|f| f.edits.first());
        assert_eq!(
            edit.map(|e| e.replacement.as_str()),
            Some("1"),
            "truthy condition should replace with consequent"
        );
    }

    #[test]
    fn test_ternary_false_fix_replaces_with_alternate() {
        let diags = lint("var r = false ? 1 : 2;");
        assert_eq!(diags.len(), 1);
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert!(fix.is_some(), "ternary should have a fix");
        let edit = fix.and_then(|f| f.edits.first());
        assert_eq!(
            edit.map(|e| e.replacement.as_str()),
            Some("2"),
            "falsy condition should replace with alternate"
        );
    }

    #[test]
    fn test_ternary_null_fix() {
        let diags = lint("var r = null ? a : b;");
        let edit = diags
            .first()
            .and_then(|d| d.fix.as_ref())
            .and_then(|f| f.edits.first());
        assert_eq!(
            edit.map(|e| e.replacement.as_str()),
            Some("b"),
            "null is falsy, should replace with alternate"
        );
    }

    #[test]
    fn test_if_statement_has_no_fix() {
        let diags = lint("if (true) { x(); }");
        assert_eq!(diags.len(), 1, "should flag if(true)");
        assert!(
            diags.first().and_then(|d| d.fix.as_ref()).is_none(),
            "if statement should not have a fix"
        );
    }

    #[test]
    fn test_allows_ternary_variable() {
        let diags = lint("var r = x ? 1 : 2;");
        assert!(
            diags.is_empty(),
            "ternary with variable condition should not be flagged"
        );
    }
}
