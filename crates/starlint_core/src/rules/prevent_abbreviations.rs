//! Rule: `prevent-abbreviations` (unicorn)
//!
//! Flags common abbreviations in identifiers and suggests full words instead.
//! For example, `btn` should be `button`, `cb` should be `callback`, etc.
//! Only flags exact matches where the entire identifier is an abbreviation.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Known abbreviation-to-expansion mappings.
const ABBREVIATIONS: &[(&str, &str)] = &[
    ("arg", "argument"),
    ("args", "arguments"),
    ("arr", "array"),
    ("btn", "button"),
    ("cb", "callback"),
    ("cfg", "configuration"),
    ("conf", "configuration"),
    ("ctx", "context"),
    ("dest", "destination"),
    ("dir", "directory"),
    ("doc", "document"),
    ("el", "element"),
    ("env", "environment"),
    ("err", "error"),
    ("evt", "event"),
    ("fn", "function"),
    ("idx", "index"),
    ("len", "length"),
    ("lib", "library"),
    ("msg", "message"),
    ("num", "number"),
    ("obj", "object"),
    ("param", "parameter"),
    ("params", "parameters"),
    ("pkg", "package"),
    ("prev", "previous"),
    ("prop", "property"),
    ("props", "properties"),
    ("ref", "reference"),
    ("refs", "references"),
    ("req", "request"),
    ("res", "response"),
    ("src", "source"),
    ("str", "string"),
    ("temp", "temporary"),
    ("tmp", "temporary"),
    ("val", "value"),
];

/// Flags identifiers that are common abbreviations.
#[derive(Debug)]
pub struct PreventAbbreviations;

/// Look up the expansion for a given abbreviation.
///
/// Returns the suggested full word if the name matches a known abbreviation,
/// or `None` if it is not an abbreviation.
fn find_expansion(name: &str) -> Option<&'static str> {
    for &(abbr, expansion) in ABBREVIATIONS {
        if name == abbr {
            return Some(expansion);
        }
    }
    None
}

impl LintRule for PreventAbbreviations {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prevent-abbreviations".to_owned(),
            description: "Prefer full words over common abbreviations in identifiers".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_semantic(&self) -> bool {
        true
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::BindingIdentifier])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::BindingIdentifier(ident) = node else {
            return;
        };

        let name = ident.name.as_str();

        if let Some(expansion) = find_expansion(name) {
            let decl_span = Span::new(ident.span.start, ident.span.end);

            // With semantic, rename declaration + all references.
            // Without semantic, fall back to renaming only the declaration.
            let fix = match ctx.resolve_symbol_id(ident.span).and_then(|symbol_id| {
                let sd = ctx.scope_data()?;
                let edits = fix_utils::rename_symbol_edits(sd, symbol_id, expansion, decl_span);
                FixBuilder::new(format!("Rename to `{expansion}`"), FixKind::SuggestionFix)
                    .edits(edits)
                    .build()
            }) {
                Some(f) => Some(f),
                None => FixBuilder::new(format!("Rename to `{expansion}`"), FixKind::SuggestionFix)
                    .replace(decl_span, expansion)
                    .build(),
            };

            ctx.report(Diagnostic {
                rule_name: "prevent-abbreviations".to_owned(),
                message: format!("The abbreviation `{name}` should be written as `{expansion}`"),
                span: decl_span,
                severity: Severity::Warning,
                help: None,
                fix,
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::fix::apply_fixes;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreventAbbreviations)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_btn() {
        let diags = lint("const btn = 1;");
        assert_eq!(diags.len(), 1, "abbreviation 'btn' should be flagged");
        assert!(
            diags.first().is_some_and(|d| d.message.contains("button")),
            "should suggest 'button'"
        );
    }

    #[test]
    fn test_flags_cb_function() {
        let diags = lint("function cb() {}");
        assert_eq!(
            diags.len(),
            1,
            "abbreviation 'cb' in function name should be flagged"
        );
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("callback")),
            "should suggest 'callback'"
        );
    }

    #[test]
    fn test_flags_err() {
        let diags = lint("let err = new Error();");
        assert_eq!(diags.len(), 1, "abbreviation 'err' should be flagged");
    }

    #[test]
    fn test_flags_msg() {
        let diags = lint("var msg = 'hello';");
        assert_eq!(diags.len(), 1, "abbreviation 'msg' should be flagged");
    }

    #[test]
    fn test_flags_val() {
        let diags = lint("const val = 42;");
        assert_eq!(diags.len(), 1, "abbreviation 'val' should be flagged");
    }

    #[test]
    fn test_flags_ctx() {
        let diags = lint("let ctx = getContext();");
        assert_eq!(diags.len(), 1, "abbreviation 'ctx' should be flagged");
    }

    #[test]
    fn test_flags_param_in_function() {
        let diags = lint("function foo(param) {}");
        assert_eq!(
            diags.len(),
            1,
            "abbreviation 'param' in parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_button() {
        let diags = lint("const button = 1;");
        assert!(diags.is_empty(), "full word 'button' should not be flagged");
    }

    #[test]
    fn test_allows_callback() {
        let diags = lint("const callback = fn;");
        assert!(
            diags.is_empty(),
            "full word 'callback' should not be flagged"
        );
    }

    #[test]
    fn test_allows_normal_names() {
        let diags = lint("const total = 100; let count = 0; var name = 'test';");
        assert!(diags.is_empty(), "normal identifiers should not be flagged");
    }

    #[test]
    fn test_does_not_flag_partial_match() {
        let diags = lint("const errorHandler = null;");
        assert!(
            diags.is_empty(),
            "identifiers containing abbreviations as substrings should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_abbreviations() {
        let diags = lint("const btn = 1; let msg = 'hi';");
        assert_eq!(
            diags.len(),
            2,
            "multiple abbreviations should each be flagged"
        );
    }

    #[test]
    fn test_flags_tmp() {
        let diags = lint("let tmp = null;");
        assert_eq!(diags.len(), 1, "abbreviation 'tmp' should be flagged");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("temporary")),
            "should suggest 'temporary'"
        );
    }

    #[test]
    fn test_fix_renames_declaration_and_references() {
        let source = "const btn = 1; console.log(btn);";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "should flag 'btn'");
        let fixed = apply_fixes(source, &diags);
        assert_eq!(
            fixed, "const button = 1; console.log(button);",
            "fix should rename both declaration and reference"
        );
    }

    #[test]
    fn test_fix_renames_multiple_references() {
        let source = "let msg = 'hi'; console.log(msg); alert(msg);";
        let diags = lint(source);
        let fixed = apply_fixes(source, &diags);
        assert_eq!(
            fixed, "let message = 'hi'; console.log(message); alert(message);",
            "fix should rename all references"
        );
    }
}
