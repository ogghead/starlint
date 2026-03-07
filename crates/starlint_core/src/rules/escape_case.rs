//! Rule: `escape-case` (unicorn)
//!
//! Require escape sequences to use uppercase hex digits. For example,
//! `\xff` should be `\xFF` and `\u00ff` should be `\u00FF`.

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};

/// Flags escape sequences with lowercase hex digits.
#[derive(Debug)]
pub struct EscapeCase;

impl LintRule for EscapeCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "escape-case".to_owned(),
            description: "Require uppercase hex digits in escape sequences".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::StringLiteral])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::StringLiteral(lit) = node else {
            return;
        };

        let source = ctx.source_text();
        let Some(raw) = source.get(lit.span.start as usize..lit.span.end as usize) else {
            return;
        };

        if !has_lowercase_escape(raw) {
            return;
        }

        let fixed = uppercase_escapes(raw);
        ctx.report(Diagnostic {
            rule_name: "escape-case".to_owned(),
            message: "Use uppercase hex digits in escape sequences (e.g., `\\xFF` not `\\xff`)"
                .to_owned(),
            span: Span::new(lit.span.start, lit.span.end),
            severity: Severity::Warning,
            help: Some("Uppercase hex digits in escape sequences".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Uppercase hex digits".to_owned(),
                edits: vec![Edit {
                    span: Span::new(lit.span.start, lit.span.end),
                    replacement: fixed,
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

/// Produce a new raw string with all hex digits in escape sequences uppercased.
fn uppercase_escapes(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        result.push('\\');
        match chars.peek() {
            Some('x') => {
                if let Some(x) = chars.next() {
                    result.push(x);
                } // push 'x'
                for _ in 0..2 {
                    if let Some(&c) = chars.peek() {
                        if c.is_ascii_hexdigit() {
                            result.push(c.to_ascii_uppercase());
                            let _ = chars.next();
                        } else {
                            break;
                        }
                    }
                }
            }
            Some('u') => {
                if let Some(u) = chars.next() {
                    result.push(u);
                } // push 'u'
                if chars.peek() == Some(&'{') {
                    if let Some(brace) = chars.next() {
                        result.push(brace);
                    } // push '{'
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            if let Some(close) = chars.next() {
                                result.push(close);
                            }
                            break;
                        }
                        if c.is_ascii_hexdigit() {
                            result.push(c.to_ascii_uppercase());
                        } else {
                            result.push(c);
                        }
                        let _ = chars.next();
                    }
                } else {
                    for _ in 0..4 {
                        if let Some(&c) = chars.peek() {
                            if c.is_ascii_hexdigit() {
                                result.push(c.to_ascii_uppercase());
                                let _ = chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
            Some(&next) => {
                result.push(next);
                let _ = chars.next();
            }
            None => {}
        }
    }
    result
}

/// Check if a raw string contains escape sequences with lowercase hex digits.
///
/// Looks for `\xNN` or `\uNNNN` where the hex digits contain lowercase a-f.
fn has_lowercase_escape(raw: &str) -> bool {
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            continue;
        }
        match chars.peek() {
            Some('x') => {
                let _x = chars.next(); // consume 'x'
                // Check next 2 hex digits
                for _ in 0..2 {
                    if let Some(&c) = chars.peek() {
                        if c.is_ascii_hexdigit() && c.is_ascii_lowercase() {
                            return true;
                        }
                        let _skip = chars.next();
                    }
                }
            }
            Some('u') => {
                let _u = chars.next(); // consume 'u'
                if chars.peek() == Some(&'{') {
                    // \u{...} form
                    let _brace = chars.next();
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            let _close = chars.next();
                            break;
                        }
                        if c.is_ascii_hexdigit() && c.is_ascii_lowercase() {
                            return true;
                        }
                        let _skip = chars.next();
                    }
                } else {
                    // \uNNNN form — check next 4 hex digits
                    for _ in 0..4 {
                        if let Some(&c) = chars.peek() {
                            if c.is_ascii_hexdigit() && c.is_ascii_lowercase() {
                                return true;
                            }
                            let _skip = chars.next();
                        }
                    }
                }
            }
            Some(_) => {
                let _skip = chars.next();
            }
            None => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(EscapeCase)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_lowercase_hex_escape() {
        let diags = lint(r"var s = '\xff';");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_uppercase_hex_escape() {
        let diags = lint(r"var s = '\xFF';");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_flags_lowercase_unicode_escape() {
        let diags = lint(r"var s = '\u00ff';");
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allows_uppercase_unicode_escape() {
        let diags = lint(r"var s = '\u00FF';");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allows_normal_string() {
        let diags = lint(r"var s = 'hello';");
        assert!(diags.is_empty());
    }
}
