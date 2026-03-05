//! Rule: `react/rules-of-hooks`
//!
//! Error when hooks are called conditionally or in loops.
//! Simplified: flags `use*()` calls inside if/for/while blocks by checking
//! source text position for enclosing control flow.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags React hook calls (`use*()`) that appear inside control flow
/// statements (if, for, while, ternary), which violates the Rules of Hooks.
#[derive(Debug)]
pub struct RulesOfHooks {
    /// Track depth of control flow nesting. When > 0, we're inside a
    /// conditional/loop block.
    control_flow_depth: std::sync::atomic::AtomicU32,
}

impl Default for RulesOfHooks {
    fn default() -> Self {
        Self {
            control_flow_depth: std::sync::atomic::AtomicU32::new(0),
        }
    }
}

impl RulesOfHooks {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Check if a callee name looks like a React hook (starts with "use" followed
/// by an uppercase letter or is exactly "use").
fn is_hook_name(name: &str) -> bool {
    if name == "use" {
        return true;
    }
    if let Some(rest) = name.strip_prefix("use") {
        rest.as_bytes()
            .first()
            .is_some_and(|&b| b.is_ascii_uppercase())
    } else {
        false
    }
}

impl NativeRule for RulesOfHooks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/rules-of-hooks".to_owned(),
            description: "Hooks must be called at the top level of a function component".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::CallExpression,
            AstType::ConditionalExpression,
            AstType::DoWhileStatement,
            AstType::ForStatement,
            AstType::IfStatement,
            AstType::WhileStatement,
        ])
    }

    fn leave_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[
            AstType::CallExpression,
            AstType::ConditionalExpression,
            AstType::DoWhileStatement,
            AstType::ForStatement,
            AstType::IfStatement,
            AstType::WhileStatement,
        ])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        use std::sync::atomic::Ordering;
        match kind {
            AstKind::IfStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::ConditionalExpression(_) => {
                self.control_flow_depth.fetch_add(1, Ordering::Relaxed);
            }
            AstKind::CallExpression(call) => {
                if self.control_flow_depth.load(Ordering::Relaxed) == 0 {
                    return;
                }

                let hook_name = match &call.callee {
                    Expression::Identifier(id) => id.name.as_str(),
                    // React.useXxx()
                    Expression::StaticMemberExpression(member) => member.property.name.as_str(),
                    _ => return,
                };

                if is_hook_name(hook_name) {
                    ctx.report(Diagnostic {
                        rule_name: "react/rules-of-hooks".to_owned(),
                        message: format!(
                            "React hook `{hook_name}` is called conditionally — hooks must be called at the top level"
                        ),
                        span: Span::new(call.span.start, call.span.end),
                        severity: Severity::Error,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }

    fn leave(&self, kind: &AstKind<'_>, _ctx: &mut NativeLintContext<'_>) {
        use std::sync::atomic::Ordering;
        match kind {
            AstKind::IfStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::ConditionalExpression(_) => {
                self.control_flow_depth.fetch_sub(1, Ordering::Relaxed);
            }
            _ => {}
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(RulesOfHooks::new())];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_hook_in_if_block() {
        let source = "function Comp() { if (cond) { useState(0); } return null; }";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "hook inside if block should be flagged");
    }

    #[test]
    fn test_flags_hook_in_for_loop() {
        let source = "function Comp() { for (let i = 0; i < 10; i++) { useEffect(() => {}); } return null; }";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "hook inside for loop should be flagged");
    }

    #[test]
    fn test_allows_hook_at_top_level() {
        let source = "function Comp() { const [x, setX] = useState(0); return <div>{x}</div>; }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "hook at top level of function should not be flagged"
        );
    }
}
