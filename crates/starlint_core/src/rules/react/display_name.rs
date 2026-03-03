//! Rule: `react/display-name`
//!
//! Component definition is missing display name. Components wrapped in
//! `React.memo()` or `React.forwardRef()` with anonymous functions make
//! debugging harder because they appear as "Anonymous" in React `DevTools`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `React.memo()` and `React.forwardRef()` calls with anonymous functions.
#[derive(Debug)]
pub struct DisplayName;

impl NativeRule for DisplayName {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/display-name".to_owned(),
            description: "Component definition is missing display name".to_owned(),
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

        // Check for React.memo(...) or React.forwardRef(...) or memo(...) or forwardRef(...)
        let wrapper_name = match &call.callee {
            Expression::StaticMemberExpression(member) => {
                let prop = member.property.name.as_str();
                (prop == "memo" || prop == "forwardRef").then_some(prop)
            }
            Expression::Identifier(ident) => {
                let name = ident.name.as_str();
                (name == "memo" || name == "forwardRef").then_some(name)
            }
            _ => None,
        };

        let Some(wrapper) = wrapper_name else {
            return;
        };

        // Check if the first argument is an anonymous function
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let is_anonymous = match first_arg {
            oxc_ast::ast::Argument::FunctionExpression(func) => func.id.is_none(),
            oxc_ast::ast::Argument::ArrowFunctionExpression(_) => true,
            _ => false,
        };

        if is_anonymous {
            ctx.report_warning(
                "react/display-name",
                &format!("Component wrapped in `{wrapper}` is missing a display name"),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(DisplayName)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_memo_with_arrow() {
        let source = "const Comp = React.memo(() => <div />);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "React.memo with arrow function should be flagged"
        );
    }

    #[test]
    fn test_flags_forward_ref_with_arrow() {
        let source = "const Comp = React.forwardRef((props, ref) => <div />);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "React.forwardRef with arrow function should be flagged"
        );
    }

    #[test]
    fn test_flags_memo_with_anonymous_function() {
        let source = "const Comp = React.memo(function() { return <div />; });";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "React.memo with anonymous function should be flagged"
        );
    }

    #[test]
    fn test_allows_memo_with_named_function() {
        let source = "const Comp = React.memo(function MyComp() { return <div />; });";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "React.memo with named function should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_calls() {
        let source = "const x = someFunc(() => <div />);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "non-memo/forwardRef calls should not be flagged"
        );
    }
}
