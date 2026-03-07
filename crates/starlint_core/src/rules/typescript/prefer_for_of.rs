//! Rule: `typescript/prefer-for-of`
//!
//! Prefer `for...of` loops over index-based `for` loops when the loop counter
//! is only used to access array elements. A `for (let i = 0; i < arr.length; i++)`
//! pattern can often be replaced with the cleaner `for (const item of arr)`.
//!
//! This rule flags `ForStatement` nodes that follow the classic indexed loop
//! pattern: numeric initializer at `0`, a `.length` comparison test, and
//! an increment (`++`) update.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::operator::{BinaryOperator, UpdateOperator};
use starlint_ast::types::NodeId;

/// Flags classic index-based `for` loops that could use `for...of`.
#[derive(Debug)]
pub struct PreferForOf;

impl LintRule for PreferForOf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "typescript/prefer-for-of".to_owned(),
            description: "Prefer `for...of` loops over index-based `for` loops".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ForStatement])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ForStatement(stmt) = node else {
            return;
        };

        // Step 1: init must be `let i = 0` (variable declaration with numeric literal 0)
        let Some(init_id) = stmt.init else {
            return;
        };
        let Some(counter_name) = extract_zero_init(init_id, ctx) else {
            return;
        };

        // Step 2: test must be `i < <something>.length`
        let Some(test_id) = stmt.test else {
            return;
        };
        if !is_length_comparison(test_id, &counter_name, ctx) {
            return;
        }

        // Step 3: update must be `i++` or `++i`
        let Some(update_id) = stmt.update else {
            return;
        };
        if !is_increment(update_id, &counter_name, ctx) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "typescript/prefer-for-of".to_owned(),
            message: "This index-based `for` loop can be replaced with a `for...of` loop"
                .to_owned(),
            span: Span::new(stmt.span.start, stmt.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

/// Check if the `for` loop init is a `VariableDeclaration` with a single
/// declarator initialized to `0`. Returns the counter variable name if so.
fn extract_zero_init(init_id: NodeId, ctx: &LintContext<'_>) -> Option<String> {
    let AstNode::VariableDeclaration(decl) = ctx.node(init_id)? else {
        return None;
    };

    let &first_decl_id = decl.declarations.first()?;
    let AstNode::VariableDeclarator(declarator) = ctx.node(first_decl_id)? else {
        return None;
    };

    // Init must be `0`
    let init_node_id = declarator.init?;
    let AstNode::NumericLiteral(lit) = ctx.node(init_node_id)? else {
        return None;
    };

    #[allow(clippy::float_cmp)]
    if lit.value != 0.0 {
        return None;
    }

    // Binding must be a simple identifier
    let AstNode::BindingIdentifier(ident) = ctx.node(declarator.id)? else {
        return None;
    };

    Some(ident.name.clone())
}

/// Check if the test expression is `counter < something.length`.
fn is_length_comparison(test_id: NodeId, counter_name: &str, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::BinaryExpression(bin)) = ctx.node(test_id) else {
        return false;
    };

    if bin.operator != BinaryOperator::LessThan {
        return false;
    }

    // Left side must be our counter variable
    let Some(AstNode::IdentifierReference(left_id)) = ctx.node(bin.left) else {
        return false;
    };
    if left_id.name.as_str() != counter_name {
        return false;
    }

    // Right side must be `<something>.length`
    let Some(AstNode::StaticMemberExpression(member)) = ctx.node(bin.right) else {
        return false;
    };

    member.property.as_str() == "length"
}

/// Check if the update expression is `counter++` or `++counter`.
fn is_increment(update_id: NodeId, counter_name: &str, ctx: &LintContext<'_>) -> bool {
    let Some(AstNode::UpdateExpression(upd)) = ctx.node(update_id) else {
        return false;
    };

    if upd.operator != UpdateOperator::Increment {
        return false;
    }

    // The argument of an update expression is the identifier being updated
    let Some(AstNode::IdentifierReference(ident)) = ctx.node(upd.argument) else {
        return false;
    };

    ident.name.as_str() == counter_name
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferForOf)];
        lint_source(source, "test.ts", &rules)
    }

    #[test]
    fn test_flags_classic_indexed_loop() {
        let diags = lint("const arr = [1]; for (let i = 0; i < arr.length; i++) { arr[i]; }");
        assert_eq!(diags.len(), 1, "classic indexed for loop should be flagged");
    }

    #[test]
    fn test_allows_for_of() {
        let diags = lint("const arr = [1]; for (const x of arr) {}");
        assert!(diags.is_empty(), "`for...of` should not be flagged");
    }

    #[test]
    fn test_allows_non_length_bound() {
        let diags = lint("for (let i = 0; i < 10; i++) {}");
        assert!(
            diags.is_empty(),
            "loop with numeric bound (not `.length`) should not be flagged"
        );
    }

    #[test]
    fn test_allows_non_zero_init() {
        let diags = lint("const arr = [1]; for (let i = 1; i < arr.length; i++) {}");
        assert!(diags.is_empty(), "loop starting at 1 should not be flagged");
    }

    #[test]
    fn test_allows_decrement_update() {
        let diags = lint("const arr = [1]; for (let i = 0; i < arr.length; i--) {}");
        assert!(
            diags.is_empty(),
            "loop with decrement update should not be flagged"
        );
    }

    #[test]
    fn test_flags_prefix_increment() {
        let diags = lint("const arr = [1]; for (let i = 0; i < arr.length; ++i) { arr[i]; }");
        assert_eq!(diags.len(), 1, "prefix increment should also be flagged");
    }
}
