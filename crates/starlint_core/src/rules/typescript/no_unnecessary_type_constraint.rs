//! Rule: `typescript/no-unnecessary-type-constraint`
//!
//! Disallow unnecessary constraints on generic type parameters. When a type
//! parameter extends `any` or `unknown`, the constraint is redundant because
//! these are already the implicit defaults for unconstrained type parameters.

use oxc_ast::AstKind;
use oxc_ast::ast::TSType;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags type parameters with unnecessary `extends any` or `extends unknown` constraints.
#[derive(Debug)]
pub struct NoUnnecessaryTypeConstraint;

impl NativeRule for NoUnnecessaryTypeConstraint {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/no-unnecessary-type-constraint".to_owned(),
            description: "Disallow unnecessary constraints on generic type parameters".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::TSTypeParameter])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::TSTypeParameter(param) = kind else {
            return;
        };

        let constraint_name = match &param.constraint {
            Some(TSType::TSAnyKeyword(_)) => "any",
            Some(TSType::TSUnknownKeyword(_)) => "unknown",
            _ => return,
        };

        ctx.report_warning(
            "typescript/no-unnecessary-type-constraint",
            &format!(
                "Unnecessary `extends {constraint_name}` constraint — type parameters default to `{constraint_name}` implicitly"
            ),
            Span::new(param.span.start, param.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUnnecessaryTypeConstraint)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_extends_any() {
        let diags = lint("function f<T extends any>() {}");
        assert_eq!(
            diags.len(),
            1,
            "`T extends any` should be flagged as unnecessary"
        );
    }

    #[test]
    fn test_flags_extends_unknown() {
        let diags = lint("function f<T extends unknown>() {}");
        assert_eq!(
            diags.len(),
            1,
            "`T extends unknown` should be flagged as unnecessary"
        );
    }

    #[test]
    fn test_allows_extends_string() {
        let diags = lint("function f<T extends string>() {}");
        assert!(diags.is_empty(), "`T extends string` should not be flagged");
    }

    #[test]
    fn test_allows_unconstrained() {
        let diags = lint("function f<T>() {}");
        assert!(
            diags.is_empty(),
            "unconstrained type parameter should not be flagged"
        );
    }

    #[test]
    fn test_flags_multiple_params_one_bad() {
        let diags = lint("function f<T extends any, U extends string>() {}");
        assert_eq!(
            diags.len(),
            1,
            "only `T extends any` should be flagged, not `U extends string`"
        );
    }

    #[test]
    fn test_flags_type_alias() {
        let diags = lint("type Box<T extends any> = { value: T };");
        assert_eq!(
            diags.len(),
            1,
            "`extends any` on type alias should be flagged"
        );
    }
}
