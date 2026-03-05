//! Rule: `max-statements` (eslint)
//!
//! Flag functions with too many statements. Functions with many statements
//! are harder to understand and should be broken into smaller pieces.

use std::sync::RwLock;

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default maximum number of statements per function.
const DEFAULT_MAX: u32 = 10;

/// Flags functions with too many statements.
#[derive(Debug)]
pub struct MaxStatements {
    /// Maximum number of statements allowed per function.
    max: RwLock<u32>,
}

impl MaxStatements {
    /// Create a new `MaxStatements` rule with the default threshold.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max: RwLock::new(DEFAULT_MAX),
        }
    }
}

impl Default for MaxStatements {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRule for MaxStatements {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-statements".to_owned(),
            description: "Enforce a maximum number of statements per function".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            let val = u32::try_from(n).unwrap_or(DEFAULT_MAX);
            if let Ok(mut guard) = self.max.write() {
                *guard = val;
            }
        }
        Ok(())
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ArrowFunctionExpression, AstType::Function])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let threshold = self.max.read().map_or(DEFAULT_MAX, |g| *g);

        let (stmt_count, span, name) = match kind {
            AstKind::Function(f) => {
                let Some(body) = &f.body else { return };
                let count = u32::try_from(body.statements.len()).unwrap_or(0);
                let fn_name =
                    f.id.as_ref()
                        .map_or_else(|| "(anonymous)".to_owned(), |id| id.name.to_string());
                (count, f.span, fn_name)
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                let count = u32::try_from(arrow.body.statements.len()).unwrap_or(0);
                (count, arrow.span, "(arrow function)".to_owned())
            }
            _ => return,
        };

        if stmt_count > threshold {
            ctx.report(Diagnostic {
                rule_name: "max-statements".to_owned(),
                message: format!(
                    "Function '{name}' has too many statements ({stmt_count}). Maximum allowed is {threshold}"
                ),
                span: Span::new(span.start, span.end),
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
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MaxStatements {
                max: RwLock::new(max),
            })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_few_statements() {
        let source = "function foo() { var a = 1; var b = 2; var c = 3; }";
        let diags = lint_with_max(source, 10);
        assert!(
            diags.is_empty(),
            "function with few statements should not be flagged"
        );
    }

    #[test]
    fn test_flags_many_statements() {
        let source = r"function foo() {
            var a = 1;
            var b = 2;
            var c = 3;
            var d = 4;
            var e = 5;
            var f = 6;
            var g = 7;
            var h = 8;
            var i = 9;
            var j = 10;
            var k = 11;
        }";
        let diags = lint_with_max(source, 10);
        assert_eq!(
            diags.len(),
            1,
            "function with many statements should be flagged"
        );
    }

    #[test]
    fn test_allows_at_limit() {
        let source = r"function foo() {
            var a = 1;
            var b = 2;
            var c = 3;
        }";
        let diags = lint_with_max(source, 3);
        assert!(
            diags.is_empty(),
            "function at the limit should not be flagged"
        );
    }

    #[test]
    fn test_arrow_function_flagged() {
        let source = r"const foo = () => {
            var a = 1;
            var b = 2;
            var c = 3;
            var d = 4;
        };";
        let diags = lint_with_max(source, 3);
        assert_eq!(
            diags.len(),
            1,
            "arrow function with too many statements should be flagged"
        );
    }
}
