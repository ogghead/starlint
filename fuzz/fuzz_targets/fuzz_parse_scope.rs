#![no_main]

use libfuzzer_sys::fuzz_target;
use starlint_parser::{ParseOptions, parse};
use starlint_scope::build_scope_data;

// Fuzz the full parse → scope analysis pipeline.
//
// Parses arbitrary source text, then runs scope analysis on the resulting
// AST. This catches panics in both the parser and the scope builder.
fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    // Use the most permissive options to exercise the most code paths
    let options = ParseOptions {
        jsx: true,
        typescript: true,
        module: true,
    };

    let result = parse(source, options);
    let _scope = build_scope_data(&result.tree);
});
