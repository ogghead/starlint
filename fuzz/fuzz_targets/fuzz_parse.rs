#![no_main]

use libfuzzer_sys::fuzz_target;
use starlint_parser::{ParseOptions, parse};

// Fuzz the parser with arbitrary byte sequences across all option combos.
//
// The parser accepts `&str`, so we convert the fuzzer's `&[u8]` to a string
// (skipping invalid UTF-8). Each input is parsed with all 8 combinations of
// `ParseOptions` to maximize branch coverage.
fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    // All 8 combinations of (jsx, typescript, module)
    for jsx in [false, true] {
        for typescript in [false, true] {
            for module in [false, true] {
                let options = ParseOptions {
                    jsx,
                    typescript,
                    module,
                };
                let _result = parse(source, options);
            }
        }
    }
});
