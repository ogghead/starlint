//! Rule: `storybook/use-storybook-testing-library`
//!
//! Do not import testing-library directly in stories, use `@storybook/test`.
//! Matches `ImportDeclaration` for `@testing-library/`.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/use-storybook-testing-library";

/// Do not import testing-library directly in stories.
#[derive(Debug)]
pub struct UseStorybookTestingLibrary;

impl LintRule for UseStorybookTestingLibrary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Do not import testing-library directly in stories — use `@storybook/test`"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let file_name = ctx.file_path().to_string_lossy();
        if !file_name.contains(".stories.") && !file_name.contains(".story.") {
            return;
        }

        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        if import.source.starts_with("@testing-library/") {
            // Find the source string position in the import declaration to target the fix
            let source_text = ctx.source_text();
            let span_start = usize::try_from(import.span.start).unwrap_or(0);
            let span_end = usize::try_from(import.span.end).unwrap_or(0);
            let import_text = source_text.get(span_start..span_end).unwrap_or("");
            // Find the quoted source in the import text
            let fix = import_text.find(&import.source).map(|offset| {
                let abs_start = import
                    .span
                    .start
                    .saturating_add(u32::try_from(offset).unwrap_or(0));
                let abs_end =
                    abs_start.saturating_add(u32::try_from(import.source.len()).unwrap_or(0));
                Fix {
                    kind: FixKind::SuggestionFix,
                    message: "Replace import source with `@storybook/test`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(abs_start, abs_end),
                        replacement: "@storybook/test".to_owned(),
                    }],
                    is_snippet: false,
                }
            });
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Import from `@storybook/test` instead of `@testing-library/` directly"
                    .to_owned(),
                span: Span::new(import.span.start, import.span.end),
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
    use super::*;
    starlint_rule_framework::lint_rule_test!(UseStorybookTestingLibrary, "Button.stories.ts");

    #[test]
    fn test_flags_testing_library_import() {
        let diags = lint("import { render } from '@testing-library/react';");
        assert_eq!(diags.len(), 1, "should flag direct testing-library import");
    }

    #[test]
    fn test_allows_storybook_test_import() {
        let diags = lint("import { within } from '@storybook/test';");
        assert!(diags.is_empty(), "should allow @storybook/test import");
    }

    #[test]
    fn test_allows_other_imports() {
        let diags = lint("import React from 'react';");
        assert!(diags.is_empty(), "should allow other imports");
    }
}
