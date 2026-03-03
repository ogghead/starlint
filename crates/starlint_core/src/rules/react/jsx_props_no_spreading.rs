//! Rule: `react/jsx-props-no-spreading`
//!
//! Warn against using spread attributes `{...props}` in JSX.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-props-no-spreading";

/// Flags JSX spread attributes (`{...props}`). Spreading makes it harder to
/// track which props a component receives and can pass unintended props.
#[derive(Debug)]
pub struct JsxPropsNoSpreading;

impl NativeRule for JsxPropsNoSpreading {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow spreading props in JSX".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXSpreadAttribute])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXSpreadAttribute(spread) = kind else {
            return;
        };

        ctx.report_warning(
            RULE_NAME,
            "Avoid spreading props in JSX — pass props explicitly for clarity",
            Span::new(spread.span.start, spread.span.end),
        );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxPropsNoSpreading)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_spread_props() {
        let diags = lint("const el = <div {...props} />;");
        assert_eq!(diags.len(), 1, "should flag spread attributes");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_explicit_props() {
        let diags = lint(r#"const el = <div className="foo" id="bar" />;"#);
        assert!(diags.is_empty(), "should not flag explicit props");
    }

    #[test]
    fn test_flags_multiple_spreads() {
        let diags = lint("const el = <div {...a} {...b} />;");
        assert_eq!(diags.len(), 2, "should flag each spread attribute");
    }
}
