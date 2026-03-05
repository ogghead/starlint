//! Rule: `jest/prefer-lowercase-title`
//!
//! Suggest lowercase titles for `it`/`test` calls. Consistent lowercase
//! titles read more naturally as sentences: "it should work" vs
//! "it Should work".

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `it`/`test` calls with uppercase-starting titles.
#[derive(Debug)]
pub struct PreferLowercaseTitle;

impl NativeRule for PreferLowercaseTitle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "jest/prefer-lowercase-title".to_owned(),
            description: "Suggest lowercase titles for `it`/`test` calls".to_owned(),
            category: Category::Suggestion,
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

        // Must be `it(...)` or `test(...)` — not `describe`
        let callee_name = match &call.callee {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };
        if callee_name != "it" && callee_name != "test" {
            return;
        }

        // First argument must be a string literal
        let Some(first_arg) = call.arguments.first() else {
            return;
        };
        let Argument::StringLiteral(title) = first_arg else {
            return;
        };
        let title_str = title.value.as_str();

        // Check if the first character is uppercase
        let Some(first_char) = title_str.chars().next() else {
            return;
        };
        if first_char.is_uppercase() {
            // Replace just the first character inside the string literal
            // title.span includes quotes, so the first content char is at start+1
            let char_start = title.span.start.saturating_add(1);
            let char_end =
                char_start.saturating_add(u32::try_from(first_char.len_utf8()).unwrap_or(1));
            let lowered: String = first_char.to_lowercase().collect();
            ctx.report(Diagnostic {
                rule_name: "jest/prefer-lowercase-title".to_owned(),
                message: "Test titles should start with a lowercase letter".to_owned(),
                span: Span::new(title.span.start, title.span.end),
                severity: Severity::Warning,
                help: Some("Lowercase the first letter of the test title".to_owned()),
                fix: Some(Fix {
                    message: "Lowercase first letter".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(char_start, char_end),
                        replacement: lowered,
                    }],
                }),
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferLowercaseTitle)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_uppercase_title() {
        let diags = lint(r"test('Should work', () => {});");
        assert_eq!(
            diags.len(),
            1,
            "uppercase-starting test title should be flagged"
        );
    }

    #[test]
    fn test_allows_lowercase_title() {
        let diags = lint(r"test('should work', () => {});");
        assert!(
            diags.is_empty(),
            "lowercase-starting test title should not be flagged"
        );
    }

    #[test]
    fn test_allows_describe_uppercase() {
        let diags = lint(r"describe('MyComponent', () => {});");
        assert!(
            diags.is_empty(),
            "`describe` with uppercase title should not be flagged"
        );
    }
}
