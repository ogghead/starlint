//! Rule: `react/react-in-jsx-scope`
//!
//! Require `React` to be in scope when using JSX. With the classic JSX
//! transform, JSX compiles to `React.createElement(...)` calls, so `React`
//! must be in scope. This rule is mostly obsolete with the new automatic
//! JSX transform (React 17+).

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags JSX usage when `React` is not imported.
///
/// This is a simplified stub implementation that checks for the presence
/// of a `React` import in the source text. With the modern automatic JSX
/// transform, this rule is largely unnecessary.
#[derive(Debug)]
pub struct ReactInJsxScope;

impl NativeRule for ReactInJsxScope {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/react-in-jsx-scope".to_owned(),
            description: "Require `React` in scope when using JSX".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXElement(element) = kind else {
            return;
        };

        let source = ctx.source_text();

        // Simple heuristic: check if "React" is imported anywhere in the file.
        // A proper implementation would use semantic analysis.
        let has_react_import = source.contains("import React")
            || source.contains("require('react')")
            || source.contains("require(\"react\")");

        if !has_react_import {
            ctx.report_warning(
                "react/react-in-jsx-scope",
                "`React` must be in scope when using JSX",
                Span::new(element.span.start, element.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ReactInJsxScope)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_jsx_without_react_import() {
        let source = "var x = <div />;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "JSX without React import should be flagged");
    }

    #[test]
    fn test_allows_jsx_with_react_import() {
        let source = "import React from 'react';\nvar x = <div />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "JSX with React import should not be flagged"
        );
    }

    #[test]
    fn test_allows_jsx_with_require() {
        let source = "const React = require('react');\nvar x = <div />;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "JSX with React require should not be flagged"
        );
    }
}
