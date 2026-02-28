//! Rule: `no-keyword-prefix`
//!
//! Flags identifiers that start with a JavaScript keyword followed by an
//! underscore (e.g. `new_foo`, `class_name`). These prefixes are confusing
//! because they look like keyword usage rather than variable names.

use oxc_ast::AstKind;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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
fn report_keyword_prefix(
    name: &str,
    span_start: u32,
    span_end: u32,
    ctx: &mut NativeLintContext<'_>,
) {
    if let Some(keyword) = find_keyword_prefix(name) {
        ctx.report_warning(
            "no-keyword-prefix",
            &format!("Do not prefix identifiers with keyword `{keyword}_`"),
            Span::new(span_start, span_end),
        );
    }
}

impl NativeRule for NoKeywordPrefix {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-keyword-prefix".to_owned(),
            description: "Disallow identifiers starting with a JavaScript keyword prefix"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::BindingIdentifier(ident) => {
                report_keyword_prefix(ident.name.as_str(), ident.span.start, ident.span.end, ctx);
            }
            AstKind::IdentifierReference(ident) => {
                report_keyword_prefix(ident.name.as_str(), ident.span.start, ident.span.end, ctx);
            }
            _ => {}
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

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoKeywordPrefix)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
