//! Rule: `react/no-this-in-sfc`
//!
//! Warn when `this` is used in a stateless functional component.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `this` expressions that appear inside arrow functions or regular
/// functions (not class methods) that contain JSX, indicating a likely
/// stateless functional component incorrectly using `this`.
#[derive(Debug)]
pub struct NoThisInSfc;

/// Check whether the source text in a given byte range contains JSX-like patterns.
fn region_has_jsx(source_bytes: &[u8], start: usize, end: usize) -> bool {
    if end > source_bytes.len() || start >= end {
        return false;
    }
    let Some(region) = source_bytes.get(start..end) else {
        return false;
    };
    for (i, &b) in region.iter().enumerate() {
        if b == b'<' {
            if let Some(&next) = region.get(i.saturating_add(1)) {
                if next.is_ascii_alphabetic() || next == b'>' {
                    return true;
                }
            }
        }
    }
    false
}

impl NativeRule for NoThisInSfc {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-this-in-sfc".to_owned(),
            description: "Warn when `this` is used in a stateless functional component".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ThisExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ThisExpression(this_expr) = kind else {
            return;
        };

        let source = ctx.source_text();
        let this_pos = usize::try_from(this_expr.span.start).unwrap_or(0);

        // Heuristic: check if this `this` expression is inside a function
        // (not a class method) that returns JSX.
        let before = &source[..this_pos];

        let last_arrow = before.rfind("=>");
        let last_function = before.rfind("function");
        let last_class = before.rfind("class ");

        // Determine the nearest function boundary
        let nearest_func = match (last_arrow, last_function) {
            (Some(a), Some(f)) => Some(a.max(f)),
            (Some(a), None) => Some(a),
            (None, Some(f)) => Some(f),
            (None, None) => None,
        };

        let Some(nearest_func_pos) = nearest_func else {
            return;
        };

        // If a class keyword appears after the function boundary, this is a class method
        if let Some(class_pos) = last_class {
            if class_pos > nearest_func_pos {
                return;
            }
        }

        // Check if the enclosing function body has JSX
        let search_end = this_pos.saturating_add(500).min(source.len());
        let source_bytes = source.as_bytes();
        if region_has_jsx(source_bytes, nearest_func_pos, search_end) {
            ctx.report(Diagnostic {
                rule_name: "react/no-this-in-sfc".to_owned(),
                message: "Unexpected `this` in a stateless functional component".to_owned(),
                span: Span::new(this_expr.span.start, this_expr.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoThisInSfc)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_this_in_arrow_sfc() {
        let source = "const Comp = () => <div>{this.props.name}</div>;";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "this in arrow SFC should be flagged");
    }

    #[test]
    fn test_allows_this_in_class_method() {
        let source = "class Comp extends React.Component { render() { return <div>{this.props.name}</div>; } }";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "this in class component method should not be flagged"
        );
    }

    #[test]
    fn test_allows_no_this() {
        let source = "const Comp = (props) => <div>{props.name}</div>;";
        let diags = lint(source);
        assert!(diags.is_empty(), "SFC without this should not be flagged");
    }
}
