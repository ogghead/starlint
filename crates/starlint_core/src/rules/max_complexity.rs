//! Rule: `max-complexity`
//!
//! Enforce a maximum cyclomatic complexity per function. Cyclomatic complexity
//! measures the number of linearly independent paths through a function.
//! High complexity correlates with bugs and maintainability problems.

use std::sync::RwLock;

use oxc_ast::AstKind;
use oxc_ast::ast::LogicalOperator;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default maximum cyclomatic complexity allowed per function.
const DEFAULT_THRESHOLD: u32 = 20;

/// Tracks complexity for a single function scope.
#[derive(Debug)]
struct FunctionScope {
    /// Function name for diagnostic messages.
    name: String,
    /// Function span for diagnostic location.
    span: Span,
    /// Running cyclomatic complexity count (starts at 1 — the base path).
    complexity: u32,
}

/// Enforces a maximum cyclomatic complexity per function.
///
/// Counts decision points: `if`, `? :`, `case` (non-default), `for`,
/// `for-in`, `for-of`, `while`, `do-while`, `catch`, `&&`, `||`.
/// The nullish coalescing operator `??` is not counted, matching `ESLint`.
#[derive(Debug)]
pub struct MaxComplexity {
    /// Maximum complexity before a warning is emitted.
    threshold: u32,
    /// Stack of open function scopes for tracking complexity.
    scopes: RwLock<Vec<FunctionScope>>,
}

impl Default for MaxComplexity {
    fn default() -> Self {
        Self::new()
    }
}

impl MaxComplexity {
    /// Create a new rule with the default threshold.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
            scopes: RwLock::new(Vec::new()),
        }
    }

    /// Increment the complexity of the innermost function scope by 1.
    fn increment_complexity(&self) {
        if let Ok(mut guard) = self.scopes.write() {
            if let Some(scope) = guard.last_mut() {
                scope.complexity = scope.complexity.saturating_add(1);
            }
        }
    }
}

