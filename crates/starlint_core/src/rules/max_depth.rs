//! Rule: `max-depth`
//!
//! Enforce a maximum depth of nested control-flow blocks. Deeply nested
//! code is harder to read and understand — prefer extracting into functions.

use std::sync::RwLock;

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default maximum nesting depth.
const DEFAULT_MAX: u32 = 4;

/// Enforces a maximum depth of nested control-flow blocks.
#[derive(Debug)]
pub struct MaxDepth {
    /// Maximum nesting depth allowed.
    max: u32,
    /// Current nesting depth (tracked during traversal).
    depth: RwLock<u32>,
}

impl MaxDepth {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max: DEFAULT_MAX,
            depth: RwLock::new(0),
        }
    }
}

impl Default for MaxDepth {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if the AST node introduces a new nesting level.
const fn is_nesting_node(kind: &AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::IfStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::SwitchStatement(_)
            | AstKind::TryStatement(_)
    )
}

impl NativeRule for MaxDepth {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-depth".to_owned(),
            description: "Enforce a maximum depth of nested blocks".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            self.max = u32::try_from(n).unwrap_or(DEFAULT_MAX);
        }
        Ok(())
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::DoWhileStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::IfStatement,
            AstType::SwitchStatement,
            AstType::TryStatement,
            AstType::WhileStatement,
        ])
    }

    fn leave_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::DoWhileStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::IfStatement,
            AstType::SwitchStatement,
            AstType::TryStatement,
            AstType::WhileStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        if !is_nesting_node(kind) {
            return;
        }

        let Ok(mut depth_guard) = self.depth.write() else {
            return;
        };
        *depth_guard = depth_guard.saturating_add(1);
        let current = *depth_guard;
        drop(depth_guard);

        if current > self.max {
            let span = match kind {
                AstKind::IfStatement(s) => s.span,
                AstKind::ForStatement(s) => s.span,
                AstKind::ForInStatement(s) => s.span,
                AstKind::ForOfStatement(s) => s.span,
                AstKind::WhileStatement(s) => s.span,
                AstKind::DoWhileStatement(s) => s.span,
                AstKind::SwitchStatement(s) => s.span,
                AstKind::TryStatement(s) => s.span,
                _ => return,
            };
            ctx.report_warning(
                "max-depth",
                &format!(
                    "Blocks are nested too deeply ({current}). Maximum allowed is {}",
                    self.max
                ),
                Span::new(span.start, span.end),
            );
        }
    }

    fn leave(&self, kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        if !is_nesting_node(kind) {
            return;
        }

        if let Ok(mut depth_guard) = self.depth.write() {
            *depth_guard = depth_guard.saturating_sub(1);
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

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MaxDepth {
                max,
                depth: RwLock::new(0),
            })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_shallow_nesting() {
        let source = "if (true) { console.log(1); }";
        let diags = lint_with_max(source, 2);
        assert!(diags.is_empty(), "shallow nesting should not be flagged");
    }

    #[test]
    fn test_flags_deep_nesting() {
        let source = "if (a) { if (b) { if (c) { console.log(1); } } }";
        let diags = lint_with_max(source, 2);
        assert_eq!(diags.len(), 1, "third level should be flagged");
    }

    #[test]
    fn test_allows_at_limit() {
        let source = "if (a) { if (b) { console.log(1); } }";
        let diags = lint_with_max(source, 2);
        assert!(diags.is_empty(), "nesting at limit should not be flagged");
    }

    #[test]
    fn test_flags_loop_nesting() {
        let source = "for (var i = 0; i < 10; i++) { while (true) { if (a) { break; } } }";
        let diags = lint_with_max(source, 2);
        assert_eq!(diags.len(), 1, "deeply nested loops should be flagged");
    }

    #[test]
    fn test_sequential_not_nested() {
        let source = "if (a) {} if (b) {} if (c) {}";
        let diags = lint_with_max(source, 1);
        assert!(
            diags.is_empty(),
            "sequential blocks should not count as nested"
        );
    }
}
