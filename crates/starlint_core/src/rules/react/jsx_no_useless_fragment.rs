//! Rule: `react/jsx-no-useless-fragment`
//!
//! Warn when `<>child</>` or `<React.Fragment>child</React.Fragment>` wraps
//! only a single child element.

use oxc_ast::AstKind;
use oxc_ast::ast::JSXChild;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "react/jsx-no-useless-fragment";

/// Flags fragments (`<>...</>`) that wrap a single child, which is unnecessary.
#[derive(Debug)]
pub struct JsxNoUselessFragment;

/// Count meaningful children (skip whitespace-only text nodes).
fn meaningful_children_count(children: &[JSXChild<'_>]) -> usize {
    children
        .iter()
        .filter(|child| {
            if let JSXChild::Text(text) = child {
                // Skip whitespace-only text nodes
                !text.value.as_str().trim().is_empty()
            } else {
                true
            }
        })
        .count()
}

impl NativeRule for JsxNoUselessFragment {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "Disallow unnecessary fragments".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SafeFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXFragment])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::JSXFragment(fragment) = kind else {
            return;
        };

        let count = meaningful_children_count(&fragment.children);
        if count <= 1 {
            let fragment_span = Span::new(fragment.span.start, fragment.span.end);

            // Build a fix for single-child case by extracting the child's source text
            let fix = if count == 1 {
                let source = ctx.source_text();
                fragment
                    .children
                    .iter()
                    .find(|child| {
                        if let JSXChild::Text(text) = child {
                            !text.value.as_str().trim().is_empty()
                        } else {
                            true
                        }
                    })
                    .map(|child| {
                        let child_span = child.span();
                        let start = usize::try_from(child_span.start).unwrap_or(0);
                        let end = usize::try_from(child_span.end).unwrap_or(0);
                        let child_text = source.get(start..end).unwrap_or("");
                        Fix {
                            message: "Remove the enclosing fragment".to_owned(),
                            edits: vec![Edit {
                                span: fragment_span,
                                replacement: child_text.to_owned(),
                            }],
                            is_snippet: false,
                        }
                    })
            } else {
                None
            };

            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Unnecessary fragment: fragments with a single child (or no children) can be removed".to_owned(),
                span: fragment_span,
                severity: Severity::Warning,
                help: Some("Remove the unnecessary fragment wrapper".to_owned()),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(JsxNoUselessFragment)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_single_child_fragment() {
        let diags = lint("const el = <><div /></>;");
        assert_eq!(diags.len(), 1, "should flag fragment with single child");
        assert_eq!(diags.first().map(|d| d.rule_name.as_str()), Some(RULE_NAME));
    }

    #[test]
    fn test_flags_empty_fragment() {
        let diags = lint("const el = <></>;");
        assert_eq!(diags.len(), 1, "should flag empty fragment");
    }

    #[test]
    fn test_allows_multiple_children() {
        let diags = lint("const el = <><div /><span /></>;");
        assert!(
            diags.is_empty(),
            "should not flag fragment with multiple children"
        );
    }
}
