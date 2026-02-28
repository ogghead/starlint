//! Rule: `react/no-will-update-set-state`
//!
//! Disallow `setState` in `componentWillUpdate`. Calling `setState` in
//! `componentWillUpdate` can cause infinite loops and is a common source
//! of bugs.

use oxc_ast::AstKind;
use oxc_ast::ast::PropertyKey;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `this.setState()` calls inside `componentWillUpdate`.
#[derive(Debug)]
pub struct NoWillUpdateSetState;

impl NativeRule for NoWillUpdateSetState {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-will-update-set-state".to_owned(),
            description: "Disallow `setState` in `componentWillUpdate`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::MethodDefinition(method) = kind else {
            return;
        };

        let method_name = match &method.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if method_name != "componentWillUpdate" {
            return;
        }

        let Some(body) = &method.value.body else {
            return;
        };

        let source = ctx.source_text();
        let start_idx = usize::try_from(body.span.start).unwrap_or(0);
        let end_idx = usize::try_from(body.span.end).unwrap_or(0);
        let body_source = &source[start_idx..end_idx];
        if body_source.contains("this.setState") {
            ctx.report_warning(
                "react/no-will-update-set-state",
                "Do not use `setState` in `componentWillUpdate`",
                Span::new(method.span.start, method.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoWillUpdateSetState)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_set_state_in_will_update() {
        let source = r"
class MyComponent extends React.Component {
    componentWillUpdate() {
        this.setState({ updated: true });
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "setState in componentWillUpdate should be flagged"
        );
    }

    #[test]
    fn test_allows_set_state_in_other_methods() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.setState({ clicked: true });
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "setState in other methods should not be flagged"
        );
    }

    #[test]
    fn test_allows_will_update_without_set_state() {
        let source = r"
class MyComponent extends React.Component {
    componentWillUpdate() {
        console.log('will update');
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "componentWillUpdate without setState should not be flagged"
        );
    }
}
