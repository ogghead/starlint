//! Rule: `react/forward-ref-uses-ref`
//!
//! Warn when `React.forwardRef()` is used but the `ref` parameter is not used.

use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, FormalParameters};
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `React.forwardRef()` calls where the callback's second parameter
/// (the `ref`) is missing or unused. If `forwardRef` is used but the ref
/// is not forwarded, it is likely a mistake.
#[derive(Debug)]
pub struct ForwardRefUsesRef;

/// Check whether the callee is `React.forwardRef` or just `forwardRef`.
fn is_forward_ref(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::StaticMemberExpression(member) => {
            member.property.name.as_str() == "forwardRef"
                && matches!(
                    &member.object,
                    Expression::Identifier(id) if id.name.as_str() == "React"
                )
        }
        Expression::Identifier(id) => id.name.as_str() == "forwardRef",
        _ => false,
    }
}

/// Check if a formal parameter list has at least 2 parameters, and the second
/// one is actually named (not just `_` which is conventionally unused).
fn ref_param_is_used(params: &FormalParameters<'_>, source: &str) -> bool {
    if params.items.len() < 2 {
        return false;
    }
    let Some(ref_param) = params.items.get(1) else {
        return false;
    };
    let Ok(start) = usize::try_from(ref_param.span.start) else {
        return false;
    };
    let Ok(end) = usize::try_from(ref_param.span.end) else {
        return false;
    };
    if end > source.len() {
        return false;
    }
    let param_text = &source[start..end];
    let name = param_text.trim();
    // If the parameter is exactly `_`, it's unused by convention
    name != "_"
}

impl NativeRule for ForwardRefUsesRef {
    fn should_run_on_file(&self, source_text: &str, _file_path: &std::path::Path) -> bool {
        source_text.contains("forwardRef")
    }

    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/forward-ref-uses-ref".to_owned(),
            description: "Warn when forwardRef is used but ref parameter is not used".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        if !is_forward_ref(&call.callee) {
            return;
        }

        // forwardRef takes one argument: a callback (props, ref) => ...
        let Some(first_arg) = call.arguments.first() else {
            return;
        };

        let params_missing_ref = match first_arg {
            oxc_ast::ast::Argument::ArrowFunctionExpression(arrow) => {
                !ref_param_is_used(&arrow.params, ctx.source_text())
            }
            oxc_ast::ast::Argument::FunctionExpression(func) => {
                !ref_param_is_used(&func.params, ctx.source_text())
            }
            _ => false,
        };

        if params_missing_ref {
            ctx.report_warning(
                "react/forward-ref-uses-ref",
                "`forwardRef` is used but the `ref` parameter is missing or unused",
                Span::new(call.span.start, call.span.end),
            );
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(ForwardRefUsesRef)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.tsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_forward_ref_without_ref_param() {
        let source = "const Comp = React.forwardRef((props) => <div />);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "forwardRef without ref parameter should be flagged"
        );
    }

    #[test]
    fn test_allows_forward_ref_with_ref_param() {
        let source = "const Comp = React.forwardRef((props, ref) => <div ref={ref} />);";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "forwardRef with ref parameter should not be flagged"
        );
    }

    #[test]
    fn test_flags_forward_ref_with_unused_ref() {
        let source = "const Comp = forwardRef((props, _) => <div />);";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "forwardRef with underscore ref should be flagged"
        );
    }
}
