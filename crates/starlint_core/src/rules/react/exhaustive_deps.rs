//! Rule: `react/exhaustive-deps`
//!
//! Warn about missing dependency arrays in React hooks.
//! Simplified: flags when `useEffect`, `useCallback`, or `useMemo` is called
//! without a dependency array (second argument).

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags calls to `useEffect`, `useCallback`, or `useMemo` that are missing
/// their dependency array argument (second parameter).
#[derive(Debug)]
pub struct ExhaustiveDeps;

/// Hook names that require a dependency array as their second argument.
const HOOKS_WITH_DEPS: &[&str] = &["useEffect", "useCallback", "useMemo"];

impl NativeRule for ExhaustiveDeps {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/exhaustive-deps".to_owned(),
            description: "Warn about missing dependency arrays in hooks".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let hook_name = match &call.callee {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        if !HOOKS_WITH_DEPS.contains(&hook_name) {
            return;
        }

        // These hooks require at least 2 arguments: the callback and the dependency array
        if call.arguments.len() < 2 {
            ctx.report_warning(
                "react/exhaustive-deps",
                &format!(
                    "`{hook_name}` is missing its dependency array — this will run on every render"
                ),
                Span::new(call.span.start, call.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ExhaustiveDeps)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_use_effect_without_deps() {
        let source = "useEffect(() => { console.log('hi'); });";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "useEffect without deps should be flagged");
    }

    #[test]
    fn test_allows_use_effect_with_deps() {
        let source = "useEffect(() => { console.log('hi'); }, []);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "useEffect with deps should not be flagged"
        );
    }

    #[test]
    fn test_flags_use_callback_without_deps() {
        let source = "const fn = useCallback(() => {});";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "useCallback without deps should be flagged");
    }

    #[test]
    fn test_allows_use_memo_with_deps() {
        let source = "const val = useMemo(() => compute(), [compute]);";
        let diags = lint(source);
        assert!(diags.is_empty(), "useMemo with deps should not be flagged");
    }
}
