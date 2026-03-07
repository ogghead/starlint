//! Rule: `max-classes-per-file` (eslint)
//!
//! Flag files with too many class declarations. Having multiple classes
//! in one file often indicates that the file should be split.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Default maximum number of classes per file.
const DEFAULT_MAX: u32 = 1;

/// Flags files containing more than the allowed number of class declarations.
#[derive(Debug)]
pub struct MaxClassesPerFile {
    /// Maximum number of class declarations allowed per file.
    max: u32,
}

impl MaxClassesPerFile {
    /// Create a new `MaxClassesPerFile` rule with the default threshold.
    #[must_use]
    pub const fn new() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

impl Default for MaxClassesPerFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Count class declarations in a list of statements (non-recursive, top-level only).
fn count_classes(body: &[NodeId], ctx: &LintContext<'_>) -> u32 {
    let mut count: u32 = 0;
    for &stmt_id in body {
        let Some(stmt) = ctx.node(stmt_id) else {
            continue;
        };
        if matches!(stmt, AstNode::Class(_)) {
            count = count.saturating_add(1);
        }
        // Also check export default class — the declaration child may be a Class node
        if let AstNode::ExportDefaultDeclaration(export) = stmt {
            if matches!(ctx.node(export.declaration), Some(AstNode::Class(_))) {
                count = count.saturating_add(1);
            }
        }
        // Check exported class declarations
        if let AstNode::ExportNamedDeclaration(export) = stmt {
            if let Some(decl_id) = export.declaration {
                if matches!(ctx.node(decl_id), Some(AstNode::Class(_))) {
                    count = count.saturating_add(1);
                }
            }
        }
    }
    count
}

impl LintRule for MaxClassesPerFile {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-classes-per-file".to_owned(),
            description: "Enforce a maximum number of classes per file".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        // We use run() with AstNode::Program to access the program body
        true
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            self.max = u32::try_from(n).unwrap_or(DEFAULT_MAX);
        }
        Ok(())
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::Program])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::Program(program) = node else {
            return;
        };

        let class_count = count_classes(&program.body, ctx);

        if class_count > self.max {
            let source_len = u32::try_from(ctx.source_text().len()).unwrap_or(0);
            ctx.report(Diagnostic {
                rule_name: "max-classes-per-file".to_owned(),
                message: format!(
                    "File has too many classes ({class_count}). Maximum allowed is {}",
                    self.max
                ),
                span: Span::new(0, source_len),
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

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(MaxClassesPerFile { max })];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_one_class() {
        let diags = lint_with_max("class Foo {}", 1);
        assert!(
            diags.is_empty(),
            "single class should not be flagged with max 1"
        );
    }

    #[test]
    fn test_flags_two_classes() {
        let diags = lint_with_max("class Foo {}\nclass Bar {}", 1);
        assert_eq!(diags.len(), 1, "two classes should be flagged with max 1");
    }

    #[test]
    fn test_allows_two_classes_with_max_two() {
        let diags = lint_with_max("class Foo {}\nclass Bar {}", 2);
        assert!(
            diags.is_empty(),
            "two classes should not be flagged with max 2"
        );
    }

    #[test]
    fn test_no_classes() {
        let diags = lint_with_max("const x = 1;", 1);
        assert!(diags.is_empty(), "no classes should not be flagged");
    }
}
