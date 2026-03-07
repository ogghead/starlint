//! Rule: `preserve-caught-error`
//!
//! Require using the caught error variable in `catch` blocks. Swallowing
//! errors silently hides bugs and makes debugging much harder. If the error
//! is genuinely not needed, use `catch {}` (optional catch binding) instead
//! of naming a parameter and ignoring it.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::lint_rule::{LintContext, LintRule};
use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;

/// Flags catch clauses that bind an error parameter but never reference it.
#[derive(Debug)]
pub struct PreserveCaughtError;

/// Check whether a byte is a valid JavaScript identifier character.
const fn is_id_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Check whether `name` appears as a whole-word identifier in `text`.
///
/// Uses byte-level word-boundary detection to avoid false positives from
/// substring matches (e.g. "errors" does not match "error").
fn identifier_appears_in(text: &str, name: &str) -> bool {
    let bytes = text.as_bytes();
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();

    let mut pos: usize = 0;
    while pos.saturating_add(name_len) <= bytes.len() {
        let Some(offset) = text.get(pos..).and_then(|s| s.find(name)) else {
            break;
        };
        let abs = pos.saturating_add(offset);

        // Check character before match
        let before_ok = abs == 0
            || bytes
                .get(abs.wrapping_sub(1))
                .is_none_or(|b| !is_id_char(*b));

        // Check character after match
        let after_pos = abs.saturating_add(name_len);
        let after_ok = bytes.get(after_pos).is_none_or(|b| !is_id_char(*b));

        if before_ok && after_ok {
            return true;
        }
        pos = abs.saturating_add(1);
    }
    false
}

impl LintRule for PreserveCaughtError {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "preserve-caught-error".to_owned(),
            description: "Require using the caught error variable in catch blocks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::CatchClause])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::CatchClause(clause) = node else {
            return;
        };

        // No parameter -> optional catch binding (`catch {}`). That is fine.
        let Some(param_id) = clause.param else {
            return;
        };

        // Only check simple identifier bindings (skip destructured patterns)
        let Some(AstNode::BindingIdentifier(id)) = ctx.node(param_id) else {
            return;
        };

        let param_name = id.name.as_str();

        // Check whether the parameter name appears in the catch body source text
        let body_span = ctx.node(clause.body).map_or(
            starlint_ast::types::Span::EMPTY,
            starlint_ast::AstNode::span,
        );
        let body_start = usize::try_from(body_span.start).unwrap_or(0);
        let body_end = usize::try_from(body_span.end).unwrap_or(0);
        let Some(body_text) = ctx.source_text().get(body_start..body_end) else {
            return;
        };

        if identifier_appears_in(body_text, param_name) {
            return;
        }

        ctx.report(Diagnostic {
            rule_name: "preserve-caught-error".to_owned(),
            message: format!(
                "Caught error `{param_name}` is not used — either handle it or remove the binding"
            ),
            span: Span::new(clause.span.start, clause.span.end),
            severity: Severity::Warning,
            help: None,
            fix: None,
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lint_rule::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreserveCaughtError)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_allows_used_error_log() {
        let diags = lint("try { } catch (e) { console.log(e); }");
        assert!(
            diags.is_empty(),
            "catch that uses the error should not be flagged"
        );
    }

    #[test]
    fn test_flags_unused_error() {
        let diags = lint("try { } catch (e) { console.log('error'); }");
        assert_eq!(
            diags.len(),
            1,
            "catch that does not use the error should be flagged"
        );
    }

    #[test]
    fn test_allows_no_binding() {
        let diags = lint("try { } catch { }");
        assert!(
            diags.is_empty(),
            "catch without binding should not be flagged"
        );
    }

    #[test]
    fn test_allows_throw_error() {
        let diags = lint("try { } catch (e) { throw e; }");
        assert!(
            diags.is_empty(),
            "catch that re-throws the error should not be flagged"
        );
    }

    #[test]
    fn test_flags_empty_catch_body() {
        let diags = lint("try { } catch (e) { }");
        assert_eq!(
            diags.len(),
            1,
            "empty catch body with binding should be flagged"
        );
    }

    #[test]
    fn test_allows_error_in_nested_call() {
        let diags = lint("try { } catch (err) { reportError(err); }");
        assert!(
            diags.is_empty(),
            "error used in function call should not be flagged"
        );
    }

    #[test]
    fn test_word_boundary_no_false_positive() {
        // "errors" is not the same as "error"
        let diags = lint("try { } catch (error) { const errors = []; }");
        assert_eq!(
            diags.len(),
            1,
            "substring match (errors vs error) should not count as usage"
        );
    }

    #[test]
    fn test_allows_error_property_access() {
        let diags = lint("try { } catch (err) { console.log(err.message); }");
        assert!(
            diags.is_empty(),
            "error with property access should not be flagged"
        );
    }
}
