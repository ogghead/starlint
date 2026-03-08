//! Rule: `jest/no-conditional-expect`
//!
//! Error when `expect()` is inside an if/try-catch block within a test.
//! Simplified: flags `expect(` calls where the source between the test callback
//! start and the expect call contains `if ` or `try {`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-conditional-expect";

/// Flags `expect()` calls inside conditional blocks within tests.
#[derive(Debug)]
pub struct NoConditionalExpect;

impl LintRule for NoConditionalExpect {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow `expect()` inside conditionals in tests".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn should_run_on_file(&self, source_text: &str, file_path: &std::path::Path) -> bool {
        source_text.contains("expect(") && crate::is_test_file(file_path)
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CallExpression])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CallExpression(call) = node else {
            return;
        };

        // Check callee is `it` or `test`
        let callee_name = match ctx.node(call.callee) {
            Some(AstNode::IdentifierReference(id)) => id.name.as_str(),
            _ => return,
        };

        if callee_name != "it" && callee_name != "test" {
            return;
        }

        // Get the callback (second argument)
        let Some(callback_id) = call.arguments.get(1) else {
            return;
        };

        let (body_start, body_end) = match ctx.node(*callback_id) {
            Some(AstNode::ArrowFunctionExpression(arrow)) => (arrow.span.start, arrow.span.end),
            Some(AstNode::Function(func)) => (func.span.start, func.span.end),
            _ => return,
        };

        // Collect violations first to avoid borrow conflict with ctx
        let violations = {
            let source = ctx.source_text();
            let start = usize::try_from(body_start).unwrap_or(0);
            let end = usize::try_from(body_end).unwrap_or(0);
            let body_source = source.get(start..end).unwrap_or("");
            find_conditional_expects(body_source, start)
        };

        for span in violations {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`expect()` should not be placed inside a conditional block".to_owned(),
                span,
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Scan the body source for `expect(` calls preceded by conditional keywords.
fn find_conditional_expects(body_source: &str, offset: usize) -> Vec<Span> {
    let mut results = Vec::new();
    let mut search_start: usize = 0;

    while let Some(expect_pos) = body_source
        .get(search_start..)
        .and_then(|s| s.find("expect("))
    {
        let abs_expect = search_start.saturating_add(expect_pos);
        let before_expect = body_source.get(..abs_expect).unwrap_or("");

        let has_if = before_expect.contains("if ");
        let has_if_paren = before_expect.contains("if(");
        let has_try = before_expect.contains("try {") || before_expect.contains("try{");

        if has_if || has_if_paren || has_try {
            let span_start = offset.saturating_add(abs_expect);
            let expect_end = body_source
                .get(abs_expect..)
                .and_then(|s| s.find(')'))
                .map_or_else(
                    || span_start.saturating_add(7),
                    |p| {
                        offset
                            .saturating_add(abs_expect)
                            .saturating_add(p)
                            .saturating_add(1)
                    },
                );

            results.push(Span::new(
                u32::try_from(span_start).unwrap_or(0),
                u32::try_from(expect_end).unwrap_or(0),
            ));
        }

        search_start = abs_expect.saturating_add(7);
    }

    results
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoConditionalExpect)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_expect_in_if() {
        let source = "test('cond', () => { if (true) { expect(1).toBe(1); } });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "expect inside if should be flagged");
    }

    #[test]
    fn test_flags_expect_in_try() {
        let source = "test('cond', () => { try { expect(1).toBe(1); } catch(e) {} });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "expect inside try should be flagged");
    }

    #[test]
    fn test_allows_unconditional_expect() {
        let source = "test('ok', () => { expect(1).toBe(1); });";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "unconditional expect should not be flagged"
        );
    }
}
