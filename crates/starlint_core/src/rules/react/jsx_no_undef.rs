//! Rule: `react/jsx-no-undef`
//!
//! Warn when JSX references a component that looks undefined (heuristic:
//! single `PascalCase` word with no dots, not an HTML intrinsic element).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-undef";

/// Well-known global JSX identifiers that should not be flagged.
const KNOWN_GLOBALS: &[&str] = &["React", "Fragment"];

/// Flags JSX references to `PascalCase` component names that are likely
/// undefined. This is a heuristic — full scope analysis would require
/// semantic data.
#[derive(Debug)]
pub struct JsxNoUndef;

impl LintRule for JsxNoUndef {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn when JSX references an undefined component (heuristic)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    #[allow(clippy::manual_let_else)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // JSXOpeningElementNode.name is a String directly in starlint_ast
        let name = opening.name.as_str();

        // Skip lowercase names (HTML intrinsic elements like div, span, etc.)
        let first_char = match name.chars().next() {
            Some(c) => c,
            None => return,
        };
        if !first_char.is_ascii_uppercase() {
            return;
        }

        // Skip well-known globals
        if KNOWN_GLOBALS.contains(&name) {
            return;
        }

        // Heuristic: check for common definition patterns in the source text
        let source = ctx.source_text();

        let has_definition = source.contains(&format!("import {name}"))
            || source.contains(&format!("import {{ {name}"))
            || source.contains(&format!("import {{{name}"))
            || source.contains(&format!("const {name}"))
            || source.contains(&format!("let {name}"))
            || source.contains(&format!("var {name}"))
            || source.contains(&format!("function {name}"))
            || source.contains(&format!("class {name}"));

        if !has_definition {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("`{name}` is not defined — possibly missing import"),
                span: Span::new(opening.span.start, opening.span.end),
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

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(JsxNoUndef)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_undefined_component() {
        let diags = lint("const el = <MyComponent />;");
        assert_eq!(diags.len(), 1, "should flag undefined PascalCase component");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_imported_component() {
        let diags = lint("import MyComponent from './my';\nconst el = <MyComponent />;");
        assert!(diags.is_empty(), "should not flag imported component");
    }

    #[test]
    fn test_allows_html_intrinsic() {
        let diags = lint(r#"const el = <div className="foo" />;"#);
        assert!(diags.is_empty(), "should not flag HTML intrinsic elements");
    }
}
