//! Rule: `react/no-namespace`
//!
//! Error when JSX elements use namespaced names like `<ns:Component>`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags JSX namespaced element names.
#[derive(Debug)]
pub struct NoNamespace;

impl NativeRule for NoNamespace {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-namespace".to_owned(),
            description: "Disallow namespaced JSX elements (e.g. `<ns:Component>`)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXNamespacedName])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXNamespacedName(ns_name) = kind else {
            return;
        };

        let namespace = ns_name.namespace.name.as_str();
        let name = ns_name.name.name.as_str();

        ctx.report(Diagnostic {
            rule_name: "react/no-namespace".to_owned(),
            message: format!("React does not support JSX namespaces — found `{namespace}:{name}`"),
            span: Span::new(ns_name.span.start, ns_name.span.end),
            severity: Severity::Error,
            help: None,
            fix: None,
            labels: vec![],
        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoNamespace)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_namespaced_element() {
        let diags = lint(r"const x = <ns:Component />;");
        // ns:Component generates a JSXNamespacedName node
        assert!(!diags.is_empty(), "should flag namespaced JSX element");
    }

    #[test]
    fn test_flags_namespaced_attribute() {
        let diags = lint(r#"const x = <div xml:lang="en" />;"#);
        // xml:lang generates a JSXNamespacedName for the attribute name
        assert!(!diags.is_empty(), "should flag namespaced JSX attribute");
    }

    #[test]
    fn test_allows_normal_element() {
        let diags = lint(r"const x = <Component />;");
        assert!(diags.is_empty(), "normal elements should not be flagged");
    }
}
