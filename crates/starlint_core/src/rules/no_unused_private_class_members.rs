//! Rule: `no-unused-private-class-members`
//!
//! Disallow unused private class members. Private fields and methods that are
//! declared but never used are dead code and should be removed.

use std::collections::{HashMap, HashSet};

use oxc_ast::AstKind;
use oxc_ast::ast::{ClassElement, Expression, PropertyKey};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags unused private class members (fields and methods).
#[derive(Debug)]
pub struct NoUnusedPrivateClassMembers;

impl NativeRule for NoUnusedPrivateClassMembers {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-unused-private-class-members".to_owned(),
            description: "Disallow unused private class members".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        // Collect declared private members and their spans
        let mut declared: HashMap<String, Span> = HashMap::new();
        // Collect used private member names
        let mut used: HashSet<String> = HashSet::new();

        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) => {
                    if let PropertyKey::PrivateIdentifier(id) = &method.key {
                        let name = id.name.to_string();
                        declared.insert(name, Span::new(id.span.start, id.span.end));
                    }
                    // Check method body for private member usage
                    if let Some(body) = &method.value.body {
                        collect_private_references_from_source(
                            ctx.source_text(),
                            body.span.start,
                            body.span.end,
                            &mut used,
                        );
                    }
                }
                ClassElement::PropertyDefinition(prop) => {
                    if let PropertyKey::PrivateIdentifier(id) = &prop.key {
                        let name = id.name.to_string();
                        declared.insert(name, Span::new(id.span.start, id.span.end));
                    }
                    // Check initializer for private member usage
                    if let Some(init) = &prop.value {
                        collect_private_references_from_expr(init, &mut used);
                    }
                }
                _ => {}
            }
        }

        // Report declared but unused private members
        for (name, span) in &declared {
            if !used.contains(name) {
                ctx.report_error(
                    "no-unused-private-class-members",
                    &format!("Private member `#{name}` is declared but never used"),
                    *span,
                );
            }
        }
    }
}

/// Collect private member references from source text in a span range.
///
/// This is a simple heuristic — we look for `#identifier` patterns in the
/// source text within the given range.
fn collect_private_references_from_source(
    source: &str,
    start: u32,
    end: u32,
    used: &mut HashSet<String>,
) {
    let start_idx = usize::try_from(start).unwrap_or(0);
    let end_idx = usize::try_from(end).unwrap_or(0);
    let Some(text) = source.get(start_idx..end_idx) else {
        return;
    };

    // Find all #identifier patterns
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes.get(i).copied() == Some(b'#') {
            let name_start = i.saturating_add(1);
            let mut name_end = name_start;
            while name_end < len {
                let ch = bytes.get(name_end).copied().unwrap_or(0);
                if ch.is_ascii_alphanumeric() || ch == b'_' {
                    name_end = name_end.saturating_add(1);
                } else {
                    break;
                }
            }
            if name_end > name_start {
                if let Some(name) = text.get(name_start..name_end) {
                    used.insert(name.to_owned());
                }
            }
            i = name_end;
        } else {
            i = i.saturating_add(1);
        }
    }
}

/// Collect private member references from an expression.
fn collect_private_references_from_expr(expr: &Expression<'_>, used: &mut HashSet<String>) {
    if let Expression::PrivateFieldExpression(field) = expr {
        used.insert(field.field.name.to_string());
    }
    // For more complex expressions we'd need recursive walking;
    // the source text heuristic above covers most cases.
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnusedPrivateClassMembers)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_unused_private_field() {
        let diags = lint("class A { #unused = 1; method() { return 2; } }");
        assert_eq!(
            diags.len(),
            1,
            "unused private field should be flagged"
        );
    }

    #[test]
    fn test_allows_used_private_field() {
        let diags = lint("class A { #x = 1; method() { return this.#x; } }");
        assert!(
            diags.is_empty(),
            "used private field should not be flagged"
        );
    }

    #[test]
    fn test_flags_unused_private_method() {
        let diags = lint("class A { #unusedMethod() {} method() { return 1; } }");
        assert_eq!(
            diags.len(),
            1,
            "unused private method should be flagged"
        );
    }

    #[test]
    fn test_allows_used_private_method() {
        let diags = lint("class A { #helper() { return 1; } method() { return this.#helper(); } }");
        assert!(
            diags.is_empty(),
            "used private method should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_private_members() {
        let diags = lint("class A { x = 1; method() { return this.x; } }");
        assert!(
            diags.is_empty(),
            "class without private members should not be flagged"
        );
    }
}
