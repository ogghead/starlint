//! Rule: `max-classes-per-file` (eslint)
//!
//! Flag files with too many class declarations. Having multiple classes
//! in one file often indicates that the file should be split.

use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Diagnostic, Severity, Span};
use starlint_plugin_sdk::rule::{Category, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Default maximum number of classes per file.
const DEFAULT_MAX: u32 = 1;

/// Flags files containing more than the allowed number of class declarations.
#[derive(Debug)]
pub struct MaxClassesPerFile {
    /// Maximum number of class declarations allowed per file.
    max: u32,
}

impl MaxClassesPerFile {
    /// Create a new `MaxClassesPerFile` rule with the default threshold.
    #[must_use]
    pub const fn new() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

impl Default for MaxClassesPerFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Count class declarations in a list of statements (non-recursive, top-level only).
fn count_classes(body: &[Statement<'_>]) -> u32 {
    let mut count: u32 = 0;
    for stmt in body {
        if matches!(stmt, Statement::ClassDeclaration(_)) {
            count = count.saturating_add(1);
        }
        // Also check export default class
        if let Statement::ExportDefaultDeclaration(export) = stmt {
            if matches!(
                &export.declaration,
                oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(_)
                    | oxc_ast::ast::ExportDefaultDeclarationKind::ClassExpression(_)
            ) {
                count = count.saturating_add(1);
            }
        }
        // Check exported class declarations
        if let Statement::ExportNamedDeclaration(export) = stmt {
            if let Some(oxc_ast::ast::Declaration::ClassDeclaration(_)) = &export.declaration {
                count = count.saturating_add(1);
            }
        }
    }
    count
}

impl NativeRule for MaxClassesPerFile {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "max-classes-per-file".to_owned(),
            description: "Enforce a maximum number of classes per file".to_owned(),
            category: Category::Style,
            default_severity: Severity::Warning,
        }
    }

    fn needs_traversal(&self) -> bool {
        // We use run() with AstKind::Program to access the program body
        true
    }

    fn configure(&mut self, config: &serde_json::Value) -> Result<(), String> {
        if let Some(n) = config.get("max").and_then(serde_json::Value::as_u64) {
            self.max = u32::try_from(n).unwrap_or(DEFAULT_MAX);
        }
        Ok(())
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::Program])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::Program(program) = kind else {
            return;
        };

        let class_count = count_classes(&program.body);

        if class_count > self.max {
            let source_len = u32::try_from(ctx.source_text().len()).unwrap_or(0);
            ctx.report(Diagnostic {
                rule_name: "max-classes-per-file".to_owned(),
                message: format!(
                    "File has too many classes ({class_count}). Maximum allowed is {}",
                    self.max
                ),
                span: Span::new(0, source_len),
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

    fn lint_with_max(source: &str, max: u32) -> Vec<starlint_plugin_sdk::diagnostic::Diagnostic> {
        let allocator = Allocator::default();
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.js")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(MaxClassesPerFile { max })];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.js"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_allows_one_class() {
        let diags = lint_with_max("class Foo {}", 1);
        assert!(
            diags.is_empty(),
            "single class should not be flagged with max 1"
        );
    }

    #[test]
    fn test_flags_two_classes() {
        let diags = lint_with_max("class Foo {}\nclass Bar {}", 1);
        assert_eq!(diags.len(), 1, "two classes should be flagged with max 1");
    }

    #[test]
    fn test_allows_two_classes_with_max_two() {
        let diags = lint_with_max("class Foo {}\nclass Bar {}", 2);
        assert!(
            diags.is_empty(),
            "two classes should not be flagged with max 2"
        );
    }

    #[test]
    fn test_no_classes() {
        let diags = lint_with_max("const x = 1;", 1);
        assert!(diags.is_empty(), "no classes should not be flagged");
    }
}
