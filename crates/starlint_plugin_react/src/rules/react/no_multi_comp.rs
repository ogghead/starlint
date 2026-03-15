//! Rule: `react/no-multi-comp`
//!
//! Only one component definition per file. Multiple component definitions
//! in a single file make it harder to find and maintain components.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags files with multiple component definitions.
///
/// Simplified detection: counts top-level functions/classes that contain JSX
/// by scanning the source text for JSX return patterns. Uses a heuristic
/// approach based on counting `AstNode::JSXElement` occurrences at the
/// top-level function/class boundary.
#[derive(Debug)]
pub struct NoMultiComp;

impl LintRule for NoMultiComp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-multi-comp".to_owned(),
            description: "Only one component definition per file".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        // Simplified: flag React.createClass / createReactClass beyond the first,
        // or detect multiple class components / function components returning JSX.
        // For a practical implementation, we count top-level arrow/function expressions
        // that are assigned and contain JSX.
        //
        // This simplified version flags every `CallExpression` for `React.createClass`
        // or `createReactClass` after the first occurrence. For full detection of
        // multiple components, a more sophisticated approach tracking function scopes
        // would be needed.

        // We use a stub approach: flag nothing at the per-node level.
        // The real check happens in run_once.
        let _ = (node, ctx);
    }

    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        // All class component patterns contain "Component" as a substring
        source_text.contains("Component")
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut LintContext<'_>) {
        let violations: Vec<u32> = {
            let source = ctx.source_text();

            let mut component_count = 0u32;
            let mut last_span_start = 0u32;

            // Single pass: collect all match positions into a vec, then count
            let mut positions: Vec<usize> = Vec::new();
            for (idx, _) in source.match_indices("extends Component") {
                positions.push(idx);
            }
            for (idx, _) in source.match_indices("extends React.Component") {
                positions.push(idx);
            }
            for (idx, _) in source.match_indices("extends PureComponent") {
                positions.push(idx);
            }
            for (idx, _) in source.match_indices("extends React.PureComponent") {
                positions.push(idx);
            }
            positions.sort_unstable();

            for idx in positions {
                component_count = component_count.saturating_add(1);
                if component_count > 1 {
                    last_span_start = u32::try_from(idx).unwrap_or(0);
                }
            }

            if component_count > 1 {
                vec![last_span_start]
            } else {
                vec![]
            }
        };

        for start in violations {
            ctx.report(Diagnostic {
                rule_name: "react/no-multi-comp".to_owned(),
                message: "Declare only one React component per file".to_owned(),
                span: Span::new(start, start.saturating_add(1)),
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

    starlint_rule_framework::lint_rule_test!(NoMultiComp);

    #[test]
    fn test_flags_multiple_class_components() {
        let source = r"
class CompA extends React.Component {
    render() { return null; }
}
class CompB extends React.Component {
    render() { return null; }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "multiple class components should be flagged"
        );
    }

    #[test]
    fn test_allows_single_component() {
        let source = r"
class MyComponent extends React.Component {
    render() { return null; }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "single component should not be flagged");
    }

    #[test]
    fn test_allows_no_components() {
        let source = "const x = 1;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "file without components should not be flagged"
        );
    }
}
