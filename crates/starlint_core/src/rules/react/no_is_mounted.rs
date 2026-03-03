//! Rule: `react/no-is-mounted`
//!
//! Disallow usage of `isMounted()`. `isMounted` is an anti-pattern, is not
//! available when using ES6 classes, and is on its way to being officially
//! deprecated.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `this.isMounted()` calls.
#[derive(Debug)]
pub struct NoIsMounted;

impl NativeRule for NoIsMounted {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-is-mounted".to_owned(),
            description: "Disallow usage of `isMounted()`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() == "isMounted"
            && matches!(&member.object, Expression::ThisExpression(_))
        {
            ctx.report_error(
                "react/no-is-mounted",
                "`isMounted` is an anti-pattern — use a `_isMounted` instance variable or cancellable promises instead",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoIsMounted)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_is_mounted() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        if (this.isMounted()) {
            this.setState({ clicked: true });
        }
    }
}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "this.isMounted() should be flagged");
    }

    #[test]
    fn test_allows_other_method_calls() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.setState({ clicked: true });
    }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "other method calls should not be flagged");
    }
}
