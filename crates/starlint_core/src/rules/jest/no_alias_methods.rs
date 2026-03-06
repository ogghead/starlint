//! Rule: `jest/no-alias-methods`
//!
//! Suggest replacing deprecated Jest matcher aliases with their canonical forms.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "jest/no-alias-methods";

/// Alias → canonical method mappings.
const ALIASES: &[(&str, &str)] = &[
    ("toBeCalled", "toHaveBeenCalled"),
    ("toBeCalledWith", "toHaveBeenCalledWith"),
    ("lastCalledWith", "toHaveBeenLastCalledWith"),
    ("nthCalledWith", "toHaveBeenNthCalledWith"),
    ("toReturn", "toHaveReturned"),
    ("toReturnWith", "toHaveReturnedWith"),
    ("lastReturnedWith", "toHaveLastReturnedWith"),
    ("nthReturnedWith", "toHaveNthReturnedWith"),
    ("toReturnTimes", "toHaveReturnedTimes"),
    ("toBeCalledTimes", "toHaveBeenCalledTimes"),
];

/// Flags deprecated Jest matcher aliases.
#[derive(Debug)]
pub struct NoAliasMethods;

impl NativeRule for NoAliasMethods {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Replace deprecated Jest matcher aliases with canonical forms".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check for member expression calls like `expect(x).toBeCalled()`
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        let method_name = member.property.name.as_str();

        for (alias, canonical) in ALIASES {
            if method_name == *alias {
                let prop_span = Span::new(member.property.span.start, member.property.span.end);
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: format!("Use `{canonical}` instead of deprecated `{alias}`"),
                    span: Span::new(call.span.start, call.span.end),
                    severity: Severity::Warning,
                    help: Some(format!("Replace `{alias}` with `{canonical}`")),
                    fix: Some(Fix {
                        message: format!("Replace with `{canonical}`"),
                        edits: vec![Edit {
                            span: prop_span,
                            replacement: (*canonical).to_owned(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoAliasMethods)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_to_be_called() {
        let diags = lint("expect(fn).toBeCalled();");
        assert_eq!(diags.len(), 1, "`toBeCalled` should be flagged");
    }

    #[test]
    fn test_flags_to_return() {
        let diags = lint("expect(fn).toReturn();");
        assert_eq!(diags.len(), 1, "`toReturn` should be flagged");
    }

    #[test]
    fn test_allows_canonical_method() {
        let diags = lint("expect(fn).toHaveBeenCalled();");
        assert!(diags.is_empty(), "`toHaveBeenCalled` should not be flagged");
    }
}
