//! Rule: `react/no-unknown-property`
//!
//! Warn when using HTML attributes instead of their React equivalents in JSX.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeName, JSXElementName};

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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
fn is_html_element(name: &JSXElementName<'_>) -> bool {
    match name {
        JSXElementName::Identifier(id) => {
            let n = id.name.as_str();
            // HTML elements are lowercase
            n.starts_with(|c: char| c.is_ascii_lowercase())
        }
        _ => false,
    }
}

impl NativeRule for NoUnknownProperty {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-unknown-property".to_owned(),
            description: "Disallow usage of unknown DOM properties".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        // Only check on native HTML elements, not custom components
        if !is_html_element(&opening.name) {
            return;
        }

        for attr_item in &opening.attributes {
            let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = attr_item else {
                continue;
            };

            let attr_name = match &attr.name {
                JSXAttributeName::Identifier(id) => id.name.as_str(),
                JSXAttributeName::NamespacedName(_) => continue,
            };

            if let Some(react_name) = react_equivalent(attr_name) {
                ctx.report_warning(
                    "react/no-unknown-property",
                    &format!("Unknown property `{attr_name}` — did you mean `{react_name}`?"),
                    Span::new(attr.span.start, attr.span.end),
                );
            }
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnknownProperty)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
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
