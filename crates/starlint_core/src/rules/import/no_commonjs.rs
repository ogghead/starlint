//! Rule: `import/no-commonjs`
//!
//! Disallow `CommonJS` `require()` calls and `module.exports` / `exports`
//! assignments. Encourages use of ES module syntax instead.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `CommonJS` `require()` calls and `module.exports` usage.
#[derive(Debug)]
pub struct NoCommonjs;

impl NativeRule for NoCommonjs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-commonjs".to_owned(),
            description: "Disallow CommonJS require/module.exports".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression, AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::CallExpression(call) => {
                // Check for require('...')
                let is_require = matches!(
                    &call.callee,
                    Expression::Identifier(id) if id.name.as_str() == "require"
                );

                if is_require {
                    // Only flag if the first argument is a string literal (standard require)
                    let has_string_arg = call
                        .arguments
                        .first()
                        .is_some_and(|arg| matches!(arg, oxc_ast::ast::Argument::StringLiteral(_)));

                    if has_string_arg {
                        ctx.report(Diagnostic {
                            rule_name: "import/no-commonjs".to_owned(),
                            message: "Use ES module import instead of CommonJS require()"
                                .to_owned(),
                            span: Span::new(call.span.start, call.span.end),
                            severity: Severity::Warning,
                            help: None,
                            fix: None,
                            labels: vec![],
                        });
                    }
                }
            }
            AstKind::AssignmentExpression(assign) => {
                // Check for module.exports = ... or exports.foo = ...
                let is_module_exports = match &assign.left {
                    oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) => {
                        is_module_exports_member(member)
                    }
                    _ => false,
                };

                if is_module_exports {
                    ctx.report(Diagnostic {
                        rule_name: "import/no-commonjs".to_owned(),
                        message: "Use ES module export instead of CommonJS module.exports"
                            .to_owned(),
                        span: Span::new(assign.span.start, assign.span.end),
                        severity: Severity::Warning,
                        help: None,
                        fix: None,
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Check if a member expression is `module.exports` or `exports.<name>`.
fn is_module_exports_member(member: &oxc_ast::ast::StaticMemberExpression<'_>) -> bool {
    let prop_name = member.property.name.as_str();

    match &member.object {
        Expression::Identifier(id) => {
            let obj_name = id.name.as_str();
            // module.exports = ...
            (obj_name == "module" && prop_name == "exports")
            // exports.foo = ...
            || obj_name == "exports"
        }
        _ => false,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoCommonjs)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_require_call() {
        let diags = lint(r"const foo = require('foo');");
        assert_eq!(diags.len(), 1, "CommonJS require should be flagged");
    }

    #[test]
    fn test_flags_module_exports() {
        let diags = lint("module.exports = {};");
        assert_eq!(
            diags.len(),
            1,
            "module.exports assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_es_import() {
        let diags = lint(r#"import foo from "foo";"#);
        assert!(diags.is_empty(), "ES import should not be flagged");
    }
}
