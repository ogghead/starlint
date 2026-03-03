//! Benchmarks for WASM plugin overhead.
//!
//! Measures the per-file cost of the WASM plugin pipeline to establish
//! baselines for the plugin extraction plan (Phase 0c).
//!
//! Groups:
//! - **load**: One-time plugin load + compile cost
//! - **per_file**: Per-file lint cost (store creation, node collection, WASM call)
//! - **file_pattern_skip**: Cost of skipping a file via glob pattern match

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::missing_docs_in_private_items,
    clippy::print_stdout,
    clippy::use_debug
)]

use std::path::Path;

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxc_allocator::Allocator;

use starlint_core::parser::parse_file;
use starlint_core::plugin::PluginHost;
use starlint_wasm_host::runtime::{ResourceLimits, WasmPluginHost};

/// Path to the pre-built example plugin (no file patterns — matches all files).
const EXAMPLE_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/example-plugin.wasm"
);

/// Path to the pre-built JSX example plugin (file patterns: *.jsx, *.tsx).
const JSX_EXAMPLE_PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/plugins/jsx-example-plugin.wasm"
);

/// Small JS source with a single debugger statement.
const SMALL_JS: &str = "debugger;\n";

/// Medium JS source with imports, functions, and a debugger.
const MEDIUM_JS: &str = r#"
import { foo } from 'bar';
import * as baz from 'qux';

function helper(a, b) {
    return a + b;
}

const result = helper(1, 2);
console.log(result);
debugger;

export default helper;
"#;

/// Large JS source — many statements but few matching nodes.
fn large_js() -> String {
    let mut source = String::from("import { x } from 'y';\n");
    for i in 0..500 {
        source.push_str(&format!("const v{i} = {i};\n"));
    }
    source.push_str("debugger;\n");
    source.push_str("export default v0;\n");
    source
}

/// JSX source for file-pattern benchmarks.
const JSX_SOURCE: &str = r#"
import React from 'react';

function App() {
    return (
        <div>
            <img src="photo.jpg" />
            <a href="https://example.com" target="_blank">Link</a>
        </div>
    );
}

export default App;
"#;

// ---------- Benchmark groups ----------

fn bench_plugin_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_load");

    group.bench_function("example_plugin", |b| {
        b.iter(|| {
            let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("create host");
            host.load_plugin(Path::new(EXAMPLE_PLUGIN), "")
                .expect("load plugin");
            black_box(host);
        });
    });

    group.bench_function("jsx_plugin", |b| {
        b.iter(|| {
            let mut host = WasmPluginHost::new(ResourceLimits::default()).expect("create host");
            host.load_plugin(Path::new(JSX_EXAMPLE_PLUGIN), "")
                .expect("load plugin");
            black_box(host);
        });
    });

    group.finish();
}

fn bench_per_file_lint(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_file_lint");

    // Pre-load the host once (amortized).
    let host = {
        let mut h = WasmPluginHost::new(ResourceLimits::default()).expect("create host");
        h.load_plugin(Path::new(EXAMPLE_PLUGIN), "")
            .expect("load plugin");
        h
    };

    let large = large_js();
    let cases: &[(&str, &str)] = &[
        ("small_1_node", SMALL_JS),
        ("medium_mixed", MEDIUM_JS),
        ("large_500_stmts", &large),
    ];

    for (label, source) in cases {
        group.bench_with_input(BenchmarkId::new("example", label), source, |b, src| {
            let allocator = Allocator::default();
            let path = Path::new("bench.js");
            let parsed = parse_file(&allocator, src, path).expect("parse");
            b.iter(|| {
                let diags = host.lint_file(black_box(path), black_box(src), &parsed.program);
                black_box(diags);
            });
        });
    }

    group.finish();
}

fn bench_file_pattern_skip(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_pattern_skip");

    // JSX plugin only matches *.jsx, *.tsx files.
    let host = {
        let mut h = WasmPluginHost::new(ResourceLimits::default()).expect("create host");
        h.load_plugin(Path::new(JSX_EXAMPLE_PLUGIN), "")
            .expect("load plugin");
        h
    };

    // Lint a .js file — should be skipped by glob pattern before any work.
    group.bench_function("skip_non_matching_js", |b| {
        let allocator = Allocator::default();
        let path = Path::new("bench.js");
        let parsed = parse_file(&allocator, MEDIUM_JS, path).expect("parse");
        b.iter(|| {
            let diags = host.lint_file(black_box(path), black_box(MEDIUM_JS), &parsed.program);
            black_box(diags);
        });
    });

    // Lint a .jsx file — should proceed to full WASM call.
    group.bench_function("match_jsx_file", |b| {
        let allocator = Allocator::default();
        let path = Path::new("bench.jsx");
        let parsed = parse_file(&allocator, JSX_SOURCE, path).expect("parse");
        b.iter(|| {
            let diags = host.lint_file(black_box(path), black_box(JSX_SOURCE), &parsed.program);
            black_box(diags);
        });
    });

    group.finish();
}

fn bench_multi_plugin(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_plugin");

    // Both plugins loaded — measures overhead of iterating + skipping.
    let host = {
        let mut h = WasmPluginHost::new(ResourceLimits::default()).expect("create host");
        h.load_plugin(Path::new(EXAMPLE_PLUGIN), "")
            .expect("load example");
        h.load_plugin(Path::new(JSX_EXAMPLE_PLUGIN), "")
            .expect("load jsx");
        h
    };

    // .js file: example plugin runs, JSX plugin skips.
    group.bench_function("js_file_2_plugins", |b| {
        let allocator = Allocator::default();
        let path = Path::new("bench.js");
        let parsed = parse_file(&allocator, MEDIUM_JS, path).expect("parse");
        b.iter(|| {
            let diags = host.lint_file(black_box(path), black_box(MEDIUM_JS), &parsed.program);
            black_box(diags);
        });
    });

    // .jsx file: both plugins run.
    group.bench_function("jsx_file_2_plugins", |b| {
        let allocator = Allocator::default();
        let path = Path::new("bench.jsx");
        let parsed = parse_file(&allocator, JSX_SOURCE, path).expect("parse");
        b.iter(|| {
            let diags = host.lint_file(black_box(path), black_box(JSX_SOURCE), &parsed.program);
            black_box(diags);
        });
    });

    group.finish();
}

fn bench_no_matching_nodes(c: &mut Criterion) {
    let mut group = c.benchmark_group("no_matching_nodes");

    let host = {
        let mut h = WasmPluginHost::new(ResourceLimits::default()).expect("create host");
        h.load_plugin(Path::new(EXAMPLE_PLUGIN), "")
            .expect("load plugin");
        h
    };

    // Source with no debugger statements or imports — node collection finds nothing.
    let clean_source = r#"
const a = 1;
const b = 2;
function add(x, y) { return x + y; }
const c = add(a, b);
"#;

    group.bench_function("clean_source", |b| {
        let allocator = Allocator::default();
        let path = Path::new("bench.js");
        let parsed = parse_file(&allocator, clean_source, path).expect("parse");
        b.iter(|| {
            let diags = host.lint_file(black_box(path), black_box(clean_source), &parsed.program);
            black_box(diags);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_plugin_load,
    bench_per_file_lint,
    bench_file_pattern_skip,
    bench_multi_plugin,
    bench_no_matching_nodes,
);
criterion_main!(benches);
