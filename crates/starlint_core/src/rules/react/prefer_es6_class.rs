//! Rule: `react/prefer-es6-class`
//!
//! Prefer ES6 class over `createReactClass`. The `createReactClass` helper
//! is legacy and ES6 classes are the standard way to define React components.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `createReactClass()` and `React.createClass()` calls.
#[derive(Debug)]
pub struct PreferEs6Class;

impl NativeRule for PreferEs6Class {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/prefer-es6-class".to_owned(),
            description: "Prefer ES6 class over `createReactClass`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
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

        let is_create_class = match &call.callee {
            // React.createClass(...)
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "createClass"
                    && matches!(&member.object, Expression::Identifier(id) if id.name.as_str() == "React")
            }
            // createReactClass(...)
            Expression::Identifier(ident) => ident.name.as_str() == "createReactClass",
            _ => false,
        };

        if is_create_class {
            ctx.report_warning(
                "react/prefer-es6-class",
                "Use ES6 class instead of `createReactClass`",
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferEs6Class)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_create_react_class() {
        let source = "var Comp = createReactClass({ render() { return null; } });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "createReactClass should be flagged");
    }

    #[test]
    fn test_flags_react_create_class() {
        let source = "var Comp = React.createClass({ render() { return null; } });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "React.createClass should be flagged");
    }

    #[test]
    fn test_allows_es6_class() {
        let source = r"
class MyComponent extends React.Component {
    render() { return null; }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "ES6 class component should not be flagged"
        );
    }
}
