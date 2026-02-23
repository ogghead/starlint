//! Property-based tests for scope analysis.
//!
//! Verifies that `build_scope_data` never panics on any AST produced
//! by the parser, and that scope invariants hold.

use proptest::prelude::*;
use starlint_parser::{ParseOptions, parse};

use crate::build_scope_data;

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

/// Strategy producing JS/TS-like source fragments that exercise scope analysis.
fn js_scope_source() -> impl Strategy<Value = String> {
    prop_oneof![
        // Pure random
        ".*",
        // Scope-heavy fragments: declarations, closures, shadowing
        prop::collection::vec(
            prop_oneof![
                Just("const x = 1; ".to_owned()),
                Just("let y = 2; ".to_owned()),
                Just("var z = 3; ".to_owned()),
                Just("function f() { ".to_owned()),
                Just("function g(a, b) { return a + b; } ".to_owned()),
                Just("const h = () => { ".to_owned()),
                Just("class C { constructor() { this.x = 1; } } ".to_owned()),
                Just("if (true) { let x = 1; } ".to_owned()),
                Just("for (let i = 0; i < 10; i++) { } ".to_owned()),
                Just("for (const k of arr) { } ".to_owned()),
                Just("try { } catch (e) { } ".to_owned()),
                Just("{ let block = 1; } ".to_owned()),
                Just("import { a } from 'mod'; ".to_owned()),
                Just("export const b = 2; ".to_owned()),
                Just("export default function() { } ".to_owned()),
                Just("const [a, b] = [1, 2]; ".to_owned()),
                Just("const { x: y } = obj; ".to_owned()),
                Just("} ".to_owned()),
                Just("; ".to_owned()),
                Just("\n".to_owned()),
            ],
            1..30,
        )
        .prop_map(|tokens| tokens.join("")),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// The full parse → scope pipeline must never panic on any input.
    #[test]
    fn scope_analysis_never_panics(source in "\\PC*", options in any_parse_options()) {
        let result = parse(&source, options);
        // build_scope_data should handle any tree the parser produces
        let _scope = build_scope_data(&result.tree);
    }

    /// Scope analysis on JS-like fragments never panics.
    #[test]
    fn scope_analysis_js_fragments_never_panics(
        source in js_scope_source(),
        options in any_parse_options(),
    ) {
        let result = parse(&source, options);
        let _scope = build_scope_data(&result.tree);
    }

    /// Every symbol's scope must be queryable without panic.
    #[test]
    fn symbol_scopes_are_valid(source in js_scope_source()) {
        let options = ParseOptions { jsx: true, typescript: true, module: true };
        let result = parse(&source, options);
        let scope = build_scope_data(&result.tree);

        for sym_id in scope.symbol_ids() {
            // These queries should not panic
            let _flags = scope.symbol_flags(sym_id);
            let _span = scope.symbol_span(sym_id);
            let _name = scope.symbol_name(sym_id);
            let _scope_id = scope.symbol_scope_id(sym_id);
            let _refs = scope.get_resolved_references(sym_id);
        }
    }

    /// All reference spans should be within source bounds.
    #[test]
    fn reference_spans_in_bounds(source in js_scope_source()) {
        let options = ParseOptions { jsx: true, typescript: true, module: true };
        let result = parse(&source, options);
        let scope = build_scope_data(&result.tree);

        let source_len = u32::try_from(source.len()).unwrap_or(u32::MAX);

        for sym_id in scope.symbol_ids() {
            let span = scope.symbol_span(sym_id);
            prop_assert!(
                span.start <= span.end,
                "symbol span start > end: {:?}",
                span,
            );
            prop_assert!(
                span.end <= source_len,
                "symbol span end ({}) > source len ({})",
                span.end,
                source_len,
            );
        }
    }
}
