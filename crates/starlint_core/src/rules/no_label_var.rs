//! Rule: `no-label-var`
//!
//! Disallow labels that share a name with a variable. This can lead to
//! confusion about which entity is being referenced.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags labels that shadow variable names (simplified: flags labels
/// whose name matches a common variable pattern).
#[derive(Debug)]
pub struct NoLabelVar;

impl NativeRule for NoLabelVar {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-label-var".to_owned(),
            description: "Disallow labels that share a name with a variable".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::LabeledStatement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::LabeledStatement(labeled) = kind else {
            return;
        };

        let label_name = labeled.label.name.as_str();

        // Check if the label name appears as a variable declaration in the source
        // This is a simplified check that scans for `var/let/const label_name`
        let source = ctx.source_text();
        let var_pattern = format!("var {label_name}");
        let let_pattern = format!("let {label_name}");
        let const_pattern = format!("const {label_name}");
        let span_start = labeled.label.span.start;
        let span_end = labeled.label.span.end;

        let has_var = source.contains(&var_pattern)
            || source.contains(&let_pattern)
            || source.contains(&const_pattern);

        if has_var {
            ctx.report(Diagnostic {
                rule_name: "no-label-var".to_owned(),
                message: format!("Found identifier `{label_name}` with the same name as a label"),
                span: Span::new(span_start, span_end),
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoLabelVar)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_label_matching_var() {
        let diags = lint("var x = 1; x: while(true) { break x; }");
        assert_eq!(
            diags.len(),
            1,
            "label sharing name with variable should be flagged"
        );
    }

    #[test]
    fn test_allows_label_not_matching_var() {
        let diags = lint("var x = 1; loop1: while(true) { break loop1; }");
        assert!(
            diags.is_empty(),
            "label not matching any variable should not be flagged"
        );
    }
}
