//! Rule: `react/jsx-curly-brace-presence`
//!
//! Suggest removing unnecessary curly braces around string literals in JSX props.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXExpression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-curly-brace-presence";

/// Flags JSX expression containers that wrap a plain string literal, which is
/// unnecessary since JSX supports string attribute values directly.
///
/// For example: `<Comp prop={"text"} />` should be `<Comp prop="text" />`.
#[derive(Debug)]
pub struct JsxCurlyBracePresence;

impl NativeRule for JsxCurlyBracePresence {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary curly braces around string literals in JSX props"
                .to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXExpressionContainer(container) = kind else {
            return;
        };

        if let JSXExpression::StringLiteral(_) = &container.expression {
            ctx.report_warning(
                RULE_NAME,
                "Unnecessary curly braces around string literal. Use a plain string attribute value instead",
                Span::new(container.span.start, container.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxCurlyBracePresence)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_in_curly_braces() {
        let diags = lint(r#"const el = <div className={"foo"} />;"#);
        assert_eq!(diags.len(), 1, "should flag string literal in curly braces");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_plain_string() {
        let diags = lint(r#"const el = <div className="foo" />;"#);
        assert!(diags.is_empty(), "should not flag plain string attribute");
    }

    #[test]
    fn test_allows_expression() {
        let diags = lint("const el = <div className={styles.foo} />;");
        assert!(
            diags.is_empty(),
            "should not flag non-string expressions in braces"
        );
    }
}
