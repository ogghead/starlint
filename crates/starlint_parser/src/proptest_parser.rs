//! Property-based tests for the parser.
//!
//! Verifies that the parser never panics on arbitrary input and that
//! structural invariants hold for every produced AST.

use proptest::prelude::*;
use starlint_ast::types::NodeId;

use crate::{ParseOptions, parse};

/// Strategy that produces all 8 combinations of `ParseOptions`.
fn any_parse_options() -> impl Strategy<Value = ParseOptions> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(jsx, typescript, module)| {
        ParseOptions {
            jsx,
            typescript,
            module,
        }
    })
}

/// Strategy producing JS/TS-like source fragments for deeper parser coverage.
fn js_like_source() -> impl Strategy<Value = String> {
    prop_oneof![
        // Pure random strings
        ".*",
        // Token-level fragments likely to exercise parser branches
        prop::collection::vec(
            prop_oneof![
                Just("const ".to_owned()),
                Just("let ".to_owned()),
                Just("var ".to_owned()),
                Just("function ".to_owned()),
                Just("class ".to_owned()),
                Just("if (".to_owned()),
                Just(") { ".to_owned()),
                Just("} ".to_owned()),
                Just("=> ".to_owned()),
                Just("import ".to_owned()),
                Just("export ".to_owned()),
                Just("from ".to_owned()),
                Just("return ".to_owned()),
                Just("throw ".to_owned()),
                Just("try { ".to_owned()),
                Just("catch (e) { ".to_owned()),
                Just("finally { ".to_owned()),
                Just("for (".to_owned()),
                Just("while (".to_owned()),
                Just("switch (".to_owned()),
                Just("case ".to_owned()),
                Just("break; ".to_owned()),
                Just("continue; ".to_owned()),
                Just("debugger; ".to_owned()),
                Just("new ".to_owned()),
                Just("typeof ".to_owned()),
                Just("void ".to_owned()),
                Just("delete ".to_owned()),
                Just("await ".to_owned()),
                Just("async ".to_owned()),
                Just("yield ".to_owned()),
                Just("<div>".to_owned()),
                Just("</div>".to_owned()),
                Just("</>".to_owned()),
                Just("x ".to_owned()),
                Just("42 ".to_owned()),
                Just("\"str\" ".to_owned()),
                Just("'str' ".to_owned()),
                Just("`tmpl` ".to_owned()),
                Just("true ".to_owned()),
                Just("false ".to_owned()),
                Just("null ".to_owned()),
                Just("undefined ".to_owned()),
                Just("= ".to_owned()),
                Just("== ".to_owned()),
                Just("=== ".to_owned()),
                Just("!= ".to_owned()),
                Just("!== ".to_owned()),
                Just("+ ".to_owned()),
                Just("- ".to_owned()),
                Just("* ".to_owned()),
                Just("/ ".to_owned()),
                Just("; ".to_owned()),
                Just(", ".to_owned()),
                Just(": ".to_owned()),
                Just("? ".to_owned()),
                Just("?. ".to_owned()),
                Just("?? ".to_owned()),
                Just("... ".to_owned()),
                Just("( ".to_owned()),
                Just(") ".to_owned()),
                Just("[ ".to_owned()),
                Just("] ".to_owned()),
                Just("{ ".to_owned()),
                Just("} ".to_owned()),
                Just(". ".to_owned()),
                Just("\n".to_owned()),
                // TypeScript tokens
                Just("interface ".to_owned()),
                Just("type ".to_owned()),
                Just("enum ".to_owned()),
                Just("as ".to_owned()),
                Just("<T>".to_owned()),
                Just(": number ".to_owned()),
                Just(": string ".to_owned()),
            ],
            1..50,
        )
        .prop_map(|tokens| tokens.join("")),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// The parser must never panic on any input string with any options.
    #[test]
    fn parse_never_panics(source in "\\PC*", options in any_parse_options()) {
        // If this completes without panic, the property holds.
        let _result = parse(&source, options);
    }

    /// JS-like token sequences exercise more parser branches without panicking.
    #[test]
    fn parse_js_fragments_never_panics(source in js_like_source(), options in any_parse_options()) {
        let _result = parse(&source, options);
    }

    /// Every span in the AST must have start <= end.
    #[test]
    fn spans_are_well_formed(source in js_like_source(), options in any_parse_options()) {
        let result = parse(&source, options);
        #[allow(clippy::cast_possible_truncation)]
        let source_len = source.len() as u32;

        for (id, _node) in result.tree.iter() {
            if let Some(span) = result.tree.span(id) {
                prop_assert!(
                    span.start <= span.end,
                    "span start ({}) > end ({}) for node {:?}",
                    span.start,
                    span.end,
                    id,
                );
                prop_assert!(
                    span.end <= source_len,
                    "span end ({}) > source len ({}) for node {:?}",
                    span.end,
                    source_len,
                    id,
                );
            }
        }
    }

    /// Parent references must point to existing nodes (no dangling refs).
    #[test]
    fn parent_refs_are_valid(source in js_like_source(), options in any_parse_options()) {
        let result = parse(&source, options);
        let tree_len = result.tree.len();

        for (id, _node) in result.tree.iter() {
            if id == NodeId::ROOT {
                // Root has no parent
                prop_assert!(
                    result.tree.parent(id).is_none(),
                    "root node should have no parent"
                );
            } else if let Some(parent_id) = result.tree.parent(id) {
                prop_assert!(
                    (parent_id.index()) < tree_len,
                    "parent id {:?} out of bounds (tree len {}) for node {:?}",
                    parent_id,
                    tree_len,
                    id,
                );
            }
        }
    }

    /// An empty source should produce a single Program node with no errors.
    #[test]
    fn empty_source_produces_program(options in any_parse_options()) {
        let result = parse("", options);
        prop_assert!(result.errors.is_empty(), "empty source should have no errors");
        prop_assert_eq!(result.tree.len(), 1, "empty source should have just Program");
    }
}
