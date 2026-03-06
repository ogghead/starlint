//! Rule: `typescript/prefer-function-type`
//!
//! Prefer function type syntax (`() => void`) over an interface or object type
//! with a single call signature. When an interface has exactly one member and
//! that member is a call signature, the interface can be replaced with a simpler
//! function type alias, improving readability.

use oxc_ast::AstKind;
use oxc_ast::ast::TSSignature;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-function-type";

/// Flags interfaces whose body contains exactly one member that is a call
/// signature declaration — these can be expressed more concisely as function
/// type aliases.
#[derive(Debug)]
pub struct PreferFunctionType;

impl NativeRule for PreferFunctionType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer function type syntax over interface with a single call signature"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSInterfaceDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSInterfaceDeclaration(decl) = kind else {
            return;
        };

        // Interfaces that extend other types are intentional even if they have
        // a single call signature (they add the callable behavior to the base).
        if !decl.extends.is_empty() {
            return;
        }

        let members = &decl.body.body;

        // Only flag when there is exactly one member
        if members.len() != 1 {
            return;
        }

        // Check if the single member is a call signature
        let Some(member) = members.first() else {
            return;
        };

        if let TSSignature::TSCallSignatureDeclaration(call_sig) = member {
            let name = decl.id.name.as_str();
            let message = format!(
                "Interface `{name}` has only a call signature — use a function type instead (e.g. `type {name} = (...) => ...`)"
            );
            let decl_span = Span::new(decl.span.start, decl.span.end);

            // Build the fix: extract params and return type from source text
            let source = ctx.source_text();
            let sig_start = usize::try_from(call_sig.span.start).unwrap_or(0);
            let sig_end = usize::try_from(call_sig.span.end).unwrap_or(0);
            let sig_text = source.get(sig_start..sig_end).unwrap_or("");

            // The call signature looks like `(params): ReturnType` or `(params)`
            // Find the matching closing paren for the params
            let fix = sig_text.find('(').and_then(|open| {
                find_matching_paren(sig_text, open).map(|close| {
                    let params = sig_text.get(open..close.saturating_add(1)).unwrap_or("()");
                    // After the closing paren, look for `: ReturnType`
                    let after_paren = sig_text.get(close.saturating_add(1)..).unwrap_or("").trim();
                    let return_type = if let Some(stripped) = after_paren.strip_prefix(':') {
                        stripped.trim().trim_end_matches(';').trim()
                    } else {
                        "void"
                    };
                    format!("type {name} = {params} => {return_type};")
                })
            });

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message,
                span: decl_span,
                severity: Severity::Warning,
                help: Some(format!("Convert to `type {name} = (...) => ...`")),
                fix: fix.map(|replacement| Fix {
                    message: "Convert to function type alias".to_owned(),
                    edits: vec![Edit {
                        span: decl_span,
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

/// Find the matching closing parenthesis for an opening paren at `open_pos`.
fn find_matching_paren(source: &str, open_pos: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth: usize = 0;
    let mut pos = open_pos;
    while pos < bytes.len() {
        match bytes.get(pos) {
            Some(b'(') => depth = depth.saturating_add(1),
            Some(b')') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
        pos = pos.saturating_add(1);
    }
    None
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreferFunctionType)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_interface_with_single_call_signature() {
        let diags = lint("interface Foo { (): void }");
        assert_eq!(
            diags.len(),
            1,
            "interface with only a call signature should be flagged"
        );
    }

    #[test]
    fn test_flags_interface_with_parameterized_call_signature() {
        let diags = lint("interface Callback { (x: number, y: string): boolean }");
        assert_eq!(
            diags.len(),
            1,
            "interface with a parameterized call signature should be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_multiple_members() {
        let diags = lint("interface Foo { (): void; bar: string }");
        assert!(
            diags.is_empty(),
            "interface with call signature and other members should not be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_extends() {
        let diags = lint("interface Foo extends Bar { (): void }");
        assert!(
            diags.is_empty(),
            "interface extending another type should not be flagged"
        );
    }

    #[test]
    fn test_allows_interface_with_properties_only() {
        let diags = lint("interface Foo { x: number; y: string }");
        assert!(
            diags.is_empty(),
            "interface with only properties should not be flagged"
        );
    }
}
