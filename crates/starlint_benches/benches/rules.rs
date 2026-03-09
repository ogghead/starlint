//! Per-rule benchmarks for all starlint lint rules.
//!
//! Each plugin gets a benchmark group. Within each group, every rule is
//! benchmarked individually plus a `_bundle` benchmark that runs all rules
//! in that plugin together.
//!
//! Filter with: `cargo bench -p starlint-benches -- <pattern>`

#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]
#![allow(clippy::unwrap_used)]
#![allow(missing_docs)]

use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use starlint_benches::ParsedFixture;
use starlint_parser::ParseOptions;
use starlint_rule_framework::{LintRule, LintRulePlugin, Plugin};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Benchmark each rule individually and as a bundle within a criterion group.
fn bench_plugin(
    c: &mut Criterion,
    group_name: &str,
    rules_factory: fn() -> Vec<Box<dyn LintRule>>,
    fixture: &ParsedFixture,
) {
    let mut group = c.benchmark_group(group_name);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);

    // Per-rule benchmarks: consume the vec, moving each rule into its own plugin.
    for rule in rules_factory() {
        let name = rule.meta().name.clone();
        let plugin = LintRulePlugin::new(vec![rule]);
        group.bench_function(&name, |b| {
            let ctx = fixture.file_context();
            b.iter(|| black_box(plugin.lint_file(&ctx)));
        });
    }

    // Bundle benchmark: all rules in the plugin running together.
    let bundle = LintRulePlugin::new(rules_factory());
    group.bench_function("_bundle", |b| {
        let ctx = fixture.file_context();
        b.iter(|| black_box(bundle.lint_file(&ctx)));
    });

    group.finish();
}

// ── Parse benchmarks ─────────────────────────────────────────────────────────

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let fixtures: &[(&str, &str, &str)] = &[
        ("js", starlint_benches::JS_FIXTURE, "bench.js"),
        ("tsx", starlint_benches::TSX_FIXTURE, "bench.tsx"),
        ("ts", starlint_benches::TS_FIXTURE, "bench.ts"),
        ("test_js", starlint_benches::TEST_FIXTURE, "bench.test.js"),
    ];

    for &(label, source, path) in fixtures {
        group.bench_function(label, |b| {
            let opts = ParseOptions::from_path(std::path::Path::new(path));
            b.iter(|| black_box(starlint_parser::parse(source, opts)));
        });
    }

    // Scope analysis benchmark (parse + scope).
    group.bench_function("js_with_scope", |b| {
        let opts = ParseOptions::from_path(std::path::Path::new("bench.js"));
        b.iter(|| {
            let result = starlint_parser::parse(starlint_benches::JS_FIXTURE, opts);
            black_box(starlint_scope::build_scope_data(&result.tree));
        });
    });

    group.finish();
}

// ── Plugin benchmarks ────────────────────────────────────────────────────────

fn bench_core(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::JS_FIXTURE, "bench.js");
    bench_plugin(c, "core", starlint_plugin_core::all_rules, &fixture);
}

fn bench_react(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::TSX_FIXTURE, "bench.tsx");
    bench_plugin(c, "react", starlint_plugin_react::all_rules, &fixture);
}

fn bench_typescript(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::TS_FIXTURE, "bench.ts");
    bench_plugin(
        c,
        "typescript",
        starlint_plugin_typescript::all_rules,
        &fixture,
    );
}

fn bench_testing(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::TEST_FIXTURE, "bench.test.js");
    bench_plugin(c, "testing", starlint_plugin_testing::all_rules, &fixture);
}

fn bench_modules(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::MODULES_FIXTURE, "bench.js");
    bench_plugin(c, "modules", starlint_plugin_modules::all_rules, &fixture);
}

fn bench_nextjs(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::NEXTJS_FIXTURE, "pages/bench.tsx");
    bench_plugin(c, "nextjs", starlint_plugin_nextjs::all_rules, &fixture);
}

fn bench_vue(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::VUE_FIXTURE, "bench.js");
    bench_plugin(c, "vue", starlint_plugin_vue::all_rules, &fixture);
}

fn bench_jsdoc(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::JSDOC_FIXTURE, "bench.js");
    bench_plugin(c, "jsdoc", starlint_plugin_jsdoc::all_rules, &fixture);
}

fn bench_storybook(c: &mut Criterion) {
    let fixture = ParsedFixture::new(starlint_benches::STORYBOOK_FIXTURE, "bench.stories.tsx");
    bench_plugin(
        c,
        "storybook",
        starlint_plugin_storybook::all_rules,
        &fixture,
    );
}

// ── Full-stack benchmark ─────────────────────────────────────────────────────

fn bench_full_stack(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_stack");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(20);

    let fixture = ParsedFixture::new(starlint_benches::JS_FIXTURE, "bench.js");

    // All 709 rules on a single JS file.
    let all_rules: Vec<Box<dyn LintRule>> = [
        starlint_plugin_core::all_rules(),
        starlint_plugin_react::all_rules(),
        starlint_plugin_typescript::all_rules(),
        starlint_plugin_testing::all_rules(),
        starlint_plugin_modules::all_rules(),
        starlint_plugin_nextjs::all_rules(),
        starlint_plugin_vue::all_rules(),
        starlint_plugin_jsdoc::all_rules(),
        starlint_plugin_storybook::all_rules(),
    ]
    .into_iter()
    .flatten()
    .collect();

    let plugin = LintRulePlugin::new(all_rules);
    group.bench_function("all_709_rules", |b| {
        let ctx = fixture.file_context();
        b.iter(|| black_box(plugin.lint_file(&ctx)));
    });

    group.finish();
}

// ── Criterion setup ──────────────────────────────────────────────────────────

criterion_group!(parse_group, bench_parse);

criterion_group!(core_group, bench_core);

criterion_group!(react_group, bench_react);

criterion_group!(typescript_group, bench_typescript);

criterion_group!(testing_group, bench_testing);

criterion_group!(modules_group, bench_modules);

criterion_group!(nextjs_group, bench_nextjs);

criterion_group!(vue_group, bench_vue);

criterion_group!(jsdoc_group, bench_jsdoc);

criterion_group!(storybook_group, bench_storybook);

criterion_group!(full_stack_group, bench_full_stack);

criterion_main!(
    parse_group,
    core_group,
    react_group,
    typescript_group,
    testing_group,
    modules_group,
    nextjs_group,
    vue_group,
    jsdoc_group,
    storybook_group,
    full_stack_group
);
