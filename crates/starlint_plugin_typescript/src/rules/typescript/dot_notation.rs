//! Rule: `typescript/dot-notation`
//!
//! Enforce dot notation whenever possible instead of bracket notation for
//! property access. Writing `obj["property"]` is harder to read than
//! `obj.property` and should be avoided when the property name is a valid
//! JavaScript identifier.
//!
//! This rule uses the AST to find `ComputedMemberExpression` nodes whose
//! property is a `StringLiteral` containing a valid identifier name.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags bracket notation (`obj["prop"]`) when dot notation (`obj.prop`) works.
#[derive(Debug)]
pub struct DotNotation;

impl LintRule for DotNotation {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/dot-notation".to_owned(),
            description: "Enforce dot notation over bracket notation for property access"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ComputedMemberExpression])
    }

    #[allow(clippy::as_conversions)] // u32→usize is lossless on 32/64-bit
    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ComputedMemberExpression(computed) = node else {
            return;
        };

        // computed.expression is a NodeId — resolve it
        let (property_name, prop_owned) = {
            let Some(AstNode::StringLiteral(lit)) = ctx.node(computed.expression) else {
                return;
            };
            (lit.value.clone(), lit.value.clone())
        };

        if is_valid_js_identifier(&property_name) {
            // Build fix: replace obj["prop"] with obj.prop
            let source = ctx.source_text();
            // computed.object is a NodeId — resolve its span
            let obj_span = ctx.node(computed.object).map(AstNode::span);
            let obj_text = obj_span
                .map_or("", |sp| {
                    source.get(sp.start as usize..sp.end as usize).unwrap_or("")
                })
                .to_owned();

            let fix = (!obj_text.is_empty()).then(|| Fix {
                kind: FixKind::SafeFix,
                message: format!("Use `{obj_text}.{prop_owned}`"),
                edits: vec![Edit {
                    span: Span::new(computed.span.start, computed.span.end),
                    replacement: format!("{obj_text}.{prop_owned}"),
                }],
                is_snippet: false,
            });

            ctx.report(Diagnostic {
                rule_name: "typescript/dot-notation".to_owned(),
                message: format!(
                    "Use dot notation `obj.{prop_owned}` instead of bracket notation `obj[\"{prop_owned}\"]`"
                ),
                span: Span::new(computed.span.start, computed.span.end),
                severity: Severity::Warning,
                help: Some(format!("Use `.{prop_owned}` instead")),
                fix,
                labels: vec![],
            });
        }
    }
}

/// Check whether a string is a valid JavaScript identifier that can be used
/// with dot notation.
///
/// A valid identifier starts with a letter, `_`, or `$`, and contains only
/// letters, digits, `_`, or `$`. It must also not be empty.
///
/// This does not check for reserved words — JavaScript allows reserved words
/// as property names in dot notation (e.g. `obj.class` is valid).
fn is_valid_js_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character: must be a letter, `_`, or `$`
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
        return false;
    }

    // Remaining characters: letters, digits, `_`, or `$`
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(DotNotation)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_bracket_with_simple_string() {
        let diags = lint(r#"const x = obj["property"];"#);
        assert_eq!(diags.len(), 1, "`obj[\"property\"]` should be flagged");
    }

    #[test]
    fn test_flags_bracket_with_underscore_name() {
        let diags = lint(r#"const x = obj["_private"];"#);
        assert_eq!(diags.len(), 1, "`obj[\"_private\"]` should be flagged");
    }

    #[test]
    fn test_allows_bracket_with_hyphenated_name() {
        let diags = lint(r#"const x = obj["my-property"];"#);
        assert!(
            diags.is_empty(),
            "bracket notation with hyphens should not be flagged"
        );
    }

    #[test]
    fn test_allows_bracket_with_space_in_name() {
        let diags = lint(r#"const x = obj["has space"];"#);
        assert!(
            diags.is_empty(),
            "bracket notation with spaces should not be flagged"
        );
    }

    #[test]
    fn test_allows_bracket_with_numeric_key() {
        let diags = lint("const x = arr[0];");
        assert!(
            diags.is_empty(),
            "numeric bracket access should not be flagged"
        );
    }
}
