//! Rule: `react/state-in-constructor`
//!
//! State should be initialized in the constructor. Using class property syntax
//! for state initialization (`state = {...}`) is less explicit and can be
//! confusing when mixed with constructor-based initialization.

use oxc_ast::AstKind;
use oxc_ast::ast::PropertyKey;

use starlint_plugin_sdk::diagnostic::{Severity, Span};
use starlint_plugin_sdk::rule::{Category, FixKind, RuleMeta};

use crate::rule::{NativeLintContext, NativeRule};

/// Flags `state` as a class field property definition.
#[derive(Debug)]
pub struct StateInConstructor;

impl NativeRule for StateInConstructor {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: "react/state-in-constructor".to_owned(),
            description: "State should be initialized in the constructor".to_owned(),
            category: Category::Suggestion,
            default_severity: Severity::Warning,
            fix_kind: FixKind::None,
        }
    }

    fn run(&self, kind: &AstKind<'_>, ctx: &mut NativeLintContext<'_>) {
        let AstKind::PropertyDefinition(prop) = kind else {
            return;
        };

        // Only flag non-static, non-computed property definitions named "state"
        if prop.r#static || prop.computed {
            return;
        }

        let is_state = match &prop.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.as_str() == "state",
            _ => false,
        };

        if is_state {
            ctx.report_warning(
                "react/state-in-constructor",
                "State initialization should be in a constructor",
                Span::new(prop.span.start, prop.span.end),
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
            let rules: Vec<Box<dyn NativeRule>> = vec![Box::new(StateInConstructor)];
            traverse_and_lint(&parsed.program, &rules, source, Path::new("test.jsx"))
        } else {
            vec![]
        }
    }

    #[test]
    fn test_flags_state_class_property() {
        let source = r"
class MyComponent extends React.Component {
    state = { count: 0 };
    render() { return null; }
}";
        let diags = lint(source);
        assert_eq!(diags.len(), 1, "state as class property should be flagged");
    }

    #[test]
    fn test_allows_state_in_constructor() {
        let source = r"
class MyComponent extends React.Component {
    constructor(props) {
        super(props);
        this.state = { count: 0 };
    }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "state in constructor should not be flagged"
        );
    }

    #[test]
    fn test_allows_other_class_properties() {
        let source = r"
class MyComponent extends React.Component {
    value = 42;
    render() { return null; }
}";
        let diags = lint(source);
        assert!(
            diags.is_empty(),
            "other class properties should not be flagged"
        );
    }
}
