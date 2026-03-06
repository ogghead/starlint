//! Rule: `react/no-unsafe`
//!
//! Warn when using unsafe lifecycle methods: `UNSAFE_componentWillMount`,
//! `UNSAFE_componentWillReceiveProps`, `UNSAFE_componentWillUpdate`.

use oxc_ast::AstKind;
use oxc_ast::ast::PropertyKey;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of `UNSAFE_` lifecycle methods.
#[derive(Debug)]
pub struct NoUnsafe;

impl NativeRule for NoUnsafe {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-unsafe".to_owned(),
            description: "Disallow usage of unsafe lifecycle methods".to_owned(),
            category: Category::Correctness,
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

        let (method_name, id_span) = match &method.key {
            PropertyKey::StaticIdentifier(id) => (id.name.as_str(), id.span),
            _ => return,
        };

        let safe_name = match method_name {
            "UNSAFE_componentWillMount" => Some("componentDidMount"),
            "UNSAFE_componentWillReceiveProps" => Some("componentDidUpdate"),
            "UNSAFE_componentWillUpdate" => Some("getSnapshotBeforeUpdate"),
            _ => None,
        };

        if let Some(replacement) = safe_name {
            ctx.report(Diagnostic {
                rule_name: "react/no-unsafe".to_owned(),
                message: format!(
                    "`{method_name}` is unsafe and deprecated — use safe lifecycle methods instead"
                ),
                span: Span::new(method.span.start, method.span.end),
                severity: Severity::Warning,
                help: Some(format!("Replace with `{replacement}`")),
                fix: Some(Fix {
                    kind: FixKind::SuggestionFix,
                    message: format!("Rename to `{replacement}`"),
                    edits: vec![Edit {
                        span: Span::new(id_span.start, id_span.end),
                        replacement: replacement.to_owned(),
                    }],
                    is_snippet: false,
                }),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnsafe)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_unsafe_component_will_mount() {
        let diags = lint("class Foo extends React.Component { UNSAFE_componentWillMount() {} }");
        assert_eq!(diags.len(), 1, "should flag UNSAFE_componentWillMount");
    }

    #[test]
    fn test_flags_unsafe_component_will_receive_props() {
        let diags =
            lint("class Foo extends React.Component { UNSAFE_componentWillReceiveProps() {} }");
        assert_eq!(
            diags.len(),
            1,
            "should flag UNSAFE_componentWillReceiveProps"
        );
    }

    #[test]
    fn test_allows_safe_lifecycle_methods() {
        let diags = lint(
            "class Foo extends React.Component { componentDidMount() {} render() { return null; } }",
        );
        assert!(
            diags.is_empty(),
            "safe lifecycle methods should not be flagged"
        );
    }
}
