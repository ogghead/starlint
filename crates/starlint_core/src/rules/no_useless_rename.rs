//! Rule: `no-useless-rename`
//!
//! Disallow renaming imports and exports to the same name.
//! `import { foo as foo }` and `export { bar as bar }` are redundant.

use oxc_ast::AstKind;
use oxc_ast::ast::ModuleExportName;
use oxc_ast::ast_kind::AstType;
use oxc_span::GetSpan;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Edit, Fix, Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags redundant renames in import/export specifiers.
#[derive(Debug)]
pub struct NoUselessRename;

impl NativeRule for NoUselessRename {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "no-useless-rename".to_owned(),
            description: "Disallow renaming imports and exports to the same name".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ExportSpecifier, AstType::ImportSpecifier])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        match kind {
            AstKind::ImportSpecifier(spec) => {
                let imported_name = spec.imported.name();
                let local_name = &spec.local.name;

                // Skip shorthand: `import { foo }` — imported and local share the same span.
                if spec.imported.span().start == spec.local.span.start {
                    return;
                }

                if imported_name.as_str() == local_name.as_str() {
                    let local_str = local_name.as_str();
                    ctx.report(Diagnostic {
                        rule_name: "no-useless-rename".to_owned(),
                        message: format!("Import `{local_str}` is redundantly renamed to itself"),
                        span: Span::new(spec.span.start, spec.span.end),
                        severity: Severity::Warning,
                        help: Some(format!("Use `{local_str}` directly without `as`")),
                        fix: Some(Fix {
                            kind: FixKind::SafeFix,
                            message: format!("Remove redundant `as {local_str}`"),
                            edits: vec![Edit {
                                span: Span::new(spec.span.start, spec.span.end),
                                replacement: local_str.to_owned(),
                            }],
                            is_snippet: false,
                        }),
                        labels: vec![],
                    });
                }
            }
            AstKind::ExportSpecifier(spec) => {
                let local_name = spec.local.name();
                let exported_name = spec.exported.name();

                // Skip shorthand: `export { foo }` — local and exported share the same span.
                if spec.local.span().start == spec.exported.span().start {
                    return;
                }

                // `export { foo as default }` is meaningful, not useless.
                if matches!(&spec.exported, ModuleExportName::IdentifierName(id) if id.name == "default")
                {
                    return;
                }

                if local_name.as_str() == exported_name.as_str() {
                    let name = local_name.as_str();
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
                                replacement: name.to_owned(),
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::parser::parse_file;
    use crate::traversal::traverse_and_lint;

    fn lint(source: &str) -> Vec<Diagnostic> {
        let allocator = Allocator::default();
        let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) else {
            return vec![];
        };
        let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoUselessRename)];
        traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
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
