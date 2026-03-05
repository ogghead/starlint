//! Rule: `react/jsx-props-no-spread-multi`
//!
//! Warn when a JSX element has multiple spread attributes.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXAttributeItem;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-props-no-spread-multi";

/// Flags JSX elements with more than one spread attribute. Multiple spreads
/// make prop resolution order confusing and error-prone.
#[derive(Debug)]
pub struct JsxPropsNoSpreadMulti;

impl NativeRule for JsxPropsNoSpreadMulti {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow multiple spread attributes on a JSX element".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let spread_count = opening
            .attributes
            .iter()
            .filter(|attr| matches!(attr, JSXAttributeItem::SpreadAttribute(_)))
            .count();

        if spread_count > 1 {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!(
                    "JSX element has {spread_count} spread attributes — use at most one to avoid confusing prop resolution order"
                ),
                span: Span::new(opening.span.start, opening.span.end),
                severity: Severity::Warning,
                help: None,
                fix: None,
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

    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxPropsNoSpreadMulti)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_multiple_spreads() {
        let diags = lint("const el = <div {...a} {...b} />;");
        assert_eq!(diags.len(), 1, "should flag element with multiple spreads");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_single_spread() {
        let diags = lint("const el = <div {...props} />;");
        assert!(diags.is_empty(), "should not flag element with one spread");
    }

    #[test]
    fn test_allows_no_spreads() {
        let diags = lint(r#"const el = <div className="foo" />;"#);
        assert!(diags.is_empty(), "should not flag element with no spreads");
    }
}
