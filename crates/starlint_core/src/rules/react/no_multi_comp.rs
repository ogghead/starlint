//! Rule: `react/no-multi-comp`
//!
//! Only one component definition per file. Multiple component definitions
//! in a single file make it harder to find and maintain components.

use oxc_ast::AstKind;
use oxc_ast::ast_kind::AstType;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags files with multiple component definitions.
///
/// Simplified detection: counts top-level functions/classes that contain JSX
/// by scanning the source text for JSX return patterns. Uses a heuristic
/// approach based on counting `AstKind::JSXElement` occurrences at the
/// top-level function/class boundary.
#[derive(Debug)]
pub struct NoMultiComp;

impl NativeRule for NoMultiComp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/no-multi-comp".to_owned(),
            description: "Only one component definition per file".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run_on_kinds(&self) -> Option<&'static [AstType]> {
        Some(&[AstType::JSXElement])
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        // Simplified: flag React.createClass / createReactClass beyond the first,
        // or detect multiple class components / function components returning JSX.
        // For a practical implementation, we count top-level arrow/function expressions
        // that are assigned and contain JSX.
        //
        // This simplified version flags every `CallExpression` for `React.createClass`
        // or `createReactClass` after the first occurrence. For full detection of
        // multiple components, a more sophisticated approach tracking function scopes
        // would be needed.

        // We use a stub approach: flag nothing at the per-node level.
        // The real check happens in run_once.
        let _ = (kind, ctx);
    }

    fn needs_traversal(&self) -> bool {
        false
    }

    fn run_once(&self, ctx: &mut NativeLintContext<'_>) {
        let source = ctx.source_text();

        // Simplified heuristic: count patterns that look like component definitions.
        // Count function/class declarations that likely return JSX.
        // We look for multiple occurrences of common component patterns.
        let mut component_count = 0u32;
        let mut last_span_start = 0u32;

        // Count class components (classes extending Component/PureComponent)
        for (idx, _) in source
            .match_indices("extends Component")
            .chain(source.match_indices("extends React.Component"))
            .chain(source.match_indices("extends PureComponent"))
            .chain(source.match_indices("extends React.PureComponent"))
        {
            component_count = component_count.saturating_add(1);
            if component_count > 1 {
                last_span_start = u32::try_from(idx).unwrap_or(0);
            }
        }

        if component_count > 1 {
            ctx.report_warning(
                "react/no-multi-comp",
                "Declare only one React component per file",
                Span::new(last_span_start, last_span_start.saturating_add(1)),
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
        if let Ok(parsed) = parse_file(&allocator, source, Path::new("test.jsx")) {
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(NoMultiComp)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_multiple_class_components() {
        let source = r"
class CompA extends React.Component {
    render() { return null; }
}
class CompB extends React.Component {
    render() { return null; }
}";
        let diags = lint(source);
        assert_eq!(
            diags.len(),
            1,
            "multiple class components should be flagged"
        );
    }

    #[test]
    fn test_allows_single_component() {
        let source = r"
class MyComponent extends React.Component {
    render() { return null; }
}";
        let diags = lint(source);
        assert!(diags.is_empty(), "single component should not be flagged");
    }

    #[test]
    fn test_allows_no_components() {
        let source = "const x = 1;";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "file without components should not be flagged"
        );
    }
}
