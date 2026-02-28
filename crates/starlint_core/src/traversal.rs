//! Single-pass AST traversal with multi-rule dispatch.
//!
//! Traverses the oxc AST once, dispatching each node to all registered rules
//! that implement [`NativeRule::run`]. A missed pattern match is essentially
//! free (single branch prediction).

use std::path::Path;

use oxc_ast::AstKind;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_semantic::Semantic;

use crate::rule::{NativeLintContext, NativeRule};
use starlint_plugin_sdk::diagnostic::Diagnostic;

/// Traverse the AST and dispatch to all active rules.
///
/// Returns diagnostics from all rules combined.
/// For rules that need semantic analysis, use [`traverse_and_lint_with_semantic`].
pub fn traverse_and_lint<'a>(
    program: &Program<'a>,
    rules: &[Box<dyn NativeRule>],
    source_text: &'a str,
    file_path: &'a Path,
) -> Vec<Diagnostic> {
    traverse_and_lint_with_semantic(program, rules, source_text, file_path, None)
}

/// Traverse the AST and dispatch to all active rules, with optional semantic data.
///
/// When `semantic` is `Some`, rules can access scope/symbol information via
/// [`NativeLintContext::semantic()`].
pub fn traverse_and_lint_with_semantic<'a>(
    program: &Program<'a>,
    rules: &[Box<dyn NativeRule>],
    source_text: &'a str,
    file_path: &'a Path,
    semantic: Option<&'a Semantic<'a>>,
) -> Vec<Diagnostic> {
    let traversal_rules: Vec<&dyn NativeRule> = rules
        .iter()
        .filter(|r| r.needs_traversal())
        .map(std::convert::AsRef::as_ref)
        .collect();
    let once_rules: Vec<&dyn NativeRule> = rules
        .iter()
        .filter(|r| !r.needs_traversal())
        .map(std::convert::AsRef::as_ref)
        .collect();

    let mut all_diagnostics = Vec::new();

    // Run traversal-based rules via single-pass visitor.
    if !traversal_rules.is_empty() {
        let ctx = match semantic {
            Some(sem) => NativeLintContext::with_semantic(source_text, file_path, sem),
            None => NativeLintContext::new(source_text, file_path),
        };
        let mut visitor = RuleDispatchVisitor {
            rules: &traversal_rules,
            ctx,
        };
        visitor.visit_program(program);
        all_diagnostics.extend(visitor.ctx.into_diagnostics());
    }

    // Run file-level rules (run_once).
    for rule in &once_rules {
        let mut ctx = match semantic {
            Some(sem) => NativeLintContext::with_semantic(source_text, file_path, sem),
            None => NativeLintContext::new(source_text, file_path),
        };
        rule.run_once(&mut ctx);
        all_diagnostics.extend(ctx.into_diagnostics());
    }

    // Run run_once for traversal rules too (they may have both).
    for rule in &traversal_rules {
        let mut ctx = match semantic {
            Some(sem) => NativeLintContext::with_semantic(source_text, file_path, sem),
            None => NativeLintContext::new(source_text, file_path),
        };
        rule.run_once(&mut ctx);
        all_diagnostics.extend(ctx.into_diagnostics());
    }

    all_diagnostics
}

/// Visitor that dispatches AST nodes to multiple rules.
///
/// Uses a single shared [`NativeLintContext`] for the entire traversal to
/// avoid allocating a new `Vec<Diagnostic>` per node per rule.
struct RuleDispatchVisitor<'a, 'rules> {
    /// Active rules to dispatch to.
    rules: &'rules [&'rules dyn NativeRule],
    /// Shared lint context — all rules push diagnostics into this.
    ctx: NativeLintContext<'a>,
}

impl<'a> Visit<'a> for RuleDispatchVisitor<'a, '_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        for rule in self.rules {
            rule.run(&kind, &mut self.ctx);
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        for rule in self.rules {
            rule.leave(&kind, &mut self.ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    use oxc_allocator::Allocator;
    use starlint_plugin_sdk::diagnostic::Span;
    use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

    use crate::parser::parse_file;

    /// A test rule that flags all `DebuggerStatement` nodes.
    #[derive(Debug)]
    struct NoDebuggerRule;

    impl NativeRule for NoDebuggerRule {
        fn meta(&self) -> RuleMeta {
            RuleMeta {
                name: "no-debugger".to_owned(),
                description: "Disallow debugger statements".to_owned(),
                category: Category::Correctness,
                default_severity: starlint_plugin_sdk::diagnostic::Severity::Error,
                fix_kind: FixKind::SafeFix,
            }
        }

        fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
            if let AstKind::DebuggerStatement(stmt) = kind {
                ctx.report_error(
                    "no-debugger",
                    "Unexpected `debugger` statement",
                    Span::new(stmt.span.start, stmt.span.end),
                );
            }
        }
    }

    #[test]
    fn test_traverse_finds_debugger() {
        let allocator = Allocator::default();
        let source = "debugger;\nconst x = 1;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        assert!(result.is_ok(), "parse should succeed");
        if let Ok(parsed) = result {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebuggerRule)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 1, "should find one debugger statement");
        }
    }

    #[test]
    fn test_traverse_clean_file() {
        let allocator = Allocator::default();
        let source = "const x = 1;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        if let Ok(parsed) = result {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebuggerRule)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 0, "clean file should have no diagnostics");
        }
    }
}
