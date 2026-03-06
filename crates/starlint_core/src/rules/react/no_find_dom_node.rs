//! Rule: `react/no-find-dom-node`
//!
//! Disallow usage of `findDOMNode`. `ReactDOM.findDOMNode` is deprecated and
//! will be removed in a future major version. Use `ref` callbacks instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `findDOMNode()` calls.
#[derive(Debug)]
pub struct NoFindDomNode;

impl NativeRule for NoFindDomNode {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-find-dom-node".to_owned(),
            description: "Disallow usage of `findDOMNode`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let is_find_dom_node = match &call.callee {
            // ReactDOM.findDOMNode(...) or any obj.findDOMNode(...)
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "findDOMNode"
            }
            // findDOMNode(...)
            Expression::Identifier(ident) => ident.name.as_str() == "findDOMNode",
            _ => false,
        };

        if is_find_dom_node {
            ctx.report(Diagnostic {
                rule_name: "react/no-find-dom-node".to_owned(),
                message: "`findDOMNode` is deprecated — use `ref` callbacks or `createRef` instead"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoFindDomNode)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_find_dom_node_call() {
        let source = "var node = findDOMNode(this);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "findDOMNode() should be flagged");
    }

    #[test]
    fn test_flags_react_dom_find_dom_node() {
        let source = "var node = ReactDOM.findDOMNode(this);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "ReactDOM.findDOMNode() should be flagged");
    }

    #[test]
    fn test_allows_other_calls() {
        let source = "var node = document.getElementById('root');";
        let diags = lint(source);
        assert!(diags.is_empty(), "other DOM calls should not be flagged");
    }
}
