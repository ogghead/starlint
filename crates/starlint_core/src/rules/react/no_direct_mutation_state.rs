//! Rule: `react/no-direct-mutation-state`
//!
//! Disallow direct mutation of `this.state`. Mutating state directly does not
//! trigger a re-render and leads to stale UI. Always use `setState()`.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `this.state = ...` assignments.
#[derive(Debug)]
pub struct NoDirectMutationState;

impl NativeRule for NoDirectMutationState {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-direct-mutation-state".to_owned(),
            description: "Disallow direct mutation of `this.state`".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Error,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::AssignmentExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::AssignmentExpression(assign) = kind else {
            return;
        };

        // Check if the left side is this.state via a member expression.
        // AssignmentTarget inherits SimpleAssignmentTarget which inherits MemberExpression.
        // We check for StaticMemberExpression pattern: this.state
        if is_this_state_target(&assign.left) {
            ctx.report(Diagnostic {
                rule_name: "react/no-direct-mutation-state".to_owned(),
                message: "Do not mutate `this.state` directly — use `setState()` instead"
                    .to_owned(),
                span: Span::new(assign.span.start, assign.span.end),
                severity: Severity::Error,
                help: None,
                fix: None,
                labels: vec![],
            });
        }
    }
}

/// Check if an assignment target is `this.state`.
fn is_this_state_target(target: &oxc_ast::ast::AssignmentTarget<'_>) -> bool {
    // AssignmentTarget can contain StaticMemberExpression via inheritance
    match target {
        oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) => {
            member.property.name.as_str() == "state"
                && matches!(&member.object, Expression::ThisExpression(_))
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoDirectMutationState)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_direct_state_mutation() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.state = { count: 1 };
    }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "direct this.state assignment should be flagged"
        );
    }

    #[test]
    fn test_allows_set_state() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.setState({ count: 1 });
    }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "setState call should not be flagged");
    }

    #[test]
    fn test_allows_other_this_assignment() {
        let source = r"
class MyComponent extends React.Component {
    handleClick() {
        this.value = 42;
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "this.value assignment should not be flagged"
        );
    }
}