impl NativeRule for MaxComplexity {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-complexity".to_owned(),
            description: "Enforce a maximum cyclomatic complexity per function".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(threshold) = config.get("threshold").and_then(serde_json::Value::as_u64) {
            self.threshold = u32::try_from(threshold)
                .map_err(|err| format!("threshold must fit in u32: {err}"))?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::CatchClause,
            AstType::ConditionalExpression,
            AstType::DoWhileStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::Function,
            AstType::IfStatement,
            AstType::LogicalExpression,
            AstType::SwitchCase,
            AstType::WhileStatement,
        ])
    }

    fn leave_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::ArrowFunctionExpression,
            AstType::CatchClause,
            AstType::ConditionalExpression,
            AstType::DoWhileStatement,
            AstType::ForInStatement,
            AstType::ForOfStatement,
            AstType::ForStatement,
            AstType::Function,
            AstType::IfStatement,
            AstType::LogicalExpression,
            AstType::SwitchCase,
            AstType::WhileStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        match kind {
            // Push a new scope for function boundaries.
            AstKind::Function(f) => {
                let name =
                    f.id.as_ref()
                        .map_or_else(|| "<anonymous>".to_owned(), |id| id.name.to_string());
                if let Ok(mut guard) = self.scopes.write() {
                    guard.push(FunctionScope {
                        name,
                        span: Span::new(f.span.start, f.span.end),
                        complexity: 1,
                    });
                }
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if let Ok(mut guard) = self.scopes.write() {
                    guard.push(FunctionScope {
                        name: "<anonymous>".to_owned(),
                        span: Span::new(arrow.span.start, arrow.span.end),
                        complexity: 1,
                    });
                }
            }

            // Decision points: each adds +1 complexity.
            AstKind::IfStatement(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::CatchClause(_) => {
                self.increment_complexity();
            }

            // Switch case: only non-default cases add complexity.
            AstKind::SwitchCase(case) => {
                if case.test.is_some() {
                    self.increment_complexity();
                }
            }

            // Logical operators: && and || add complexity, ?? does not.
            AstKind::LogicalExpression(expr) => {
                if matches!(expr.operator, LogicalOperator::And | LogicalOperator::Or) {
                    self.increment_complexity();
                }
            }

            _ => {}
        }
    }

    fn leave(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let should_pop = matches!(
            kind,
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
        );
        if !should_pop {
            return;
        }

        let Ok(mut guard) = self.scopes.write() else {
            return;
        };
        let Some(scope) = guard.pop() else {
            return;
        };
        // Drop the lock before reporting to avoid holding it during allocation.
        drop(guard);

        if scope.complexity > self.threshold {
            ctx.report(Diagnostic {
                rule_name: "max-complexity".to_owned(),
                message: format!(
                    "Function `{}` has a cyclomatic complexity of {} (max allowed: {})",
                    scope.name, scope.complexity, self.threshold,
                ),
                span: scope.span,
                severity: Severity::Warning,
                help: Some("Consider refactoring into smaller functions".to_owned()),
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

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MaxComplexity::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    fn lint_with_threshold(source: &str, threshold: u32) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MaxComplexity {
                threshold,
                scopes: RwLock::new(Vec::new()),
            })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_simple_function_under_threshold() {
        let source = "function foo() { return 1; }";
        let diags = lint(source);
        assert!(diags.is_empty(), "simple function should not be flagged");
    }

    #[test]
    fn test_function_over_threshold() {
        // Build a function with complexity > 1 (threshold set to 1)
        let source = "function foo() { if (a) {} }";
        let diags = lint_with_threshold(source, 1);
        assert_eq!(diags.len(), 1, "should flag function over threshold");
        assert!(
            diags.first().is_some_and(|d| d.message.contains("foo")),
            "message should mention function name"
        );
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("complexity of 2")),
            "message should include actual complexity"
        );
    }

    #[test]
    fn test_arrow_function() {
        let source = "const fn = () => { if (a) {} if (b) {} };";
        let diags = lint_with_threshold(source, 2);
        assert_eq!(diags.len(), 1, "arrow function should be counted");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("<anonymous>")),
            "arrow should be anonymous"
        );
    }

    #[test]
    fn test_nested_functions_counted_independently() {
        // Outer has: base(1) + if(1) = 2
        // Inner has: base(1) + if(1) + if(1) = 3
        let source = r"
            function outer() {
                if (a) {}
                function inner() {
                    if (b) {}
                    if (c) {}
                }
            }
        ";
        let diags = lint_with_threshold(source, 2);
        assert_eq!(
            diags.len(),
            1,
            "only inner function should exceed threshold"
        );
        assert!(
            diags.first().is_some_and(|d| d.message.contains("inner")),
            "should flag inner, not outer"
        );
    }

    #[test]
    fn test_all_decision_points() {
        // Each decision point adds +1: if, ?:, case, for, for-in, for-of,
        // while, do-while, catch, &&, ||
        let source = r"
            function complex(a, b, c) {
                if (a) {}
                var x = a ? 1 : 2;
                switch (a) {
                    case 1: break;
                    case 2: break;
                    default: break;
                }
                for (var i = 0; i < 10; i++) {}
                for (var k in b) {}
                for (var v of c) {}
                while (a) { break; }
                do { break; } while (a);
                try {} catch (e) {}
                if (a && b) {}
                if (a || b) {}
            }
        ";
        // Base: 1
        // if: +1, ternary: +1, case 1: +1, case 2: +1, for: +1,
        // for-in: +1, for-of: +1, while: +1, do-while: +1, catch: +1,
        // if: +1, &&: +1, if: +1, ||: +1
        // Total: 1 + 14 = 15
        let diags = lint_with_threshold(source, 14);
        assert_eq!(diags.len(), 1, "should exceed threshold of 14");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("complexity of 15")),
            "complexity should be 15"
        );
    }

    #[test]
    fn test_default_case_not_counted() {
        // switch with only default: base(1) = 1
        let source = r"
            function foo(x) {
                switch (x) {
                    default: break;
                }
            }
        ";
        let diags = lint_with_threshold(source, 1);
        assert!(
            diags.is_empty(),
            "default case should not add to complexity"
        );
    }

    #[test]
    fn test_nullish_coalescing_not_counted() {
        // ?? should NOT add complexity
        let source = "function foo(a) { return a ?? 'default'; }";
        let diags = lint_with_threshold(source, 1);
        assert!(diags.is_empty(), "?? should not add to complexity");
    }

    #[test]
    fn test_logical_and_or_counted() {
        // base(1) + &&(1) + ||(1) = 3
        let source = "function foo(a, b, c) { return a && b || c; }";
        let diags = lint_with_threshold(source, 2);
        assert_eq!(diags.len(), 1, "&& and || should add to complexity");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("complexity of 3")),
            "complexity should be 3"
        );
    }

    #[test]
    fn test_top_level_code_not_counted() {
        // Decision points outside functions should not be counted
        let source = "if (true) {} for (;;) { break; }";
        let diags = lint(source);
        assert!(diags.is_empty(), "top-level code should not be flagged");
    }

    #[test]
    fn test_configure_threshold() {
        let mut rule = MaxComplexity::new();
        let config = serde_json::json!({ "threshold": 5 });
        assert!(rule.configure(&config).is_ok());
        assert_eq!(rule.threshold, 5, "threshold should be updated");
    }
}
