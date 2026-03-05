//! Rule: `no-shadow-restricted-names`
//!
//! Disallow identifiers from shadowing restricted names such as `undefined`,
//! `NaN`, `Infinity`, `eval`, and `arguments`. Shadowing these can lead to
//! confusing behavior and subtle bugs.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Names that should not be shadowed.
const RESTRICTED_NAMES: &[&str] = &["undefined", "NaN", "Infinity", "eval", "arguments"];

/// Flags binding identifiers that shadow restricted global names.
#[derive(Debug)]
pub struct NoShadowRestrictedNames;

impl NativeRule for NoShadowRestrictedNames {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-shadow-restricted-names".to_owned(),
            description: "Disallow identifiers from shadowing restricted names".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::BindingIdentifier])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::BindingIdentifier(ident) = kind else {
            return;
        };

        let name = ident.name.as_str();
        if RESTRICTED_NAMES.contains(&name) {
            ctx.report(Diagnostic {
                rule_name: "no-shadow-restricted-names".to_owned(),
                message: format!("Shadowing of global property `{name}`"),
                span: Span::new(ident.span.start, ident.span.end),
                severity: Severity::Error,
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoShadowRestrictedNames)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_var_undefined() {
        let diags = lint("var undefined = 1;");
        assert_eq!(diags.len(), 1, "shadowing undefined should be flagged");
    }

    #[test]
    fn test_flags_let_nan() {
        let diags = lint("let NaN = 42;");
        assert_eq!(diags.len(), 1, "shadowing NaN should be flagged");
    }

    #[test]
    fn test_flags_const_infinity() {
        let diags = lint("const Infinity = 0;");
        assert_eq!(diags.len(), 1, "shadowing Infinity should be flagged");
    }

    #[test]
    fn test_flags_function_param_eval() {
        let diags = lint("function f(eval) {}");
        assert_eq!(diags.len(), 1, "shadowing eval in params should be flagged");
    }

    #[test]
    fn test_flags_catch_arguments() {
        let diags = lint("try {} catch (arguments) {}");
        assert_eq!(
            diags.len(),
            1,
            "shadowing arguments in catch should be flagged"
        );
    }

    #[test]
    fn test_allows_normal_names() {
        let diags = lint("let x = 1; const y = 2; var z = 3;");
        assert!(diags.is_empty(), "normal names should not be flagged");
    }

    #[test]
    fn test_allows_using_restricted_names() {
        let diags = lint("console.log(undefined, NaN, Infinity);");
        assert!(
            diags.is_empty(),
            "using (not shadowing) restricted names should not be flagged"
        );
    }
}
