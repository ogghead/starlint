//! Rule: `preserve-caught-error`
//!
//! Require using the caught error variable in `catch` blocks. Swallowing
//! errors silently hides bugs and makes debugging much harder. If the error
//! is genuinely not needed, use `catch {}` (optional catch binding) instead
//! of naming a parameter and ignoring it.

use oxc_ast::AstKind;
use oxc_ast::ast::BindingPattern;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

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

impl NativeRule for PreserveCaughtError {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "preserve-caught-error".to_owned(),
            description: "Require using the caught error variable in catch blocks".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CatchClause])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CatchClause(clause) = kind else {
            return;
        };

        // No parameter → optional catch binding (`catch {}`). That is fine.
        let Some(param) = &clause.param else {
            return;
        };

        // Only check simple identifier bindings (skip destructured patterns)
        let BindingPattern::BindingIdentifier(id) = &param.pattern else {
            return;
        };

        let param_name = id.name.as_str();

        // Check whether the parameter name appears in the catch body source text
        let body_start = usize::try_from(clause.body.span.start).unwrap_or(0);
        let body_end = usize::try_from(clause.body.span.end).unwrap_or(0);
        let Some(body_text) = ctx.source_text().get(body_start..body_end) else {
            return;
        };

        if identifier_appears_in(body_text, param_name) {
            return;
        }

        ctx.report_warning(
            "preserve-caught-error",
            &format!(
                "Caught error `{param_name}` is not used — either handle it or remove the binding"
            ),
            Span::new(clause.span.start, clause.span.end),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    /// Helper to lint source code.
    fn lint(source: &str) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(PreserveCaughtError)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
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
