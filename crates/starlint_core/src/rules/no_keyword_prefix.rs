//! Rule: `no-keyword-prefix`
//!
//! Flags identifiers that start with a JavaScript keyword followed by an
//! underscore (e.g. `new_foo`, `class_name`). These prefixes are confusing
//! because they look like keyword usage rather than variable names.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Keyword prefixes to check for (each includes the trailing underscore).
const KEYWORD_PREFIXES: &[&str] = &[
    "class_", "export_", "import_", "new_", "return_", "throw_", "typeof_",
];

/// Flags identifiers that start with a JavaScript keyword prefix followed by `_`.
#[derive(Debug)]
pub struct NoKeywordPrefix;

/// Check whether a name starts with a keyword prefix (keyword + underscore).
///
/// Returns the matched keyword (without the trailing underscore) if found,
/// or `None` otherwise. The name must have at least one character after
/// the prefix to be considered a match.
fn find_keyword_prefix(name: &str) -> Option<&'static str> {
    for &prefix in KEYWORD_PREFIXES {
        if name.starts_with(prefix) && name.len() > prefix.len() {
            let keyword_end = prefix.len().saturating_sub(1);
            if let Some(keyword) = prefix.get(..keyword_end) {
                return Some(keyword);
            }
        }
    }
    None
}

/// Report a diagnostic for an identifier with a keyword prefix.
fn report_keyword_prefix(name: &str, span_start: u32, span_end: u32, ctx: &mut LintContext<'_>) {
    if let Some(keyword) = find_keyword_prefix(name) {
        ctx.report(Diagnostic {
            rule_name: "no-keyword-prefix".to_owned(),
            message: format!("Do not prefix identifiers with keyword `{keyword}_`"),
            span: Span::new(span_start, span_end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

impl LintRule for NoKeywordPrefix {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-keyword-prefix".to_owned(),
            description: "Disallow identifiers starting with a JavaScript keyword prefix"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[
            AstNodeType::BindingIdentifier,
            AstNodeType::IdentifierReference,
        ])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::BindingIdentifier(ident) => {
                report_keyword_prefix(ident.name.as_str(), ident.span.start, ident.span.end, ctx);
            }
            AstNode::IdentifierReference(ident) => {
                report_keyword_prefix(ident.name.as_str(), ident.span.start, ident.span.end, ctx);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;
    use starlint_plugin_sdk::diagnostic::Diagnostic;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoKeywordPrefix)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_new_prefix() {
        let diags = lint("const new_foo = 1;");
        assert!(
            !diags.is_empty(),
            "identifier starting with 'new_' should be flagged"
        );
    }

    #[test]
    fn test_flags_class_prefix() {
        let diags = lint("const class_name = 'foo';");
        assert!(
            !diags.is_empty(),
            "identifier starting with 'class_' should be flagged"
        );
    }

    #[test]
    fn test_flags_return_prefix() {
        let diags = lint("let return_value = 42;");
        assert!(
            !diags.is_empty(),
            "identifier starting with 'return_' should be flagged"
        );
    }

    #[test]
    fn test_flags_typeof_prefix() {
        let diags = lint("var typeof_check = true;");
        assert!(
            !diags.is_empty(),
            "identifier starting with 'typeof_' should be flagged"
        );
    }

    #[test]
    fn test_flags_import_prefix() {
        let diags = lint("const import_path = './foo';");
        assert!(
            !diags.is_empty(),
            "identifier starting with 'import_' should be flagged"
        );
    }

    #[test]
    fn test_flags_export_prefix() {
        let diags = lint("let export_name = 'bar';");
        assert!(
            !diags.is_empty(),
            "identifier starting with 'export_' should be flagged"
        );
    }

    #[test]
    fn test_flags_throw_prefix() {
        let diags = lint("const throw_error = false;");
        assert!(
            !diags.is_empty(),
            "identifier starting with 'throw_' should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_identifier() {
        let diags = lint("const myVar = 1;");
        assert!(diags.is_empty(), "normal identifiers should not be flagged");
    }

    #[test]
    fn test_allows_keyword_without_underscore() {
        let diags = lint("const newValue = 1;");
        assert!(
            diags.is_empty(),
            "'newValue' (no underscore) should not be flagged"
        );
    }

    #[test]
    fn test_flags_identifier_reference() {
        let diags = lint("const new_foo = 1; console.log(new_foo);");
        assert!(
            diags.len() >= 2,
            "both binding and reference should be flagged"
        );
    }

    #[test]
    fn test_allows_newspaper() {
        let diags = lint("const newspaper = 1;");
        assert!(diags.is_empty(), "'newspaper' should not be flagged");
    }
}
