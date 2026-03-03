//! Rule: `import/no-webpack-loader-syntax`
//!
//! Forbid webpack loader syntax in imports. Webpack loader syntax (e.g.
//! `import 'style-loader!css-loader!./file.css'`) couples code to webpack
//! and should be configured in webpack config instead.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags import sources containing webpack loader syntax (`!`).
#[derive(Debug)]
pub struct NoWebpackLoaderSyntax;

impl NativeRule for NoWebpackLoaderSyntax {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "import/no-webpack-loader-syntax".to_owned(),
            description: "Forbid webpack loader syntax in imports".to_owned(),
            category: Category::Correctness,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::ImportDeclaration])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::ImportDeclaration(import) = kind else {
            return;
        };

        let source_value = import.source.value.as_str();
        if source_value.contains('!') {
            ctx.report_warning(
                "import/no-webpack-loader-syntax",
                "Unexpected use of webpack loader syntax in import source",
                Span::new(import.span.start, import.span.end),
            );
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.ts")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoWebpackLoaderSyntax)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.ts"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_loader_syntax() {
        let diags = lint(r#"import foo from "style-loader!css-loader!./styles.css";"#);
        assert_eq!(diags.len(), 1, "webpack loader syntax should be flagged");
    }

    #[test]
    fn test_allows_normal_import() {
        let diags = lint(r#"import foo from "./styles.css";"#);
        assert!(diags.is_empty(), "normal import should not be flagged");
    }

    #[test]
    fn test_flags_single_loader() {
        let diags = lint(r#"import styles from "css-loader!./styles.css";"#);
        assert_eq!(diags.len(), 1, "single loader syntax should be flagged");
    }
}
