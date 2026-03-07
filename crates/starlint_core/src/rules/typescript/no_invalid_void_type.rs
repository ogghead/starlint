//! Rule: `typescript/no-invalid-void-type`
//!
//! Disallow `void` type outside of return types and generic type parameters.
//! The `void` type is only meaningful as a function return type, indicating that
//! a function does not return a value. Using `void` as a variable type,
//! parameter type, or union member is almost always a mistake — prefer
//! `undefined` in those contexts.

#![allow(clippy::cast_possible_truncation, clippy::or_fun_call)]
use std::sync::RwLock;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

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

impl LintRule for NoInvalidVoidType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-invalid-void-type".to_owned(),
            description: "Disallow `void` type outside of return types and generic type parameters"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ArrowFunctionExpression,
            AstNodeType::Function,
            AstNodeType::TSVoidKeyword,
        ])
    }

    fn leave_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::ArrowFunctionExpression,
            AstNodeType::Function,
            AstNodeType::TSVoidKeyword,
        ])
    }

    #[allow(clippy::cast_possible_truncation, clippy::map_unwrap_or)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::Function(func) => {
                // FunctionNode has no return_type field. Use source text to detect
                // the return type annotation span. Look for `):`  pattern after the
                // function params.
                let source = ctx.source_text();
                if let Some((start, end)) =
                    find_return_type_span(source, func.span.start, func.span.end)
                {
                    if let Ok(mut spans) = self.return_type_spans.write() {
                        spans.push((start, end));
                    }
                }
            }
            AstNode::ArrowFunctionExpression(arrow) => {
                let source = ctx.source_text();
                if let Some((start, end)) =
                    find_return_type_span(source, arrow.span.start, arrow.span.end)
                {
                    if let Ok(mut spans) = self.return_type_spans.write() {
                        spans.push((start, end));
                    }
                }
            }
            AstNode::TSVoidKeyword(keyword) => {
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
                            kind: FixKind::SafeFix,
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

    fn leave(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::Function(func) => {
                let source = ctx.source_text();
                if let Some((start, end)) =
                    find_return_type_span(source, func.span.start, func.span.end)
                {
                    if let Ok(mut spans) = self.return_type_spans.write() {
                        spans.retain(|&(s, e)| s != start || e != end);
                    }
                }
            }
            AstNode::ArrowFunctionExpression(arrow) => {
                let source = ctx.source_text();
                if let Some((start, end)) =
                    find_return_type_span(source, arrow.span.start, arrow.span.end)
                {
                    if let Ok(mut spans) = self.return_type_spans.write() {
                        spans.retain(|&(s, e)| s != start || e != end);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Find the return type annotation span in a function/arrow source text.
/// Looks for `):<whitespace><type>` pattern and returns the span of the
/// type portion (after the colon).
#[allow(clippy::as_conversions)]
fn find_return_type_span(source: &str, func_start: u32, func_end: u32) -> Option<(u32, u32)> {
    let start = func_start as usize;
    let end = func_end as usize;
    let text = source.get(start..end)?;

    // Find the closing paren of params
    let mut depth: usize = 0;
    let mut close_paren = None;
    for (i, b) in text.bytes().enumerate() {
        match b {
            b'(' => depth = depth.saturating_add(1),
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_paren = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let close = close_paren?;
    // After `)`, skip whitespace and look for `:`
    let after = text.get(close.saturating_add(1)..)?;
    let trimmed = after.trim_start();
    if !trimmed.starts_with(':') {
        return None;
    }
    // Find where the colon is in absolute offset
    let colon_offset = close
        .saturating_add(1)
        .saturating_add(after.len().saturating_sub(trimmed.len()));
    // The type starts after the colon + whitespace
    let after_colon = trimmed.get(1..)?.trim_start();
    let type_start_in_text = colon_offset.saturating_add(1).saturating_add(
        trimmed
            .len()
            .saturating_sub(1)
            .saturating_sub(after_colon.len()),
    );

    // The type ends at `{` or `=>` (whichever comes first)
    let type_end_in_text = after_colon
        .find('{')
        .or_else(|| after_colon.find("=>"))
        .map_or(
            type_start_in_text.saturating_add(after_colon.len()),
            |pos| type_start_in_text.saturating_add(pos),
        );

    let abs_start = func_start.saturating_add(type_start_in_text as u32);
    let abs_end = func_start.saturating_add(type_end_in_text as u32);
    Some((abs_start, abs_end))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoInvalidVoidType::new())];
        lint_source(source, "test.ts", &rules)
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
