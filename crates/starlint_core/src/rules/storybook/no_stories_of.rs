//! Rule: `storybook/no-stories-of`
//!
//! `storiesOf` is deprecated and should not be used.

use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/no-stories-of";

/// `storiesOf` is deprecated and should not be used.
#[derive(Debug)]
pub struct NoStoriesOf;

impl NativeRule for NoStoriesOf {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description: "`storiesOf` is deprecated and should not be used".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::CallExpression])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::CallExpression(call) = kind else {
            return;
        };

        let is_stories_of = match &call.callee {
            Expression::Identifier(ident) => ident.name.as_str() == "storiesOf",
            _ => false,
        };

        if is_stories_of {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "`storiesOf` is deprecated — use CSF (Component Story Format) instead"
                    .to_owned(),
                span: Span::new(call.span.start, call.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("Button.stories.tsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoStoriesOf)];
            traverse_and_lint(
                &parsed.program,
                &rules,
                source,
                Path::new("Button.stories.tsx"),
            )
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_stories_of() {
        let diags = lint("storiesOf('Button', module).add('default', () => {});");
        assert_eq!(diags.len(), 1, "should flag storiesOf call");
    }

    #[test]
    fn test_allows_csf() {
        let diags = lint("export default { title: 'Button' }; export const Default = {};");
        assert!(diags.is_empty(), "should allow CSF format");
    }

    #[test]
    fn test_allows_other_calls() {
        let diags = lint("someFunction('Button');");
        assert!(diags.is_empty(), "should allow other function calls");
    }
}
