//! Rule: `react/no-render-return-value`
//!
//! Warn when the return value of `ReactDOM.render()` is used.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/no-render-return-value";

/// Flags usage of the return value of `ReactDOM.render()`.
#[derive(Debug)]
pub struct NoRenderReturnValue;

/// Check if a call expression is `ReactDOM.render(...)`.
fn is_react_dom_render(callee: &Expression<'_>) -> bool {
    if let Expression::StaticMemberExpression(member) = callee {
        if member.property.name.as_str() != "render" {
            return false;
        }
        if let Expression::Identifier(obj) = &member.object {
            return obj.name.as_str() == "ReactDOM";
        }
    }
    false
}

impl NativeRule for NoRenderReturnValue {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow using the return value of `ReactDOM.render()`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        if !is_react_dom_render(&call.callee) {
            return;
        }

        // Check the surrounding source to determine if the return value is used.
        let src = ctx.source_text();
        let start = usize::try_from(call.span.start).unwrap_or(0);
        let before = &src[..start];
        let trimmed = before.trim_end();

        // If the call is preceded by `=` (assignment or variable declaration),
        // the return value is being used.
        let return_value_used =
            trimmed.ends_with('=') || trimmed.ends_with('(') || trimmed.ends_with(',');

        if return_value_used {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Do not use the return value of `ReactDOM.render()` — it is a legacy API"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
                labels: vec![],
            });
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRenderReturnValue)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_assigned_render_return_value() {
        let diags =
            lint("const instance = ReactDOM.render(<App />, document.getElementById('root'));");
        assert_eq!(
            diags.len(),
            1,
            "should flag using the return value of ReactDOM.render()"
        );
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_standalone_render_call() {
        let diags = lint("ReactDOM.render(<App />, document.getElementById('root'));");
        assert!(
            diags.is_empty(),
            "should not flag when return value is not used"
        );
    }

    #[test]
    fn test_flags_render_in_assignment() {
        let diags = lint("let x;\nx = ReactDOM.render(<App />, el);");
        assert_eq!(
            diags.len(),
            1,
            "should flag assignment of ReactDOM.render() return value"
        );
    }
}
