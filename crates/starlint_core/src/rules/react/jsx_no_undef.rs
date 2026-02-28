//! Rule: `react/jsx-no-undef`
//!
//! Warn when JSX references a component that looks undefined (heuristic:
//! single `PascalCase` word with no dots, not an HTML intrinsic element).

use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-undef";

/// Well-known global JSX identifiers that should not be flagged.
const KNOWN_GLOBALS: &[&str] = &["React", "Fragment"];

/// Flags JSX references to `PascalCase` component names that are likely
/// undefined. This is a heuristic — full scope analysis would require
/// semantic data.
#[derive(Debug)]
pub struct JsxNoUndef;

impl NativeRule for JsxNoUndef {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Warn when JSX references an undefined component (heuristic)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXOpeningElement(opening) = kind else {
            return;
        };

        // Only check simple identifier references (not member expressions like Foo.Bar)
        let name = match &opening.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            JSXElementName::IdentifierReference(ident_ref) => ident_ref.name.as_str(),
            _ => return,
        };

        // Skip lowercase names (HTML intrinsic elements like div, span, etc.)
        let Some(first_char) = name.chars().next() else {
            return;
        };
        if !first_char.is_ascii_uppercase() {
            return;
        }

        // Skip well-known globals
        if KNOWN_GLOBALS.contains(&name) {
            return;
        }

        // Heuristic: check for common definition patterns in the source text
        let source = ctx.source_text();

        let has_definition = source.contains(&format!("import {name}"))
            || source.contains(&format!("import {{ {name}"))
            || source.contains(&format!("import {{{name}"))
            || source.contains(&format!("const {name}"))
            || source.contains(&format!("let {name}"))
            || source.contains(&format!("var {name}"))
            || source.contains(&format!("function {name}"))
            || source.contains(&format!("class {name}"));

        if !has_definition {
            ctx.report_warning(
                RULE_NAME,
                &format!("`{name}` is not defined — possibly missing import"),
                Span::new(opening.span.start, opening.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoUndef)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_undefined_component() {
        let diags = lint("const el = <MyComponent />;");
        assert_eq!(diags.len(), 1, "should flag undefined PascalCase component");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_allows_imported_component() {
        let diags = lint("import MyComponent from './my';\nconst el = <MyComponent />;");
        assert!(diags.is_empty(), "should not flag imported component");
    }

    #[test]
    fn test_allows_html_intrinsic() {
        let diags = lint(r#"const el = <div className="foo" />;"#);
        assert!(diags.is_empty(), "should not flag HTML intrinsic elements");
    }
}
