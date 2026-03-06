//! Rule: `react/jsx-pascal-case`
//!
//! Warn when user-defined JSX components don't use `PascalCase` naming.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-pascal-case";

/// Flags user-defined JSX component names that are not `PascalCase`.
#[derive(Debug)]
pub struct JsxPascalCase;

/// Check if a name is `PascalCase` (starts with uppercase, contains at least
/// one lowercase character). `ALL_CAPS` names like `SVG` are allowed.
fn is_pascal_case(name: &str) -> bool {
    let Some(first) = name.chars().next() else {
        return true;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    // Allow ALL_CAPS_WITH_UNDERSCORES (e.g., SVG, UNSAFE_Component)
    let is_all_upper = name
        .chars()
        .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit());
    if is_all_upper {
        return true;
    }
    // Must have at least one lowercase letter for PascalCase
    // and no underscores (except leading _)
    let has_lowercase = name.chars().any(|c| c.is_ascii_lowercase());
    let has_invalid_underscore = name.chars().skip(1).any(|c| c == '_');
    has_lowercase && !has_invalid_underscore
}

impl NativeRule for JsxPascalCase {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Enforce PascalCase for user-defined JSX components".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXOpeningElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        let (name, span) = match &opening.name {
            JSXElementName::Identifier(ident) => (ident.name.as_str(), ident.span),
            JSXElementName::IdentifierReference(ident_ref) => {
                (ident_ref.name.as_str(), ident_ref.span)
            }
            // Member expressions (Foo.Bar) and namespaced names are skipped
            _ => return,
        };

        // Only check user-defined components (start with uppercase)
        let Some(first) = name.chars().next() else {
            return;
        };
        if !first.is_ascii_uppercase() {
            return;
        }

        if !is_pascal_case(name) {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: format!("Component `{name}` should use PascalCase naming"),
                span: Span::new(span.start, span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxPascalCase)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_snake_case_component() {
        let diags = lint("const el = <My_Component />;");
        assert_eq!(diags.len(), 1, "should flag snake_case component name");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_pascal_case() {
        let diags = lint("const el = <MyComponent />;");
        assert!(diags.is_empty(), "should not flag PascalCase name");
    }

    #[test]
    fn test_allows_all_caps() {
        let diags = lint("const el = <SVG />;");
        assert!(diags.is_empty(), "should not flag ALL_CAPS name");
    }
}
