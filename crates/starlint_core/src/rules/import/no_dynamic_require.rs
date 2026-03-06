//! Rule: `import/no-dynamic-require`
//!
//! Forbid `require()` calls with expressions (non-literal arguments).
//! Dynamic requires make it hard for bundlers and static analysis tools to
//! determine the dependency graph.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `require()` calls whose argument is not a string literal.
#[derive(Debug)]
pub struct NoDynamicRequire;

impl NativeRule for NoDynamicRequire {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-dynamic-require".to_owned(),
            description: "Forbid `require()` calls with expressions".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        // Check if callee is `require`
        let is_require = matches!(
            &call.callee,
            Expression::Identifier(id) if id.name.as_str() == "require"
        );

        if !is_require {
            return;
        }

        // Check if the first argument is a string literal
        let first_arg = call.arguments.first();
        let is_literal =
            first_arg.is_some_and(|arg| matches!(arg, oxc_ast::ast::Argument::StringLiteral(_)));

        if !is_literal {
            ctx.report(Diagnostic {
                rule_name: "import/no-dynamic-require".to_owned(),
                message: "Calls to `require()` should use a string literal argument".to_owned(),
                span: Span::new(call.span.start, call.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDynamicRequire)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_dynamic_require() {
        let diags = lint(r"const mod = require(name);");
        assert_eq!(
            diags.len(),
            1,
            "dynamic require with variable should be flagged"
        );
    }

    #[test]
    fn test_flags_template_literal_require() {
        let diags = lint(r"const mod = require(`./path/${name}`);");
        assert_eq!(
            diags.len(),
            1,
            "dynamic require with template literal should be flagged"
        );
    }

    #[test]
    fn test_allows_static_require() {
        let diags = lint(r#"const mod = require("lodash");"#);
        assert!(diags.is_empty(), "static require should not be flagged");
    }
}
