//! Rule: `react/react-in-jsx-scope`
//!
//! Require `React` to be in scope when using JSX. With the classic JSX
//! transform, JSX compiles to `React.createElement(...)` calls, so `React`
//! must be in scope. This rule is mostly obsolete with the new automatic
//! JSX transform (React 17+).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags JSX usage when `React` is not imported.
///
/// This is a simplified stub implementation that checks for the presence
/// of a `React` import in the source text. With the modern automatic JSX
/// transform, this rule is largely unnecessary.
#[derive(Debug)]
pub struct ReactInJsxScope;

impl LintRule for ReactInJsxScope {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/react-in-jsx-scope".to_owned(),
            description: "Require `React` in scope when using JSX".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXElement(element) = node else {
            return;
        };

        let source = ctx.source_text();

        // Simple heuristic: check if "React" is imported anywhere in the file.
        // A proper implementation would use semantic analysis.
        let has_react_import = source.contains("import React")
            || source.contains("require('react')")
            || source.contains("require(\"react\")");

        if !has_react_import {
            // Fix: insert `import React from 'react';\n` at file start
            let fix = Some(Fix {
                kind: FixKind::SuggestionFix,
                message: "Add `import React from 'react'`".to_owned(),
                edits: vec![Edit {
                    span: Span::new(0, 0),
                    replacement: "import React from 'react';\n".to_owned(),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "react/react-in-jsx-scope".to_owned(),
                message: "`React` must be in scope when using JSX".to_owned(),
                span: Span::new(element.span.start, element.span.end),
                severity: Severity::Warning,
                help: Some("Import React at the top of the file".to_owned()),
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;
    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(ReactInJsxScope)];
        lint_source(source, "test.jsx", &rules)
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
