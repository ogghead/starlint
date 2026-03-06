//! Rule: `typescript/parameter-properties`
//!
//! Disallow TypeScript parameter properties in class constructors. Parameter
//! properties (e.g. `constructor(public name: string)`) combine parameter
//! declaration and property assignment into one, which can be confusing and
//! makes class structure harder to read at a glance.

use oxc_ast::AstKind;
use oxc_ast::ast::MethodDefinitionKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "typescript/parameter-properties";

/// Flags constructor parameters that use TypeScript parameter properties
/// (accessibility modifiers or `readonly`).
#[derive(Debug)]
pub struct ParameterProperties;

impl NativeRule for ParameterProperties {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow TypeScript parameter properties in class constructors"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::MethodDefinition])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::MethodDefinition(method) = kind else {
            return;
        };

        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }

        for param in &method.value.params.items {
            let has_accessibility = param.accessibility.is_some();
            let has_readonly = param.readonly;

            if has_accessibility || has_readonly {
                ctx.report(Diagnostic {
                    rule_name: RULE_NAME.to_owned(),
                    message: "Unexpected parameter property — declare the property explicitly in the class body instead".to_owned(),
                    span: Span::new(param.span.start, param.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: None,
                    labels: vec![],
                });
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

    /// Helper to lint source code as TypeScript.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ParameterProperties)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_public_parameter_property() {
        let diags = lint("class Foo { constructor(public name: string) {} }");
        assert_eq!(
            diags.len(),
            1,
            "public parameter property should be flagged"
        );
    }

    #[test]
    fn test_flags_private_parameter_property() {
        let diags = lint("class Foo { constructor(private name: string) {} }");
        assert_eq!(
            diags.len(),
            1,
            "private parameter property should be flagged"
        );
    }

    #[test]
    fn test_flags_readonly_parameter_property() {
        let diags = lint("class Foo { constructor(readonly name: string) {} }");
        assert_eq!(
            diags.len(),
            1,
            "readonly parameter property should be flagged"
        );
    }

    #[test]
    fn test_allows_plain_constructor_parameter() {
        let diags = lint("class Foo { constructor(name: string) {} }");
        assert!(
            diags.is_empty(),
            "plain constructor parameter should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_constructor_method() {
        let diags = lint("class Foo { bar(name: string) {} }");
        assert!(
            diags.is_empty(),
            "non-constructor method parameter should not be flagged"
        );
    }
}
