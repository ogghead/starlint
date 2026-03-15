//! Rule: `typescript/prefer-function-type`
//!
//! Prefer function type syntax (`() => void`) over an interface or object type
//! with a single call signature. When an interface has exactly one member and
//! that member is a call signature, the interface can be replaced with a simpler
//! function type alias, improving readability.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/prefer-function-type";

/// Flags interfaces whose body contains exactly one member that is a call
/// signature declaration — these can be expressed more concisely as function
/// type aliases.
#[derive(Debug)]
pub struct PreferFunctionType;

impl LintRule for PreferFunctionType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Prefer function type syntax over interface with a single call signature"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::TSInterfaceDeclaration])
    }

    #[allow(clippy::as_conversions)]
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::TSInterfaceDeclaration(decl) = node else {
            return;
        };

        // TSInterfaceDeclarationNode has no extends field. Use source text to check.
        let source = ctx.source_text();
        let decl_start = decl.span.start as usize;
        let decl_end = decl.span.end as usize;
        let decl_text = source.get(decl_start..decl_end).unwrap_or("");
        // Check if the interface extends something by looking for "extends" keyword
        // before the opening brace
        if let Some(brace_pos) = decl_text.find('{') {
            let before_brace = &decl_text[..brace_pos];
            if before_brace.contains("extends") {
                return;
            }
        }

        let members = &decl.body;

        // Only flag when there is exactly one member
        if members.len() != 1 {
            return;
        }

        // Check if the single member is a call signature using source text.
        // TSCallSignatureDeclaration is not in starlint_ast (mapped as Unknown).
        // A call signature starts with `(` in the interface body.
        let Some(member_id) = members.first() else {
            return;
        };

        let member_span = ctx.node(*member_id).map_or(
            starlint_ast::types::Span::EMPTY,
            starlint_ast::AstNode::span,
        );
        let sig_start = member_span.start as usize;
        let sig_end = member_span.end as usize;
        let sig_text = source.get(sig_start..sig_end).unwrap_or("").trim();

        // A call signature starts with `(`, not a property name
        if !sig_text.starts_with('(') {
            return;
        }

        let id_name = ctx
            .node(decl.id)
            .and_then(|n| {
                if let AstNode::BindingIdentifier(id) = n {
                    Some(id.name.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("Unknown");
        let message = format!(
            "Interface `{id_name}` has only a call signature — use a function type instead (e.g. `type {id_name} = (...) => ...`)"
        );
        let decl_span = Span::new(decl.span.start, decl.span.end);

        // Build the fix: extract params and return type from source text
        // The call signature looks like `(params): ReturnType` or `(params)`
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
                format!("type {id_name} = {params} => {return_type};")
            })
        });

        ctx.report(Diagnostic {
            rule_name: RULE_NAME.to_owned(),
            message,
            span: decl_span,
            severity: Severity::Warning,
            help: Some(format!("Convert to `type {id_name} = (...) => ...`")),
            fix: fix.map(|replacement| Fix {
                kind: FixKind::SafeFix,
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

    use super::*;

    starlint_rule_framework::lint_rule_test!(PreferFunctionType, "test.ts");

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
