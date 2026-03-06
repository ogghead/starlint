//! Rule: `consistent-template-literal-escape` (unicorn)
//!
//! Flags unnecessary escape sequences in template literals. Single quotes
//! (`\'`) and double quotes (`\"`) do not need escaping inside template
//! literals (backtick-delimited strings), so their escaped forms are
//! unnecessary noise.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unnecessary `\'` or `\"` escapes in template literals.
#[derive(Debug)]
pub struct ConsistentTemplateLiteralEscape;

/// Check if a raw template quasi string contains unnecessary escape sequences.
///
/// Looks for `\'` or `\"` which do not need escaping in template literals.
fn has_unnecessary_escape(raw: &str) -> bool {
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('\'' | '"') => return true,
                Some(_) => {
                    // Skip the escaped character
                    let _skip = chars.next();
                }
                None => {}
            }
        }
    }
    false
}

/// Remove unnecessary `\'` and `\"` escape sequences from raw template text.
fn remove_unnecessary_escapes(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && matches!(chars.peek(), Some('\'' | '"')) {
            // Skip the backslash, the quote will be added by the next iteration
        } else {
            result.push(ch);
        }
    }
    result
}

/// Find the span of the raw content within a template quasi element.
#[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
fn find_quasi_raw_span(quasi: &oxc_ast::ast::TemplateElement<'_>, source: &str) -> Option<Span> {
    let start = quasi.span.start as usize;
    let end = quasi.span.end as usize;
    let slice = source.get(start..end)?;

    let content_start = if slice.starts_with('`') || slice.starts_with('}') {
        start.saturating_add(1)
    } else {
        start
    };
    let content_end = if slice.ends_with('`') {
        end.saturating_sub(1)
    } else if slice.ends_with("${") {
        end.saturating_sub(2)
    } else {
        end
    };

    Some(Span::new(content_start as u32, content_end as u32))
}

impl NativeRule for ConsistentTemplateLiteralEscape {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "consistent-template-literal-escape".to_owned(),
            description:
                "Disallow unnecessary escape sequences `\\'` and `\\\"` in template literals"
                    .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TemplateLiteral])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TemplateLiteral(template) = kind else {
            return;
        };

        let mut edits = Vec::new();
        let mut found = false;
        for quasi in &template.quasis {
            let raw = quasi.value.raw.as_str();
            if has_unnecessary_escape(raw) {
                found = true;
                let cleaned = remove_unnecessary_escapes(raw);
                #[allow(clippy::as_conversions)]
                if let Some(raw_edit) =
                    find_quasi_raw_span(quasi, ctx.source_text()).map(|span| Edit {
                        span,
                        replacement: cleaned,
                    })
                {
                    edits.push(raw_edit);
                }
            }
        }

        if found {
            let fix = (!edits.is_empty()).then(|| Fix {
                kind: FixKind::SafeFix,
                message: "Remove unnecessary escape sequences".to_owned(),
                edits,
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "consistent-template-literal-escape".to_owned(),
                message:
                    "Unnecessary escape sequence in template literal — `\\'` and `\\\"` do not \
                     need escaping in template literals"
                        .to_owned(),
                span: Span::new(template.span.start, template.span.end),
                severity: Severity::Warning,
                help: Some(
                    "Remove the backslash — single and double quotes don't need escaping in \
                     template literals"
                        .to_owned(),
                ),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ConsistentTemplateLiteralEscape)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_escaped_single_quote() {
        let diags = lint(r"var x = `hello \'world\'`;");
        assert_eq!(
            diags.len(),
            1,
            "escaped single quotes in template literal should be flagged"
        );
    }

    #[test]
    fn test_flags_escaped_double_quote() {
        let diags = lint(r#"var x = `say \"hi\"`;"#);
        assert_eq!(
            diags.len(),
            1,
            "escaped double quotes in template literal should be flagged"
        );
    }

    #[test]
    fn test_allows_escaped_backtick() {
        let diags = lint(r"var x = `hello \`world\``;");
        assert!(
            diags.is_empty(),
            "escaped backticks in template literal should not be flagged (they are necessary)"
        );
    }

    #[test]
    fn test_allows_template_with_expression() {
        let diags = lint("var x = `hello ${name}`;");
        assert!(
            diags.is_empty(),
            "plain template literal with expression should not be flagged"
        );
    }

    #[test]
    fn test_allows_plain_template() {
        let diags = lint("var x = `hello world`;");
        assert!(
            diags.is_empty(),
            "plain template literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_escapes() {
        let diags = lint(r"var x = `hello\nworld`;");
        assert!(
            diags.is_empty(),
            "newline escape in template literal should not be flagged"
        );
    }

    #[test]
    fn test_allows_tab_escape() {
        let diags = lint(r"var x = `col1\tcol2`;");
        assert!(
            diags.is_empty(),
            "tab escape in template literal should not be flagged"
        );
    }

    #[test]
    fn test_flags_only_once_per_template() {
        let diags = lint(r"var x = `\'hello\' and \'world\'`;");
        assert_eq!(
            diags.len(),
            1,
            "should report only once per template literal even with multiple occurrences"
        );
    }
}
