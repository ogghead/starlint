//! Rule: `react/no-children-prop`
//!
//! Warn when passing `children` as a prop rather than nesting children inside the element.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXAttributeName;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of `children` as a JSX prop.
#[derive(Debug)]
pub struct NoChildrenProp;

impl NativeRule for NoChildrenProp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-children-prop".to_owned(),
            description: "Disallow passing `children` as a prop".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXAttribute])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXAttribute(attr) = kind else {
            return;
        };

        let name = match &attr.name {
            JSXAttributeName::Identifier(id) => id.name.as_str(),
            JSXAttributeName::NamespacedName(_) => return,
        };

        if name == "children" {
            ctx.report(Diagnostic {
                rule_name: "react/no-children-prop".to_owned(),
                message: "Do not pass `children` as a prop — nest children between opening and closing tags instead".to_owned(),
                span: Span::new(attr.span.start, attr.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoChildrenProp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_children_prop() {
        let diags = lint(r#"const x = <div children="hello" />;"#);
        assert_eq!(diags.len(), 1, "should flag children prop");
    }

    #[test]
    fn test_allows_nested_children() {
        let diags = lint(r"const x = <div>hello</div>;");
        assert!(diags.is_empty(), "nested children should not be flagged");
    }

    #[test]
    fn test_flags_children_expression() {
        let diags = lint(r"const x = <Comp children={<span />} />;");
        assert_eq!(
            diags.len(),
            1,
            "should flag children prop with expression value"
        );
    }
}
