//! Rule: `react/no-set-state`
//!
//! Disallow usage of `setState`. When using an external state management
//! library (Redux, `MobX`, etc.), `setState` should not be used at all.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags all `this.setState()` and bare `setState()` calls.
#[derive(Debug)]
pub struct NoSetState;

impl NativeRule for NoSetState {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-set-state".to_owned(),
            description: "Disallow usage of `setState`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let is_set_state = match &call.callee {
            // this.setState(...)
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "setState"
                    && matches!(&member.object, Expression::ThisExpression(_))
            }
            // setState(...)
            Expression::Identifier(ident) => ident.name.as_str() == "setState",
            _ => false,
        };

        if is_set_state {
            ctx.report_warning(
                "react/no-set-state",
                "Do not use `setState` — manage state with an external store instead",
                Span::new(call.span.start, call.span.end),
            );
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.jsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoSetState)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_this_set_state() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.setState({ count: 1 });
    }
}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "this.setState should be flagged");
    }

    #[test]
    fn test_flags_bare_set_state() {
        let source = "setState({ count: 1 });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "bare setState call should be flagged");
    }

    #[test]
    fn test_allows_other_method_calls() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.forceUpdate();
    }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "other method calls should not be flagged");
    }
}
