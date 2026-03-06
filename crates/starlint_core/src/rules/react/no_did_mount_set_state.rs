//! Rule: `react/no-did-mount-set-state`
//!
//! Disallow `setState` in `componentDidMount`. Calling `setState` in
//! `componentDidMount` triggers an extra re-render that can cause performance
//! issues and confusing behavior.

use oxc_ast::AstKind;
use oxc_ast::ast::PropertyKey;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `this.setState()` calls inside `componentDidMount`.
#[derive(Debug)]
pub struct NoDidMountSetState;

impl NativeRule for NoDidMountSetState {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-did-mount-set-state".to_owned(),
            description: "Disallow `setState` in `componentDidMount`".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
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

        if method_name != "componentDidMount" {
            return;
        }

        let Some(body) = &method.value.body else {
            return;
        };

        // Walk the body source range looking for this.setState calls
        // We check all call expressions within this method via source span containment
        let method_start = body.span.start;
        let method_end = body.span.end;
        let source = ctx.source_text();

        // Simple source-text scan for `this.setState` within the method body
        let start_idx = usize::try_from(method_start).unwrap_or(0);
        let end_idx = usize::try_from(method_end).unwrap_or(0);
        let body_source = &source[start_idx..end_idx];
        if body_source.contains("this.setState") {
            ctx.report(Diagnostic {
                rule_name: "react/no-did-mount-set-state".to_owned(),
                message: "Do not use `setState` in `componentDidMount`".to_owned(),
                span: Span::new(method.span.start, method.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.jsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDidMountSetState)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_set_state_in_did_mount() {
        let source = r"
class MyComponent extends React.Component {
    componentDidMount() {
        this.setState({ loaded: true });
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "setState in componentDidMount should be flagged"
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
    fn test_allows_did_mount_without_set_state() {
        let source = r"
class MyComponent extends React.Component {
    componentDidMount() {
        console.log('mounted');
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "componentDidMount without setState should not be flagged"
        );
    }
}
