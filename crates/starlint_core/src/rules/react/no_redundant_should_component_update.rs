//! Rule: `react/no-redundant-should-component-update`
//!
//! Flag `shouldComponentUpdate` when extending `PureComponent`. `PureComponent`
//! already implements a shallow comparison in `shouldComponentUpdate`, so
//! defining it again is redundant and likely a mistake.

use oxc_ast::AstKind;
use oxc_ast::ast::{ClassElement, Expression, PropertyKey};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `shouldComponentUpdate` in classes extending `PureComponent`.
#[derive(Debug)]
pub struct NoRedundantShouldComponentUpdate;

impl NativeRule for NoRedundantShouldComponentUpdate {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-redundant-should-component-update".to_owned(),
            description: "Disallow `shouldComponentUpdate` when extending `PureComponent`"
                .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Class])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Class(class) = kind else {
            return;
        };

        // Check if the class extends PureComponent or React.PureComponent
        if !extends_pure_component(class) {
            return;
        }

        for element in &class.body.body {
            let ClassElement::MethodDefinition(method) = element else {
                continue;
            };

            let method_name = match &method.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                _ => continue,
            };

            if method_name == "shouldComponentUpdate" {
                ctx.report(Diagnostic {
                    rule_name: "react/no-redundant-should-component-update".to_owned(),
                    message: "`shouldComponentUpdate` is redundant when extending `PureComponent`"
                        .to_owned(),
                    span: Span::new(method.span.start, method.span.end),
                    severity: Severity::Warning,
                    help: None,
                    fix: Some(Fix {
                        kind: FixKind::SuggestionFix,
                        message: "Remove redundant `shouldComponentUpdate` method".to_owned(),
                        edits: vec![Edit {
                            span: Span::new(method.span.start, method.span.end),
                            replacement: String::new(),
                        }],
                        is_snippet: false,
                    }),
                    labels: vec![],
                });
            }
        }
    }
}

/// Check whether a class extends `PureComponent` or `React.PureComponent`.
fn extends_pure_component(class: &oxc_ast::ast::Class<'_>) -> bool {
    let Some(super_class) = &class.super_class else {
        return false;
    };

    match super_class {
        // class Foo extends PureComponent
        Expression::Identifier(ident) => ident.name.as_str() == "PureComponent",
        // class Foo extends React.PureComponent
        Expression::StaticMemberExpression(member) => {
            member.property.name.as_str() == "PureComponent"
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.jsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoRedundantShouldComponentUpdate)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_should_component_update_in_pure_component() {
        let source = r"
class MyComponent extends React.PureComponent {
    shouldComponentUpdate() {
        return true;
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "shouldComponentUpdate in PureComponent should be flagged"
        );
    }

    #[test]
    fn test_flags_bare_pure_component() {
        let source = r"
class MyComponent extends PureComponent {
    shouldComponentUpdate() {
        return true;
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "shouldComponentUpdate in bare PureComponent should be flagged"
        );
    }

    #[test]
    fn test_allows_in_regular_component() {
        let source = r"
class MyComponent extends React.Component {
    shouldComponentUpdate() {
        return true;
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "shouldComponentUpdate in Component should not be flagged"
        );
    }

    #[test]
    fn test_allows_pure_component_without_should_update() {
        let source = r"
class MyComponent extends React.PureComponent {
    render() {
        return null;
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "PureComponent without shouldComponentUpdate should not be flagged"
        );
    }
}
