//! Single-pass AST traversal with multi-rule dispatch.
//!
//! Traverses the oxc AST once, dispatching each node to all registered rules
//! that implement [`NativeRule::run`]. A missed pattern match is essentially
//! free (single branch prediction).

use std::path::Path;

use oxc_ast::ast::Program;
use oxc_ast::visit::walk;
use oxc_ast::{AstKind, Visit};

use crate::rule::{NativeLintContext, NativeRule};
use starlint_plugin_sdk::diagnostic::Diagnostic;

/// Traverse the AST and dispatch to all active rules.
///
/// Returns diagnostics from all rules combined.
pub fn traverse_and_lint<'a>(
    program: &Program<'a>,
    rules: &[Box<dyn NativeRule>],
    source_text: &'a str,
    file_path: &'a Path,
) -> Vec<Diagnostic> {
    let traversal_rules: Vec<&dyn NativeRule> =
        rules.iter().filter(|r| r.needs_traversal()).map(|r| r.as_ref()).collect();
    let once_rules: Vec<&dyn NativeRule> =
        rules.iter().filter(|r| !r.needs_traversal()).map(|r| r.as_ref()).collect();

    let mut all_diagnostics = Vec::new();

    // Run traversal-based rules via single-pass visitor.
    if !traversal_rules.is_empty() {
        let mut visitor = RuleDispatchVisitor {
            rules: &traversal_rules,
            source_text,
            file_path,
            diagnostics: Vec::new(),
        };
        visitor.visit_program(program);
        all_diagnostics.append(&mut visitor.diagnostics);
    }

    // Run file-level rules (run_once).
    for rule in &once_rules {
        let mut ctx = NativeLintContext::new(source_text, file_path);
        rule.run_once(&mut ctx);
        all_diagnostics.extend(ctx.into_diagnostics());
    }

    // Run run_once for traversal rules too (they may have both).
    for rule in &traversal_rules {
        let mut ctx = NativeLintContext::new(source_text, file_path);
        rule.run_once(&mut ctx);
        all_diagnostics.extend(ctx.into_diagnostics());
    }

    all_diagnostics
}

/// Visitor that dispatches AST nodes to multiple rules.
struct RuleDispatchVisitor<'a, 'rules> {
    /// Active rules to dispatch to.
    rules: &'rules [&'rules dyn NativeRule],
    /// Source text.
    source_text: &'a str,
    /// File path.
    file_path: &'a Path,
    /// Collected diagnostics.
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Visit<'a> for RuleDispatchVisitor<'a, '_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        for rule in self.rules {
            let mut ctx = NativeLintContext::new(self.source_text, self.file_path);
            rule.run(&kind, &mut ctx);
            self.diagnostics.extend(ctx.into_diagnostics());
        }
    }

    fn visit_program(&mut self, program: &Program<'a>) {
        walk::walk_program(self, program);
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
        let parsed = result.ok();
        assert!(parsed.is_some(), "should have parse result");
        let parsed = parsed.unwrap_or_else(|| {
            // This won't be reached due to the assertion above.
            let alloc = Allocator::default();
            let r = parse_file(&alloc, "", Path::new("empty.js"));
            r.ok().unwrap_or_else(|| std::process::exit(1))
        });

        let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebuggerRule)];
        let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
        assert_eq!(diags.len(), 1, "should find one debugger statement");
    }

    #[test]
    fn test_traverse_clean_file() {
        let allocator = Allocator::default();
        let source = "const x = 1;";
        let result = parse_file(&allocator, source, Path::new("test.js"));
        let parsed = result.ok();
        assert!(parsed.is_some(), "should have parse result");
        // Use a safe approach to handle the unwrap
        if let Some(parsed) = parsed {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDebuggerRule)];
            let diags = traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"));
            assert_eq!(diags.len(), 0, "clean file should have no diagnostics");
        }
    }
}
