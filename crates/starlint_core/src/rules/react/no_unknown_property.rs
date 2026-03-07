//! Rule: `react/no-unknown-property`
//!
//! Warn when using HTML attributes instead of their React equivalents in JSX.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags HTML attributes that should use their React equivalents.
#[derive(Debug)]
pub struct NoUnknownProperty;

/// Returns the React equivalent for a known HTML attribute, or `None` if
/// the attribute is fine as-is.
fn react_equivalent(html_attr: &str) -> Option<&'static str> {
    match html_attr {
        "class" => Some("className"),
        "for" => Some("htmlFor"),
        "tabindex" => Some("tabIndex"),
        "readonly" => Some("readOnly"),
        "maxlength" => Some("maxLength"),
        "cellpadding" => Some("cellPadding"),
        "cellspacing" => Some("cellSpacing"),
        "colspan" => Some("colSpan"),
        "rowspan" => Some("rowSpan"),
        "enctype" => Some("encType"),
        "formaction" => Some("formAction"),
        "formenctype" => Some("formEncType"),
        "formmethod" => Some("formMethod"),
        "formnovalidate" => Some("formNoValidate"),
        "formtarget" => Some("formTarget"),
        "frameborder" => Some("frameBorder"),
        "novalidate" => Some("noValidate"),
        "accesskey" => Some("accessKey"),
        "charset" => Some("charSet"),
        "datetime" => Some("dateTime"),
        "hreflang" => Some("hrefLang"),
        "httpequiv" => Some("httpEquiv"),
        "srcdoc" => Some("srcDoc"),
        "srclang" => Some("srcLang"),
        "srcset" => Some("srcSet"),
        "usemap" => Some("useMap"),
        "crossorigin" => Some("crossOrigin"),
        "autocomplete" => Some("autoComplete"),
        "autofocus" => Some("autoFocus"),
        "autoplay" => Some("autoPlay"),
        // Event handler HTML attributes vs React equivalents
        "onclick" => Some("onClick"),
        "onchange" => Some("onChange"),
        "onfocus" => Some("onFocus"),
        "onblur" => Some("onBlur"),
        "onsubmit" => Some("onSubmit"),
        "onkeydown" => Some("onKeyDown"),
        "onkeyup" => Some("onKeyUp"),
        "onmousedown" => Some("onMouseDown"),
        "onmouseup" => Some("onMouseUp"),
        "onmouseover" => Some("onMouseOver"),
        _ => None,
    }
}

/// Returns true for native HTML element names (lowercase).
fn is_html_element(name: &str) -> bool {
    // HTML elements are lowercase
    name.starts_with(|c: char| c.is_ascii_lowercase())
}

impl LintRule for NoUnknownProperty {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-unknown-property".to_owned(),
            description: "Disallow usage of unknown DOM properties".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::JSXOpeningElement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::JSXOpeningElement(opening) = node else {
            return;
        };

        // Only check on native HTML elements, not custom components
        if !is_html_element(opening.name.as_str()) {
            return;
        }

        // Collect violations to avoid borrow conflict with ctx.report()
        let violations: Vec<(String, starlint_ast::types::Span, &'static str)> = opening
            .attributes
            .iter()
            .filter_map(|attr_id| {
                if let Some(AstNode::JSXAttribute(attr)) = ctx.node(*attr_id) {
                    let attr_name = attr.name.as_str();
                    if let Some(react_name) = react_equivalent(attr_name) {
                        return Some((attr_name.to_owned(), attr.span, react_name));
                    }
                }
                None
            })
            .collect();

        for (attr_name, attr_span, react_name) in violations {
            let span = Span::new(attr_span.start, attr_span.end);

            ctx.report(Diagnostic {
                rule_name: "react/no-unknown-property".to_owned(),
                message: format!("Unknown property `{attr_name}` — did you mean `{react_name}`?"),
                span,
                severity: Severity::Warning,
                help: Some(format!("Replace `{attr_name}` with `{react_name}`")),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: format!("Rename to `{react_name}`"),
                    edits: vec![Edit {
                        span,
                        replacement: react_name.to_owned(),
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUnknownProperty)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_class_attribute() {
        let diags = lint(r#"const x = <div class="foo" />;"#);
        assert_eq!(diags.len(), 1, "should flag `class`");
        assert!(
            diags
                .first()
                .is_some_and(|d| d.message.contains("className")),
            "should suggest className"
        );
    }

    #[test]
    fn test_flags_for_attribute() {
        let diags = lint(r#"const x = <label for="input-id">Label</label>;"#);
        assert_eq!(diags.len(), 1, "should flag `for`");
        assert!(
            diags.first().is_some_and(|d| d.message.contains("htmlFor")),
            "should suggest htmlFor"
        );
    }

    #[test]
    fn test_allows_correct_react_props() {
        let diags = lint(r#"const x = <div className="foo" tabIndex={0} />;"#);
        assert!(
            diags.is_empty(),
            "correct React props should not be flagged"
        );
    }
}
