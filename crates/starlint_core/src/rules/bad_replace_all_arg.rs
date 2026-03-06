//! Rule: `bad-replace-all-arg` (OXC)
//!
//! Catch `.replaceAll()` called with a regex argument that lacks the global
//! flag. `String.prototype.replaceAll` throws a `TypeError` at runtime if
//! given a regex without the `g` flag.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.replaceAll(/regex/)` without the global flag.
#[derive(Debug)]
pub struct BadReplaceAllArg;

impl NativeRule for BadReplaceAllArg {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "bad-replace-all-arg".to_owned(),
            description: "Catch `.replaceAll()` with a regex missing the `g` flag".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for .replaceAll() calls
        let is_replace_all = matches!(
            &call.callee,
            Expression::StaticMemberExpression(member) if member.property.name.as_str() == "replaceAll"
        );

        if !is_replace_all {
            return;
        }

        // Check if the first argument is a regex literal without the `g` flag
        let Some(Argument::RegExpLiteral(re)) = call.arguments.first() else {
            return;
        };

        if !re.regex.flags.contains(oxc_ast::ast::RegExpFlags::G) {
            // Build fix: insert `g` flag by replacing regex span with regex+g
            #[allow(clippy::as_conversions)]
            let fix = ctx
                .source_text()
                .get(re.span.start as usize..re.span.end as usize)
                .map(|regex_text| {
                    // Regex source looks like `/pattern/flags`
                    // Insert `g` at the end
                    let replacement = format!("{regex_text}g");
                    Fix {
                        kind: FixKind::SafeFix,
                        message: "Add the `g` flag".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(re.span.start, re.span.end),
                            replacement,
                        }],
                        is_snippet: false,
                    }
                });

            ctx.report(Diagnostic {
                rule_name: "bad-replace-all-arg".to_owned(),
                message: "`.replaceAll()` with a regex requires the global (`g`) flag — \
                     this will throw a TypeError at runtime"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
                severity: Severity::Error,
                help: Some("Add the `g` flag to the regex".to_owned()),
                fix,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(BadReplaceAllArg)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_regex_without_global() {
        let diags = lint("'hello'.replaceAll(/l/, 'r');");
        assert_eq!(
            diags.len(),
            1,
            "replaceAll with regex without g flag should be flagged"
        );
    }

    #[test]
    fn test_allows_regex_with_global() {
        let diags = lint("'hello'.replaceAll(/l/g, 'r');");
        assert!(
            diags.is_empty(),
            "replaceAll with regex with g flag should not be flagged"
        );
    }

    #[test]
    fn test_allows_string_argument() {
        let diags = lint("'hello'.replaceAll('l', 'r');");
        assert!(
            diags.is_empty(),
            "replaceAll with string argument should not be flagged"
        );
    }
}
