//! Rule: `storybook/use-storybook-testing-library`
//!
//! Do not import testing-library directly in stories, use `@storybook/test`.
//! Matches `ImportDeclaration` for `@testing-library/`.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Rule name constant.
const RULE_NAME: &str = "storybook/use-storybook-testing-library";

/// Do not import testing-library directly in stories.
#[derive(Debug)]
pub struct UseStorybookTestingLibrary;

impl NativeRule for UseStorybookTestingLibrary {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: RULE_NAME.to_owned(),
            description:
                "Do not import testing-library directly in stories — use `@storybook/test`"
                    .to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::SuggestionFix,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let file_name = ctx.file_path().to_string_lossy();
        if !file_name.contains(".stories.") && !file_name.contains(".story.") {
            return;
        }

        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let source_value = import.source.value.as_str();
        if source_value.starts_with("@testing-library/") {
            ctx.report(Diagnostic {
                rule_name: RULE_NAME.to_owned(),
                message: "Import from `@storybook/test` instead of `@testing-library/` directly"
                    .to_owned(),
                span: Span::new(import.span.start, import.span.end),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("Button.stories.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(UseStorybookTestingLibrary)];
            traverse_and_lint(
                &parsed.program,
                &rules,
                source,
                Path::new("Button.stories.ts"),
            )
        } else {
            vec![]
        }
    }

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
