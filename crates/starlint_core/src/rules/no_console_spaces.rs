//! Rule: `no-console-spaces`
//!
//! Disallow leading/trailing spaces in `console.log()` string arguments.
//! Leading spaces on the first argument and trailing spaces on the last
//! argument are almost always unintentional formatting mistakes.

use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Console methods to check.
const CONSOLE_METHODS: &[&str] = &["log", "warn", "error", "info", "debug"];

/// Flags leading/trailing spaces in console method string arguments.
#[derive(Debug)]
pub struct NoConsoleSpaces;

/// Extract a `StringLiteral` from an `Argument`, if it is one.
fn as_string_literal<'a>(arg: &'a Argument<'a>) -> Option<&'a oxc_ast::ast::StringLiteral<'a>> {
    if let Argument::StringLiteral(lit) = arg {
        Some(lit)
    } else {
        None
    }
}

impl NativeRule for NoConsoleSpaces {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-console-spaces".to_owned(),
            description: "Disallow leading/trailing spaces in console string arguments".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check callee is console.<method>
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        let Expression::Identifier(ident) = &member.object else {
            return;
        };
        if ident.name.as_str() != "console" {
            return;
        }
        let method = member.property.name.as_str();
        if !CONSOLE_METHODS.contains(&method) {
            return;
        }

        if call.arguments.is_empty() {
            return;
        }

        let mut edits = Vec::new();

        // Check first argument for leading space
        if let Some(lit) = call.arguments.first().and_then(as_string_literal) {
            if lit.value.starts_with(' ') && lit.value.len() > 1 {
                edits.push(Edit {
                    span: Span::new(
                        lit.span.start.saturating_add(1),
                        lit.span.start.saturating_add(2),
                    ),
                    replacement: String::new(),
                });
            }
        }

        // Check last argument for trailing space
        if let Some(lit) = call.arguments.last().and_then(as_string_literal) {
            if lit.value.ends_with(' ') && lit.value.len() > 1 {
                edits.push(Edit {
                    span: Span::new(
                        lit.span.end.saturating_sub(2),
                        lit.span.end.saturating_sub(1),
                    ),
                    replacement: String::new(),
                });
            }
        }

        if edits.is_empty() {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "no-console-spaces".to_owned(),
            message: "Unexpected space in console call argument".to_owned(),
            span: Span::new(call.span.start, call.span.end),
            severity: Severity::Warning,
            help: Some("Remove the leading/trailing space from the string".to_owned()),
            fix: Some(Fix {
                message: "Remove space".to_owned(),
                edits,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoConsoleSpaces)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_leading_space() {
        let diags = lint("console.log(' hello');");
        assert_eq!(diags.len(), 1, "should flag leading space");
    }

    #[test]
    fn test_flags_trailing_space() {
        let diags = lint("console.log('hello ');");
        assert_eq!(diags.len(), 1, "should flag trailing space");
    }

    #[test]
    fn test_flags_console_error() {
        let diags = lint("console.error(' oops');");
        assert_eq!(diags.len(), 1, "should flag console.error");
    }

    #[test]
    fn test_flags_console_warn() {
        let diags = lint("console.warn('warning ');");
        assert_eq!(diags.len(), 1, "should flag console.warn");
    }

    #[test]
    fn test_allows_no_spaces() {
        let diags = lint("console.log('hello');");
        assert!(diags.is_empty(), "no spaces should not be flagged");
    }

    #[test]
    fn test_allows_non_string_arg() {
        let diags = lint("console.log(variable);");
        assert!(diags.is_empty(), "non-string arg should not be flagged");
    }

    #[test]
    fn test_allows_non_console() {
        let diags = lint("logger.log(' hello');");
        assert!(diags.is_empty(), "non-console should not be flagged");
    }

    #[test]
    fn test_allows_console_time() {
        let diags = lint("console.time(' timer');");
        assert!(diags.is_empty(), "non-log methods should not be flagged");
    }
}
