//! Rule: `react/no-string-refs`
//!
//! Warn when string refs are used (`ref="myRef"`). String refs are deprecated.

use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeName, JSXAttributeValue};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::fix_builder::FixBuilder;
use crate::fix_utils;
use crate::rule::{NativeLintContext, NativeRule};

/// Flags usage of string refs like `ref="myRef"`.
#[derive(Debug)]
pub struct NoStringRefs;

impl NativeRule for NoStringRefs {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-string-refs".to_owned(),
            description: "Disallow using string refs (deprecated)".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
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

        if name != "ref" {
            return;
        }

        // Only flag when the value is a string literal
        if let Some(JSXAttributeValue::StringLiteral(_)) = &attr.value {
            let attr_span = Span::new(attr.span.start, attr.span.end);
            let fix = FixBuilder::new("Remove string ref", FixKind::SuggestionFix)
                .edit(fix_utils::remove_jsx_attr(ctx.source_text(), attr_span))
                .build();
            ctx.report(Diagnostic {
                rule_name: "react/no-string-refs".to_owned(),
                message: "String refs are deprecated — use `useRef` or callback refs instead"
                    .to_owned(),
                span: attr_span,
                severity: Severity::Warning,
                help: None,
                fix,
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoStringRefs)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_string_ref() {
        let diags = lint(r#"const x = <div ref="myRef" />;"#);
        assert_eq!(diags.len(), 1, "should flag string ref");
    }

    #[test]
    fn test_allows_callback_ref() {
        let diags = lint(r"const x = <div ref={myRef} />;");
        assert!(diags.is_empty(), "callback ref should not be flagged");
    }

    #[test]
    fn test_allows_non_ref_prop() {
        let diags = lint(r#"const x = <div id="myDiv" />;"#);
        assert!(diags.is_empty(), "non-ref props should not be flagged");
    }
}
