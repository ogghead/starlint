//! Rule: `prefer-optional-catch-binding`
//!
//! Flag catch clauses with unused parameters. If the caught error is never
//! referenced in the catch body, the parameter can be omitted: `catch {}`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags catch clauses whose parameter is never used in the body.
#[derive(Debug)]
pub struct PreferOptionalCatchBinding;

/// Check whether a byte is a valid JavaScript identifier character.
const fn is_id_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Check whether `name` appears as a whole-word identifier in `text`.
///
/// Uses byte-level word-boundary detection. False matches inside string
/// literals or comments cause false negatives (missed flags), never false
/// positives. This is the safe direction — we never remove a used parameter.
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

impl LintRule for PreferOptionalCatchBinding {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "prefer-optional-catch-binding".to_owned(),
            description: "Prefer omitting unused catch binding".to_owned(),
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

        // Must have a parameter
        let Some(param_id) = clause.param else {
            return;
        };

        // Only check simple identifier params
        let Some(AstNode::BindingIdentifier(id)) = ctx.node(param_id) else {
            return;
        };

        let param_name = id.name.as_str();
        let param_span = id.span;

        // Search the catch body source text for the parameter name
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

        // Parameter is unused — suggest removing it.
        // Replace `catch (err) {` with `catch {`.
        ctx.report(Diagnostic {
            rule_name: "prefer-optional-catch-binding".to_owned(),
            message: format!("Catch binding `{param_name}` is unused"),
            span: Span::new(param_span.start, param_span.end),
            severity: Severity::Warning,
            help: Some("Remove the unused catch binding".to_owned()),
            fix: Some(Fix {
                kind: FixKind::SafeFix,
                message: "Remove unused catch binding".to_owned(),
                edits: vec![Edit {
                    span: Span::new(clause.span.start, body_span.start),
                    replacement: "catch ".to_owned(),
                }],
                is_snippet: false,
            }),
            labels: vec![],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(PreferOptionalCatchBinding)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_empty_catch_body() {
        let diags = lint("try {} catch (err) {}");
        assert_eq!(diags.len(), 1, "should flag unused param in empty body");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("catch "),
            "fix should remove the binding"
        );
    }

    #[test]
    fn test_flags_unused_param() {
        let diags = lint("try {} catch (err) { console.log('failed'); }");
        assert_eq!(diags.len(), 1, "should flag unused param");
    }

    #[test]
    fn test_allows_used_param_throw() {
        let diags = lint("try {} catch (err) { throw err; }");
        assert!(diags.is_empty(), "used param should not be flagged");
    }

    #[test]
    fn test_allows_used_param_call() {
        let diags = lint("try {} catch (err) { log(err); }");
        assert!(diags.is_empty(), "used param in call should not be flagged");
    }

    #[test]
    fn test_allows_no_param() {
        let diags = lint("try {} catch {}");
        assert!(diags.is_empty(), "no param should not be flagged");
    }

    #[test]
    fn test_allows_destructured() {
        let diags = lint("try {} catch ({ message }) { log(message); }");
        assert!(diags.is_empty(), "destructured should not be flagged");
    }

    #[test]
    fn test_word_boundary_distinguishes_substrings() {
        // "errors" contains "error" but is a different identifier — param is unused
        let diags = lint("try {} catch (error) { const errors = []; }");
        assert_eq!(
            diags.len(),
            1,
            "should flag 'error' as unused when only 'errors' appears"
        );
    }

    #[test]
    fn test_identifier_helper() {
        assert!(identifier_appears_in("throw err;", "err"));
        assert!(!identifier_appears_in("throw error;", "err"));
        assert!(identifier_appears_in("log(err)", "err"));
        assert!(!identifier_appears_in("errors.push(1)", "error"));
        assert!(identifier_appears_in("error.message", "error"));
        assert!(identifier_appears_in("{error}", "error"));
    }
}
