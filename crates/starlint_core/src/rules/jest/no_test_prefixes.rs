//! Rule: `jest/no-test-prefixes`
//!
//! Suggest using `test.skip`/`test.only` instead of `xtest`/`ftest` prefixes.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-test-prefixes";

/// Mapping of prefixed names to their preferred forms.
const PREFIX_MAP: &[(&str, &str)] = &[
    ("xdescribe", "describe.skip"),
    ("xtest", "test.skip"),
    ("xit", "it.skip"),
    ("fdescribe", "describe.only"),
    ("ftest", "test.only"),
    ("fit", "it.only"),
];

/// Flags shorthand test prefixes like `xtest`/`fit` and suggests `.skip`/`.only`.
#[derive(Debug)]
pub struct NoTestPrefixes;

impl NativeRule for NoTestPrefixes {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Use `.skip`/`.only` instead of `x`/`f` test prefixes".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let callee_name = match &call.callee {
            Expression::Identifier(id) => id.name.as_str(),
            _ => return,
        };

        for (prefix, replacement) in PREFIX_MAP {
            if callee_name == *prefix {
                ctx.report_warning(
                    RULE_NAME,
                    &format!("Use `{replacement}` instead of `{prefix}`"),
                    Span::new(call.span.start, call.span.end),
                );
                return;
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoTestPrefixes)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_xtest() {
        let diags = lint("xtest('skipped', () => {});");
        assert_eq!(diags.len(), 1, "`xtest` should be flagged");
    }

    #[test]
    fn test_flags_fit() {
        let diags = lint("fit('focused', () => {});");
        assert_eq!(diags.len(), 1, "`fit` should be flagged");
    }

    #[test]
    fn test_allows_regular_test() {
        let diags = lint("test('normal', () => {});");
        assert!(diags.is_empty(), "regular `test` should not be flagged");
    }
}
