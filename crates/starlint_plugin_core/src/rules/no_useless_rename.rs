//! Rule: `no-useless-rename`
//!
//! Disallow renaming imports and exports to the same name.
//! `import { foo as foo }` and `export { bar as bar }` are redundant.

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use starlint_ast::node::AstNode;
use starlint_ast::node_type::AstNodeType;
use starlint_ast::types::NodeId;
use starlint_rule_framework::{LintContext, LintRule};

/// Flags redundant renames in import/export specifiers.
#[derive(Debug)]
pub struct NoUselessRename;

impl LintRule for NoUselessRename {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-rename".to_owned(),
            description: "Disallow renaming imports and exports to the same name".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_types(&self) -> Option<&'static [AstNodeType]> {
        Some(&[AstNodeType::ExportSpecifier, AstNodeType::ImportSpecifier])
    }

    fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<'_>) {
        match node {
            AstNode::ImportSpecifier(spec) => {
                let imported_name = &spec.imported;
                let local_name = &spec.local;

                // Skip shorthand: `import { foo }` — no `as` in source text
                let spec_text = source_slice(ctx.source_text(), spec.span.start, spec.span.end);
                if !spec_text.contains(" as ") {
                    return;
                }

                if imported_name == local_name {
                    ctx.report(Diagnostic {
                        rule_name: "no-useless-rename".to_owned(),
                        message: format!("Import `{local_name}` is redundantly renamed to itself"),
                        span: Span::new(spec.span.start, spec.span.end),
                        severity: Severity::Warning,
                        help: Some(format!("Use `{local_name}` directly without `as`")),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Remove redundant `as {local_name}`"),
                            edits: vec![Edit {
                                span: Span::new(spec.span.start, spec.span.end),
                                replacement: local_name.clone(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            AstNode::ExportSpecifier(spec) => {
                let local_name = &spec.local;
                let exported_name = &spec.exported;

                // Skip shorthand: `export { foo }` — no `as` in source text
                let spec_text = source_slice(ctx.source_text(), spec.span.start, spec.span.end);
                if !spec_text.contains(" as ") {
                    return;
                }

                // `export { foo as default }` is meaningful, not useless.
                if exported_name == "default" {
                    return;
                }

                if local_name == exported_name {
                    let name = local_name;
                    ctx.report(Diagnostic {
                        rule_name: "no-useless-rename".to_owned(),
                        message: format!("Export `{name}` is redundantly renamed to itself"),
                        span: Span::new(spec.span.start, spec.span.end),
                        severity: Severity::Warning,
                        help: Some(format!("Use `{name}` directly without `as`")),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Remove redundant `as {name}`"),
                            edits: vec![Edit {
                                span: Span::new(spec.span.start, spec.span.end),
                                replacement: name.clone(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

/// Extract a slice of source text by u32 offsets.
fn source_slice(source: &str, start: u32, end: u32) -> &str {
    let s = usize::try_from(start).unwrap_or(0);
    let e = usize::try_from(end).unwrap_or(0).min(source.len());
    source.get(s..e).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlint_rule_framework::lint_source;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(NoUselessRename)];
        lint_source(source, "test.js", &rules)
    }

    #[test]
    fn test_flags_useless_import_rename() {
        let diags = lint("import { foo as foo } from 'bar';");
        assert_eq!(diags.len(), 1, "should flag useless import rename");
        let fix = diags.first().and_then(|d| d.fix.as_ref());
        assert_eq!(
            fix.and_then(|f| f.edits.first().map(|e| e.replacement.as_str())),
            Some("foo"),
            "fix should replace with just the name"
        );
    }

    #[test]
    fn test_flags_useless_export_rename() {
        let diags = lint("const bar = 1; export { bar as bar };");
        assert_eq!(diags.len(), 1, "should flag useless export rename");
    }

    #[test]
    fn test_ignores_different_import_names() {
        let diags = lint("import { foo as bar } from 'baz';");
        assert!(diags.is_empty(), "different names should not be flagged");
    }

    #[test]
    fn test_ignores_different_export_names() {
        let diags = lint("const foo = 1; export { foo as bar };");
        assert!(diags.is_empty(), "different names should not be flagged");
    }

    #[test]
    fn test_ignores_export_as_default() {
        let diags = lint("const default_ = 1; export { default_ as default };");
        assert!(
            diags.is_empty(),
            "export as default should not be flagged even if names match"
        );
    }

    #[test]
    fn test_ignores_normal_import() {
        let diags = lint("import { foo } from 'bar';");
        assert!(diags.is_empty(), "normal import should not be flagged");
    }
}
