//! Rule: `prefer-string-trim-start-end`
//!
//! Prefer `.trimStart()` / `.trimEnd()` over the deprecated
//! `.trimLeft()` / `.trimRight()` aliases.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `.trimLeft()` and `.trimRight()` — use `.trimStart()` / `.trimEnd()`.
#[derive(Debug)]
pub struct PreferStringTrimStartEnd;

impl NativeRule for PreferStringTrimStartEnd {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-string-trim-start-end".to_owned(),
            description: "Prefer `.trimStart()` / `.trimEnd()` over `.trimLeft()` / `.trimRight()`"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method = member.property.name.as_str();
        let replacement_method = match method {
            "trimLeft" => "trimStart",
            "trimRight" => "trimEnd",
            _ => return,
        };

        // Only flag zero-argument calls.
        if !call.arguments.is_empty() {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "prefer-string-trim-start-end".to_owned(),
            message: format!("Use `.{replacement_method}()` instead of `.{method}()`"),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some(format!(
                "Replace `.{method}()` with `.{replacement_method}()`"
            )),
            fix: Some(Fix {
                message: format!("Replace `.{method}` with `.{replacement_method}`"),
                edits: vec![Edit {
                    span: Span::new(member.property.span.start, member.property.span.end),
                    replacement: replacement_method.to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferStringTrimStartEnd)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_trim_left() {
        let diags = lint("str.trimLeft();");
        assert_eq!(diags.len(), 1, "should flag .trimLeft()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("trimStart"),
            "fix should replace with trimStart"
        );
    }

    #[test]
    fn test_flags_trim_right() {
        let diags = lint("str.trimRight();");
        assert_eq!(diags.len(), 1, "should flag .trimRight()");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("trimEnd"),
            "fix should replace with trimEnd"
        );
    }

    #[test]
    fn test_allows_trim_start() {
        let diags = lint("str.trimStart();");
        assert!(diags.is_empty(), ".trimStart() should not be flagged");
    }

    #[test]
    fn test_allows_trim_end() {
        let diags = lint("str.trimEnd();");
        assert!(diags.is_empty(), ".trimEnd() should not be flagged");
    }
}
