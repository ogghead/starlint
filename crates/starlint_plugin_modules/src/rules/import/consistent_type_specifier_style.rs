//! Rule: `import/consistent-type-specifier-style`
//!
//! Enforce consistent usage of type specifier style (inline vs top-level).
//! For example, prefer `import type { Foo } from 'bar'` over
//! `import { type Foo } from 'bar'` (or vice versa).

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags inconsistent usage of type specifiers in import declarations.
#[derive(Debug)]
pub struct ConsistentTypeSpecifierStyle;

impl LintRule for ConsistentTypeSpecifierStyle {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/consistent-type-specifier-style".to_owned(),
            description: "Enforce consistent usage of type specifier style (inline vs top-level)"
                .to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ImportDeclaration])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        let AstNode::ImportDeclaration(import) = node else {
            return;
        };

        // Skip type-only imports — they are already consistent
        if import.import_kind_is_type {
            return;
        }

        // Check if any individual specifiers use inline `type` keyword
        if import.specifiers.is_empty() {
            return;
        }
        let has_inline_type = import.specifiers.iter().any(|&spec_id| {
            matches!(
                ctx.node(spec_id),
                Some(AstNode::ImportSpecifier(s)) if s.is_type
            )
        });

        if !has_inline_type {
            return;
        }

        // All specifiers are type imports — prefer top-level `import type`
        let all_type = import.specifiers.iter().all(|&spec_id| {
            matches!(
                ctx.node(spec_id),
                Some(AstNode::ImportSpecifier(s)) if s.is_type
            )
        });

        if all_type {
            // Build the replacement: replace `import {` with `import type {`
            // and strip inline `type` keywords from each specifier.
            let import_start = usize::try_from(import.span.start).unwrap_or(0);
            let import_end = usize::try_from(import.span.end).unwrap_or(0);
            let import_text = ctx
                .source_text()
                .get(import_start..import_end)
                .unwrap_or("");

            // Replace `import ` with `import type ` at the start, then remove all
            // inline `type ` prefixes inside the braces.
            let replacement = if let Some(rest) = import_text.strip_prefix("import ") {
                let cleaned = rest.replace("type ", "");
                format!("import type {cleaned}")
            } else {
                import_text.to_owned()
            };

            ctx.report(Diagnostic {
                rule_name: "import/consistent-type-specifier-style".to_owned(),
                message: "Prefer top-level `import type` when all specifiers are type imports"
                    .to_owned(),
                span: Span::new(import.span.start, import.span.end),
                severity: Severity::Warning,
                help: Some(
                    "Use `import type { ... }` instead of inline `type` specifiers".to_owned(),
                ),
                fix: Some(Fix {
                    kind: FixKind::SafeFix,
                    message: "Convert to top-level `import type`".to_owned(),
                    edits: vec![Edit {
                        span: Span::new(import.span.start, import.span.end),
                        replacement,
                    }],
                    is_snippet: false,
                }),
                labels: vec![],
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    starlint_rule_framework::lint_rule_test!(ConsistentTypeSpecifierStyle, "test.ts");

    #[test]
    fn test_flags_all_inline_type_specifiers() {
        let diags = lint(r"import { type Foo, type Bar } from 'mod';");
        assert_eq!(
            diags.len(),
            1,
            "all inline type specifiers should prefer top-level import type"
        );
    }

    #[test]
    fn test_allows_top_level_type_import() {
        let diags = lint(r"import type { Foo, Bar } from 'mod';");
        assert!(diags.is_empty(), "top-level type import should be allowed");
    }

    #[test]
    fn test_allows_mixed_inline_and_value() {
        let diags = lint(r"import { type Foo, bar } from 'mod';");
        assert!(
            diags.is_empty(),
            "mixed inline type and value specifiers should be allowed"
        );
    }
}
