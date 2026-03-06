//! Rule: `react/require-render-return`
//!
//! Require a return statement in `render()`. A `render` method that does not
//! return anything will cause the component to render `undefined`, which is
//! almost always a bug.

use oxc_ast::AstKind;
use oxc_ast::ast::{PropertyKey, Statement};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `render()` methods without a return statement.
#[derive(Debug)]
pub struct RequireRenderReturn;

impl NativeRule for RequireRenderReturn {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/require-render-return".to_owned(),
            description: "Require a return statement in `render()`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::MethodDefinition])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::MethodDefinition(method) = kind else {
            return;
        };

        let method_name = match &method.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if method_name != "render" {
            return;
        }

        let Some(body) = &method.value.body else {
            return;
        };

        // Check if the body contains at least one return statement at the top level
        let has_return = body
            .statements
            .iter()
            .any(|stmt| matches!(stmt, Statement::ReturnStatement(_)));

        if !has_return {
            ctx.report(Diagnostic {
                rule_name: "react/require-render-return".to_owned(),
                message: "`render()` method must contain a return statement".to_owned(),
                span: Span::new(method.span.start, method.span.end),
                severity: Severity::Error,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.jsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RequireRenderReturn)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_render_without_return() {
        let source = r"
class MyComponent extends React.Component {
    render() {
        console.log('no return');
    }
}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "render without return should be flagged");
    }

    #[test]
    fn test_allows_render_with_return() {
        let source = r"
class MyComponent extends React.Component {
    render() {
        return <div />;
    }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "render with return should not be flagged");
    }

    #[test]
    fn test_allows_render_returning_null() {
        let source = r"
class MyComponent extends React.Component {
    render() {
        return null;
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "render returning null should not be flagged"
        );
    }
}
