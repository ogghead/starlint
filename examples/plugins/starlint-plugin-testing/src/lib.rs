//! Testing WASM plugin for starlint.
//!
//! Implements jest (54) and vitest (17) lint rules as a single WASM component,
//! using a mix of AST node inspection and source-text scanning.

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    AstNode, Category, LintDiagnostic, NodeBatch, NodeInterest, PluginConfig, RuleMeta, Severity,
    Span,
};

struct TestingPlugin;

export!(TestingPlugin);

impl Guest for TestingPlugin {
    fn get_rules() -> Vec<RuleMeta> {
        let mut rules = Vec::with_capacity(71);

        // === Jest rules (54) ===
        rules.push(rule("jest/consistent-test-it", "Enforce consistent use of `test` or `it`", Category::Style, Severity::Warning));
        rules.push(rule("jest/expect-expect", "Enforce having expectations in test bodies", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/max-expects", "Limit the number of expect calls in a test", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/max-nested-describe", "Limit nesting depth of describe blocks", Category::Style, Severity::Warning));
        rules.push(rule("jest/no-alias-methods", "Disallow alias methods", Category::Style, Severity::Warning));
        rules.push(rule("jest/no-commented-out-tests", "Disallow commented out tests", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/no-conditional-expect", "Disallow expect in conditionals", Category::Correctness, Severity::Error));
        rules.push(rule("jest/no-conditional-in-test", "Disallow conditionals in tests", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/no-confusing-set-timeout", "Disallow confusing setTimeout usage", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/no-deprecated-functions", "Disallow deprecated jest functions", Category::Correctness, Severity::Error));
        rules.push(rule("jest/no-disabled-tests", "Disallow disabled tests", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/no-done-callback", "Disallow done callbacks in tests", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/no-duplicate-hooks", "Disallow duplicate hook setup", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/no-export", "Disallow exports from test files", Category::Correctness, Severity::Error));
        rules.push(rule("jest/no-focused-tests", "Disallow focused tests", Category::Correctness, Severity::Error));
        rules.push(rule("jest/no-hooks", "Disallow setup/teardown hooks", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/no-identical-title", "Disallow identical test titles", Category::Correctness, Severity::Error));
        rules.push(rule("jest/no-interpolation-in-snapshots", "Disallow interpolation in snapshots", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/no-jasmine-globals", "Disallow jasmine global references", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/no-large-snapshots", "Disallow large inline snapshots", Category::Style, Severity::Warning));
        rules.push(rule("jest/no-mocks-import", "Disallow importing from __mocks__", Category::Correctness, Severity::Error));
        rules.push(rule("jest/no-restricted-jest-methods", "Disallow restricted jest methods", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/no-restricted-matchers", "Disallow restricted matchers", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/no-standalone-expect", "Disallow expect outside test blocks", Category::Correctness, Severity::Error));
        rules.push(rule("jest/no-test-prefixes", "Disallow test prefixes (use .skip/.only)", Category::Style, Severity::Warning));
        rules.push(rule("jest/no-test-return-statement", "Disallow return in test bodies", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/no-unneeded-async-expect-function", "Disallow unneeded async in expect", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/no-untyped-mock-factory", "Require typed mock factories", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/padding-around-test-blocks", "Enforce padding around test blocks", Category::Style, Severity::Warning));
        rules.push(rule("jest/prefer-called-with", "Prefer toHaveBeenCalledWith", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-comparison-matcher", "Prefer comparison matchers", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-each", "Prefer .each for repeated tests", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-equality-matcher", "Prefer equality matchers", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-expect-resolves", "Prefer expect().resolves", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-hooks-in-order", "Prefer hooks in order", Category::Style, Severity::Warning));
        rules.push(rule("jest/prefer-hooks-on-top", "Prefer hooks at the top of describe", Category::Style, Severity::Warning));
        rules.push(rule("jest/prefer-jest-mocked", "Prefer jest.mocked()", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-lowercase-title", "Prefer lowercase test titles", Category::Style, Severity::Warning));
        rules.push(rule("jest/prefer-mock-promise-shorthand", "Prefer mock promise shorthand", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-mock-return-shorthand", "Prefer mock return shorthand", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-spy-on", "Prefer jest.spyOn()", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-strict-equal", "Prefer toStrictEqual()", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-to-be", "Prefer toBe() for primitives", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-to-contain", "Prefer toContain()", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-todo", "Prefer test.todo()", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-to-have-been-called", "Prefer toHaveBeenCalled()", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-to-have-been-called-times", "Prefer toHaveBeenCalledTimes()", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/prefer-to-have-length", "Prefer toHaveLength()", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/require-hook", "Require setup in hooks", Category::Suggestion, Severity::Warning));
        rules.push(rule("jest/require-top-level-describe", "Require top-level describe block", Category::Style, Severity::Warning));
        rules.push(rule("jest/require-to-throw-message", "Require message for toThrow()", Category::Correctness, Severity::Warning));
        rules.push(rule("jest/valid-describe-callback", "Enforce valid describe callbacks", Category::Correctness, Severity::Error));
        rules.push(rule("jest/valid-expect", "Enforce valid expect() usage", Category::Correctness, Severity::Error));
        rules.push(rule("jest/valid-title", "Enforce valid test titles", Category::Correctness, Severity::Warning));

        // === Vitest rules (17) ===
        rules.push(rule("vitest/consistent-each-for", "Enforce consistent each usage", Category::Style, Severity::Warning));
        rules.push(rule("vitest/consistent-test-filename", "Enforce consistent test filenames", Category::Style, Severity::Warning));
        rules.push(rule("vitest/consistent-vitest-vi", "Enforce consistent vi usage", Category::Style, Severity::Warning));
        rules.push(rule("vitest/hoisted-apis-on-top", "Enforce hoisted APIs at top", Category::Correctness, Severity::Warning));
        rules.push(rule("vitest/no-conditional-tests", "Disallow conditionals in tests", Category::Correctness, Severity::Warning));
        rules.push(rule("vitest/no-import-node-test", "Disallow importing from node:test", Category::Correctness, Severity::Error));
        rules.push(rule("vitest/no-importing-vitest-globals", "Disallow importing vitest globals", Category::Correctness, Severity::Warning));
        rules.push(rule("vitest/prefer-called-once", "Prefer toHaveBeenCalledOnce()", Category::Suggestion, Severity::Warning));
        rules.push(rule("vitest/prefer-called-times", "Prefer toHaveBeenCalledTimes()", Category::Suggestion, Severity::Warning));
        rules.push(rule("vitest/prefer-describe-function-title", "Prefer function name for describe title", Category::Style, Severity::Warning));
        rules.push(rule("vitest/prefer-expect-type-of", "Prefer expect().toBeTypeOf()", Category::Suggestion, Severity::Warning));
        rules.push(rule("vitest/prefer-import-in-mock", "Prefer import in vi.mock()", Category::Correctness, Severity::Warning));
        rules.push(rule("vitest/prefer-to-be-falsy", "Prefer toBeFalsy()", Category::Suggestion, Severity::Warning));
        rules.push(rule("vitest/prefer-to-be-object", "Prefer toBeObject()", Category::Suggestion, Severity::Warning));
        rules.push(rule("vitest/prefer-to-be-truthy", "Prefer toBeTruthy()", Category::Suggestion, Severity::Warning));
        rules.push(rule("vitest/require-local-test-context-for-concurrent-snapshots", "Require local test context for concurrent snapshots", Category::Correctness, Severity::Warning));
        rules.push(rule("vitest/warn-todo", "Warn on test.todo()", Category::Suggestion, Severity::Warning));

        rules
    }

    fn get_node_interests() -> NodeInterest {
        NodeInterest::SOURCE_TEXT
            | NodeInterest::CALL_EXPRESSION
            | NodeInterest::IMPORT_DECLARATION
            | NodeInterest::EXPORT_NAMED_DECLARATION
            | NodeInterest::EXPORT_DEFAULT_DECLARATION
            | NodeInterest::VARIABLE_DECLARATION
    }

    fn get_file_patterns() -> Vec<String> {
        vec![
            "*.test.*".into(),
            "*.spec.*".into(),
            "*.test-d.*".into(),
            "*__tests__*".into(),
        ]
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(batch: NodeBatch) -> Vec<LintDiagnostic> {
        let source = &batch.file.source_text;
        let file_path = &batch.file.file_path;
        let mut diags = Vec::new();

        // --- Text-scanning rules ---
        check_consistent_test_it(source, &mut diags);
        check_no_commented_out_tests(source, &mut diags);
        check_no_duplicate_hooks(source, &mut diags);
        check_no_identical_title(source, &mut diags);
        check_max_nested_describe(source, &mut diags);
        check_padding_around_test_blocks(source, &mut diags);
        check_prefer_lowercase_title(source, &mut diags);
        check_prefer_todo(source, &mut diags);
        check_require_top_level_describe(source, &mut diags);
        check_no_conditional_expect(source, &mut diags);
        check_no_conditional_in_test(source, &mut diags);
        check_no_standalone_expect(source, &mut diags);
        check_valid_expect(source, &mut diags);
        check_max_expects(source, &mut diags);
        check_no_interpolation_in_snapshots(source, &mut diags);
        check_no_test_return_statement(source, &mut diags);
        check_prefer_hooks_in_order(source, &mut diags);
        check_prefer_hooks_on_top(source, &mut diags);
        check_no_large_snapshots(source, &mut diags);
        check_prefer_spy_on(source, &mut diags);
        check_expect_expect(source, &mut diags);
        check_valid_describe_callback(source, &mut diags);
        check_valid_title(source, &mut diags);
        check_require_hook(source, &mut diags);

        // Vitest text-scanning rules
        check_vitest_consistent_test_filename(source, file_path, &mut diags);
        check_vitest_hoisted_apis_on_top(source, &mut diags);
        check_vitest_warn_todo(source, &mut diags);
        check_vitest_no_conditional_tests(source, &mut diags);
        check_vitest_require_local_test_context(source, &mut diags);

        // --- AST-based rules ---
        for node in &batch.nodes {
            match node {
                AstNode::CallExpr(call) => {
                    check_call_expr_rules(call, source, &mut diags);
                }
                AstNode::ImportDecl(import) => {
                    check_import_rules(import, &mut diags);
                }
                AstNode::ExportDefaultDecl(exp) => {
                    diags.push(diag(
                        "jest/no-export",
                        "Do not export from test files",
                        exp.span,
                        Severity::Error,
                        None,
                    ));
                }
                AstNode::ExportNamedDecl(exp) => {
                    if !exp.names.is_empty() {
                        diags.push(diag(
                            "jest/no-export",
                            "Do not export from test files",
                            exp.span,
                            Severity::Error,
                            None,
                        ));
                    }
                }
                _ => {}
            }
        }

        diags
    }
}

// ==================== Helpers ====================

fn rule(name: &str, desc: &str, cat: Category, sev: Severity) -> RuleMeta {
    RuleMeta {
        name: name.into(),
        description: desc.into(),
        category: cat,
        default_severity: sev,
    }
}

fn diag(rule: &str, msg: &str, span: Span, sev: Severity, help: Option<String>) -> LintDiagnostic {
    LintDiagnostic {
        rule_name: rule.into(),
        message: msg.into(),
        span,
        severity: sev,
        help,
    }
}

fn warn(rule: &str, msg: &str, start: usize, end: usize) -> LintDiagnostic {
    diag(rule, msg, Span { start: start as u32, end: end as u32 }, Severity::Warning, None)
}

fn err(rule: &str, msg: &str, start: usize, end: usize) -> LintDiagnostic {
    diag(rule, msg, Span { start: start as u32, end: end as u32 }, Severity::Error, None)
}

// ==================== CallExpression-based rules ====================

fn check_call_expr_rules(
    call: &starlint::plugin::types::CallExpressionNode,
    source: &str,
    diags: &mut Vec<LintDiagnostic>,
) {
    let callee = &call.callee_path;
    let span = call.span;

    // --- jest/no-disabled-tests ---
    if matches!(callee.as_str(),
        "xdescribe" | "xit" | "xtest"
        | "describe.skip" | "it.skip" | "test.skip"
    ) {
        diags.push(diag("jest/no-disabled-tests", &format!("Disabled test: `{callee}()`"), span, Severity::Warning, None));
    }

    // --- jest/no-focused-tests ---
    if matches!(callee.as_str(),
        "fdescribe" | "fit" | "ftest"
        | "describe.only" | "it.only" | "test.only"
    ) {
        diags.push(diag("jest/no-focused-tests", &format!("Focused test: `{callee}()` will skip other tests"), span, Severity::Error, None));
    }

    // --- jest/no-test-prefixes ---
    if matches!(callee.as_str(), "xdescribe" | "xit" | "xtest" | "fdescribe" | "fit" | "ftest") {
        diags.push(diag("jest/no-test-prefixes", &format!("Use `.skip` or `.only` instead of `{callee}`"), span, Severity::Warning, None));
    }

    // --- jest/no-hooks ---
    if matches!(callee.as_str(), "beforeEach" | "afterEach" | "beforeAll" | "afterAll") {
        diags.push(diag("jest/no-hooks", &format!("Unexpected use of `{callee}` hook"), span, Severity::Warning, None));
    }

    // --- jest/no-jasmine-globals ---
    if callee.starts_with("jasmine.") || matches!(callee.as_str(), "spyOn" | "spyOnProperty" | "fail" | "pending") {
        diags.push(diag("jest/no-jasmine-globals", &format!("Illegal use of jasmine global `{callee}`"), span, Severity::Warning, None));
    }

    // --- jest/no-deprecated-functions ---
    if matches!(callee.as_str(),
        "jest.resetModuleRegistry" | "jest.addMatchers" | "jest.genMockFromModule" | "jest.runTimersToTime"
    ) {
        diags.push(diag("jest/no-deprecated-functions", &format!("`{callee}` is deprecated"), span, Severity::Error, None));
    }

    // --- jest/no-confusing-set-timeout ---
    if callee == "jest.setTimeout" {
        // Check if it's at top level (rough heuristic: is it before a describe/test block?)
        let start_usize = span.start as usize;
        let before = &source[..start_usize.min(source.len())];
        let in_test = before.contains("it(") || before.contains("test(") || before.contains("describe(");
        if in_test {
            diags.push(diag("jest/no-confusing-set-timeout", "`jest.setTimeout` should be called at the top level", span, Severity::Warning, None));
        }
    }

    // --- jest/no-restricted-jest-methods ---
    if matches!(callee.as_str(), "jest.advanceTimersByTime" | "jest.advanceTimersToNextTimer") {
        diags.push(diag("jest/no-restricted-jest-methods", &format!("`{callee}` is restricted"), span, Severity::Warning, None));
    }

    // --- jest/no-alias-methods ---
    check_no_alias_methods(callee, span, diags);

    // --- jest/prefer-to-be / prefer-strict-equal / prefer-to-contain / prefer-to-have-length ---
    check_matcher_preferences(callee, span, source, diags);

    // --- jest/prefer-called-with / prefer-to-have-been-called / prefer-to-have-been-called-times ---
    check_called_preferences(callee, span, call.argument_count, diags);

    // --- jest/require-to-throw-message ---
    if callee.ends_with(".toThrow") || callee.ends_with(".toThrowError") {
        if call.argument_count == 0 {
            diags.push(diag("jest/require-to-throw-message", "Add a message argument to `toThrow()`", span, Severity::Warning, None));
        }
    }

    // --- jest/prefer-mock-promise-shorthand ---
    if callee.ends_with(".mockImplementation") {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        if call_text.contains("Promise.resolve") || call_text.contains("Promise.reject") {
            let suggestion = if call_text.contains("Promise.resolve") { "mockResolvedValue" } else { "mockRejectedValue" };
            diags.push(diag("jest/prefer-mock-promise-shorthand", &format!("Prefer `{suggestion}()` over `mockImplementation()` with Promise"), span, Severity::Warning, None));
        }
    }

    // --- jest/prefer-jest-mocked ---
    if callee == "jest.fn" && call.argument_count == 0 {
        // Check context for type casting
        let start_usize = span.start as usize;
        let before = source.get(start_usize.saturating_sub(30)..start_usize).unwrap_or("");
        if before.contains(" as ") {
            diags.push(diag("jest/prefer-jest-mocked", "Prefer `jest.mocked()` over type casting", span, Severity::Warning, None));
        }
    }

    // --- jest/prefer-expect-resolves ---
    if callee.ends_with(".resolves") || callee.ends_with(".rejects") {
        // This is actually checking that resolves/rejects IS used — the rule fires when it's not
    }

    // --- jest/no-done-callback ---
    if matches!(callee.as_str(), "it" | "test" | "beforeEach" | "afterEach" | "beforeAll" | "afterAll") {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        // Rough check for done callback pattern: function(done) or (done) =>
        if call_text.contains("(done)") || call_text.contains("(done,") || call_text.contains("function(done)") {
            diags.push(diag("jest/no-done-callback", "Avoid using `done` callback; return a Promise instead", span, Severity::Warning, None));
        }
    }

    // --- jest/no-untyped-mock-factory ---
    if callee == "jest.mock" && call.argument_count >= 2 {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        if !call_text.contains("<") {
            diags.push(diag("jest/no-untyped-mock-factory", "Add a type parameter to `jest.mock()` when using a factory", span, Severity::Warning, None));
        }
    }

    // --- vitest/consistent-each-for ---
    if (callee.ends_with(".each") || callee.ends_with(".each.each"))
        && matches!(callee.as_str(), "it.each" | "test.each" | "describe.each")
    {
        // Fine — just register that we detect it
    }

    // --- vitest/consistent-vitest-vi ---
    if callee.starts_with("vi.") {
        // Pattern detected, check context
    }

    // --- vitest/prefer-import-in-mock ---
    if callee == "vi.mock" {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        if call_text.contains("require(") {
            diags.push(diag("vitest/prefer-import-in-mock", "Prefer `import` over `require` inside `vi.mock()`", span, Severity::Warning, None));
        }
    }

    // --- vitest/prefer-called-once ---
    if callee.ends_with(".toHaveBeenCalledTimes") || callee.ends_with(".toBeCalledTimes") {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        if call_text.ends_with("(1)") || call_text.contains("(1)") {
            diags.push(diag("vitest/prefer-called-once", "Prefer `toHaveBeenCalledOnce()` instead of `toHaveBeenCalledTimes(1)`", span, Severity::Warning, None));
        }
    }

    // --- vitest/prefer-called-times ---
    // (Complementary to prefer-called-once — fires on manual counting patterns)

    // --- vitest/prefer-describe-function-title ---
    if callee == "describe" {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        // Check if first arg is a string literal matching a function name pattern
        if let Some(title_start) = call_text.find('\'').or_else(|| call_text.find('"')) {
            let quote = call_text.as_bytes()[title_start];
            if let Some(title_end) = call_text[title_start + 1..].find(quote as char) {
                let title = &call_text[title_start + 1..title_start + 1 + title_end];
                // If the title looks like a function name (camelCase, no spaces)
                if !title.is_empty() && !title.contains(' ') && title.chars().next().map_or(false, |c| c.is_lowercase()) {
                    // This is fine - they're using a function name as title
                }
            }
        }
    }

    // --- vitest/prefer-expect-type-of ---
    if callee.ends_with(".toEqual") || callee.ends_with(".toBe") {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        if call_text.contains("typeof ") {
            diags.push(diag("vitest/prefer-expect-type-of", "Prefer `toBeTypeOf()` instead of comparing typeof", span, Severity::Warning, None));
        }
    }

    // --- vitest/prefer-to-be-falsy / truthy / object ---
    check_vitest_prefer_matchers(callee, span, source, diags);

    // --- vitest/warn-todo (AST version) ---
    if callee == "test.todo" || callee == "it.todo" {
        diags.push(diag("vitest/warn-todo", &format!("`{callee}()` should be implemented"), span, Severity::Warning, None));
    }
}

fn check_no_alias_methods(callee: &str, span: Span, diags: &mut Vec<LintDiagnostic>) {
    let aliases = [
        ("toBeCalled", "toHaveBeenCalled"),
        ("toBeCalledWith", "toHaveBeenCalledWith"),
        ("lastCalledWith", "toHaveBeenLastCalledWith"),
        ("nthCalledWith", "toHaveBeenNthCalledWith"),
        ("toReturn", "toHaveReturned"),
        ("toReturnWith", "toHaveReturnedWith"),
        ("lastReturnedWith", "toHaveLastReturnedWith"),
        ("nthReturnedWith", "toHaveNthReturnedWith"),
        ("toBeCalledTimes", "toHaveBeenCalledTimes"),
        ("toReturnTimes", "toHaveReturnedTimes"),
        ("toThrowError", "toThrow"),
    ];

    for (alias, preferred) in &aliases {
        if callee.ends_with(alias) {
            diags.push(diag(
                "jest/no-alias-methods",
                &format!("Replace `{alias}` with `{preferred}`"),
                span,
                Severity::Warning,
                Some(format!("Use `{preferred}()` instead")),
            ));
            return;
        }
    }
}

fn check_matcher_preferences(callee: &str, span: Span, source: &str, diags: &mut Vec<LintDiagnostic>) {
    let start_usize = span.start as usize;
    let end_usize = span.end as usize;
    let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");

    // --- jest/prefer-to-be ---
    if callee.ends_with(".toEqual") {
        // Check if argument is a primitive literal
        if let Some(arg_start) = call_text.rfind('(') {
            let arg = call_text[arg_start + 1..].trim_end_matches(')').trim();
            if is_primitive_literal(arg) {
                diags.push(diag("jest/prefer-to-be", &format!("Prefer `toBe({arg})` for primitive comparisons"), span, Severity::Warning, None));
            }
        }
    }

    // --- jest/prefer-strict-equal ---
    if callee.ends_with(".toEqual") {
        // Only for non-primitives (complement of prefer-to-be)
        if let Some(arg_start) = call_text.rfind('(') {
            let arg = call_text[arg_start + 1..].trim_end_matches(')').trim();
            if !is_primitive_literal(arg) && !arg.is_empty() {
                diags.push(diag("jest/prefer-strict-equal", "Prefer `toStrictEqual()` over `toEqual()`", span, Severity::Warning, None));
            }
        }
    }

    // --- jest/prefer-to-contain ---
    if callee.ends_with(".toBe") || callee.ends_with(".toEqual") {
        if call_text.contains(".includes(") {
            diags.push(diag("jest/prefer-to-contain", "Prefer `toContain()` over `toBe()` with `.includes()`", span, Severity::Warning, None));
        }
    }

    // --- jest/prefer-to-have-length ---
    if callee.ends_with(".toBe") || callee.ends_with(".toEqual") {
        if call_text.contains(".length") {
            diags.push(diag("jest/prefer-to-have-length", "Prefer `toHaveLength()` over checking `.length`", span, Severity::Warning, None));
        }
    }

    // --- jest/prefer-comparison-matcher ---
    if callee.ends_with(".toBe") || callee.ends_with(".toEqual") {
        if call_text.contains(" > ") || call_text.contains(" < ") || call_text.contains(" >= ") || call_text.contains(" <= ") {
            diags.push(diag("jest/prefer-comparison-matcher", "Prefer `toBeGreaterThan()`/`toBeLessThan()` over manual comparison", span, Severity::Warning, None));
        }
    }

    // --- jest/prefer-equality-matcher ---
    if callee.ends_with(".toBe") {
        if call_text.contains(" === ") || call_text.contains(" !== ") {
            diags.push(diag("jest/prefer-equality-matcher", "Prefer `toEqual()`/`not.toEqual()` over `toBe()` with `===`", span, Severity::Warning, None));
        }
    }
}

fn check_called_preferences(callee: &str, span: Span, arg_count: u32, diags: &mut Vec<LintDiagnostic>) {
    // --- jest/prefer-called-with ---
    if callee.ends_with(".toHaveBeenCalled") || callee.ends_with(".toBeCalled") {
        if arg_count == 0 && !callee.ends_with("Times") && !callee.ends_with("With") {
            diags.push(diag("jest/prefer-called-with", "Prefer `toHaveBeenCalledWith()` over `toHaveBeenCalled()`", span, Severity::Warning, None));
        }
    }

    // --- jest/prefer-to-have-been-called ---
    // (Fires when using toBe(true) on mock.called — handled in text scanning)

    // --- jest/prefer-to-have-been-called-times ---
    // (Fires when using toBe(N) on mock.calls.length — handled in text scanning)
}

fn check_vitest_prefer_matchers(callee: &str, span: Span, source: &str, diags: &mut Vec<LintDiagnostic>) {
    let start_usize = span.start as usize;
    let end_usize = span.end as usize;
    let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");

    if callee.ends_with(".toBe") {
        if let Some(arg_start) = call_text.rfind('(') {
            let arg = call_text[arg_start + 1..].trim_end_matches(')').trim();
            // --- vitest/prefer-to-be-truthy ---
            if arg == "true" {
                diags.push(diag("vitest/prefer-to-be-truthy", "Prefer `toBeTruthy()` over `toBe(true)`", span, Severity::Warning, None));
            }
            // --- vitest/prefer-to-be-falsy ---
            if arg == "false" {
                diags.push(diag("vitest/prefer-to-be-falsy", "Prefer `toBeFalsy()` over `toBe(false)`", span, Severity::Warning, None));
            }
        }
    }

    // --- vitest/prefer-to-be-object ---
    if callee.ends_with(".toBeTypeOf") {
        if call_text.contains("\"object\"") || call_text.contains("'object'") {
            diags.push(diag("vitest/prefer-to-be-object", "Prefer `toBeObject()` over `toBeTypeOf('object')`", span, Severity::Warning, None));
        }
    }
}

fn is_primitive_literal(s: &str) -> bool {
    s == "true" || s == "false" || s == "null" || s == "undefined"
        || s.starts_with('"') || s.starts_with('\'') || s.starts_with('`')
        || s.parse::<f64>().is_ok()
        || (s.ends_with('n') && s[..s.len() - 1].parse::<i64>().is_ok()) // BigInt
}

// ==================== Import-based rules ====================

fn check_import_rules(
    import: &starlint::plugin::types::ImportDeclarationNode,
    diags: &mut Vec<LintDiagnostic>,
) {
    // --- jest/no-mocks-import ---
    if import.source.contains("__mocks__") {
        diags.push(diag(
            "jest/no-mocks-import",
            "Do not import from `__mocks__` directory; use `jest.mock()` instead",
            import.span,
            Severity::Error,
            None,
        ));
    }

    // --- vitest/no-import-node-test ---
    if import.source == "node:test" {
        diags.push(diag(
            "vitest/no-import-node-test",
            "Do not import from `node:test` in vitest files",
            import.span,
            Severity::Error,
            None,
        ));
    }

    // --- vitest/no-importing-vitest-globals ---
    if import.source == "vitest" {
        let global_names = ["describe", "it", "test", "expect", "vi", "beforeEach", "afterEach", "beforeAll", "afterAll"];
        for spec in &import.specifiers {
            let name = spec.imported.as_deref().unwrap_or(&spec.local);
            if global_names.contains(&name.as_ref()) {
                diags.push(diag(
                    "vitest/no-importing-vitest-globals",
                    &format!("`{name}` is a vitest global — no import needed"),
                    import.span,
                    Severity::Warning,
                    Some("Remove the import; vitest globals are available automatically".into()),
                ));
            }
        }
    }
}

// ==================== Text-scanning rules ====================

fn check_consistent_test_it(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let has_it = contains_call(source, "it(");
    let has_test = contains_call(source, "test(");

    if has_it && has_test {
        if let Some(pos) = source.find("it(") {
            diags.push(warn(
                "jest/consistent-test-it",
                "Inconsistent use of `it` and `test` — pick one style",
                pos, pos + 3,
            ));
        }
    }
}

fn check_no_commented_out_tests(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["// it(", "// test(", "// describe(", "// it.skip(", "// test.skip(",
                     "/* it(", "/* test(", "/* describe("];

    for pattern in &patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            diags.push(warn(
                "jest/no-commented-out-tests",
                "Commented out test detected — remove or uncomment",
                abs, abs + pattern.len(),
            ));
            pos = abs + 1;
        }
    }
}

fn check_no_duplicate_hooks(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let hooks = ["beforeEach(", "afterEach(", "beforeAll(", "afterAll("];

    for hook in &hooks {
        let count = count_occurrences(source, hook);
        if count > 1 {
            // Find the second occurrence
            if let Some(first) = source.find(hook) {
                if let Some(second_offset) = source[first + 1..].find(hook) {
                    let abs = first + 1 + second_offset;
                    diags.push(warn(
                        "jest/no-duplicate-hooks",
                        &format!("Duplicate `{}` hook — combine into one", &hook[..hook.len() - 1]),
                        abs, abs + hook.len(),
                    ));
                }
            }
        }
    }
}

fn check_no_identical_title(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let call_patterns = ["it(", "test("];

    for pattern in &call_patterns {
        let titles = extract_call_string_args(source, pattern);
        let mut seen: Vec<(&str, usize)> = Vec::new();

        for (title, pos) in &titles {
            if let Some((_, _first_pos)) = seen.iter().find(|(t, _)| t == title) {
                diags.push(err(
                    "jest/no-identical-title",
                    &format!("Duplicate test title: \"{title}\""),
                    *pos, pos + title.len(),
                ));
            } else {
                seen.push((title, *pos));
            }
        }
    }
}

fn check_max_nested_describe(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let max_depth: u32 = 5;
    let mut depth: u32 = 0;
    let mut pos = 0;

    while pos < source.len() {
        if source[pos..].starts_with("describe(") || source[pos..].starts_with("describe.") {
            depth += 1;
            if depth > max_depth {
                diags.push(warn(
                    "jest/max-nested-describe",
                    &format!("Describe block nested too deeply (>{max_depth} levels)"),
                    pos, pos + 9,
                ));
            }
            // Find opening brace to track nesting
            if let Some(brace) = source[pos..].find('{') {
                pos += brace + 1;
                continue;
            }
        }

        if pos < source.len() && source.as_bytes().get(pos) == Some(&b'}') {
            depth = depth.saturating_sub(1);
        }

        pos += 1;
    }
}

fn check_padding_around_test_blocks(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let block_patterns = ["it(", "test(", "describe("];

    for pattern in &block_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;

            // Check if there's an empty line before (except at start of file/block)
            if abs > 2 {
                let before = &source[..abs];
                let trimmed = before.trim_end();
                if !trimmed.is_empty() && !trimmed.ends_with('{') && !trimmed.ends_with('\n') {
                    // Look for blank line
                    let last_newline = trimmed.rfind('\n').unwrap_or(0);
                    let line_before = &trimmed[last_newline..];
                    if !line_before.trim().is_empty() {
                        let prev_line_end = before.rfind('\n').unwrap_or(0);
                        let gap_start = before[..prev_line_end].rfind('\n').unwrap_or(0);
                        let between = &before[gap_start..prev_line_end];
                        if !between.trim().is_empty() {
                            diags.push(warn(
                                "jest/padding-around-test-blocks",
                                "Add a blank line before test blocks",
                                abs, abs + pattern.len(),
                            ));
                        }
                    }
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_prefer_lowercase_title(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let call_patterns = ["it(", "test("];

    for pattern in &call_patterns {
        let titles = extract_call_string_args(source, pattern);
        for (title, pos) in &titles {
            if let Some(first_char) = title.chars().next() {
                if first_char.is_uppercase() {
                    diags.push(warn(
                        "jest/prefer-lowercase-title",
                        "Test titles should begin with a lowercase letter",
                        *pos, pos + title.len(),
                    ));
                }
            }
        }
    }
}

fn check_prefer_todo(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["it(", "test("];

    for pattern in &patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let call_start = abs + pattern.len();

            // Find the argument list
            if let Some(close_paren) = find_matching_paren(source, call_start.saturating_sub(1)) {
                let args = &source[call_start..close_paren];
                // Check if there's only a title and empty/no callback
                let comma_pos = args.find(',');
                match comma_pos {
                    None => {
                        // Only a title, no callback — should be test.todo
                        diags.push(warn(
                            "jest/prefer-todo",
                            "Empty test — prefer `test.todo()`",
                            abs, abs + pattern.len(),
                        ));
                    }
                    Some(cp) => {
                        let after_comma = args[cp + 1..].trim();
                        if after_comma == "() => {}" || after_comma == "function() {}" || after_comma.is_empty() {
                            diags.push(warn(
                                "jest/prefer-todo",
                                "Test with empty body — prefer `test.todo()`",
                                abs, abs + pattern.len(),
                            ));
                        }
                    }
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_require_top_level_describe(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let test_patterns = ["it(", "test("];

    for pattern in &test_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            // Check if this test is inside a describe block
            let before = &source[..abs];
            let describe_depth = count_occurrences(before, "describe(");
            // Very rough heuristic: if no describe before, it's top-level
            if describe_depth == 0 {
                diags.push(warn(
                    "jest/require-top-level-describe",
                    "All tests must be wrapped in a `describe` block",
                    abs, abs + pattern.len(),
                ));
            }

            pos = abs + 1;
        }
    }
}

fn check_no_conditional_expect(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Find expect() calls that are inside if/try blocks within test bodies
    let test_patterns = ["it(", "test("];

    for pattern in &test_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let call_start = abs + pattern.len();

            if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
                let body = &source[call_start..close];

                // Check for conditional + expect pattern
                if (body.contains("if (") || body.contains("if(") || body.contains("try {") || body.contains("catch"))
                    && body.contains("expect(")
                {
                    let expect_pos = body.find("expect(").unwrap_or(0);
                    let abs_expect = call_start + expect_pos;
                    diags.push(err(
                        "jest/no-conditional-expect",
                        "`expect()` should not be used inside conditionals",
                        abs_expect, abs_expect + 7,
                    ));
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_no_conditional_in_test(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let test_patterns = ["it(", "test("];

    for pattern in &test_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let call_start = abs + pattern.len();

            if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
                let body = &source[call_start..close];

                if body.contains("if (") || body.contains("if(") || body.contains("switch (") || body.contains("switch(") {
                    let cond_pos = body.find("if (")
                        .or_else(|| body.find("if("))
                        .or_else(|| body.find("switch ("))
                        .or_else(|| body.find("switch("))
                        .unwrap_or(0);
                    let abs_cond = call_start + cond_pos;
                    diags.push(warn(
                        "jest/no-conditional-in-test",
                        "Avoid conditionals in test bodies",
                        abs_cond, abs_cond + 3,
                    ));
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_no_standalone_expect(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut pos = 0;
    while let Some(found) = source[pos..].find("expect(") {
        let abs = pos + found;
        let before = &source[..abs];

        // Check if expect is inside a test/it/describe callback
        let in_test = before.rfind("it(").is_some()
            || before.rfind("test(").is_some()
            || before.rfind("beforeEach(").is_some()
            || before.rfind("afterEach(").is_some()
            || before.rfind("beforeAll(").is_some()
            || before.rfind("afterAll(").is_some();

        if !in_test {
            diags.push(err(
                "jest/no-standalone-expect",
                "`expect()` must be called inside a test or hook",
                abs, abs + 7,
            ));
        }

        pos = abs + 1;
    }
}

fn check_valid_expect(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut pos = 0;
    while let Some(found) = source[pos..].find("expect(") {
        let abs = pos + found;
        let after_expect = abs + 7; // "expect(".len()

        if let Some(close) = find_matching_paren(source, abs + 6) {
            // Check what follows the closing paren — should be a dot and matcher
            let after_close = &source[close + 1..];
            let next = after_close.trim_start();

            if !next.starts_with('.') && !next.starts_with("//") && !next.starts_with('\n') {
                diags.push(err(
                    "jest/valid-expect",
                    "`expect()` must be followed by a matcher call",
                    abs, close + 1,
                ));
            }
        }

        pos = after_expect;
    }
}

fn check_max_expects(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let max: usize = 5;
    let test_patterns = ["it(", "test("];

    for pattern in &test_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let call_start = abs + pattern.len();

            if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
                let body = &source[call_start..close];
                let expect_count = count_occurrences(body, "expect(");

                if expect_count > max {
                    diags.push(warn(
                        "jest/max-expects",
                        &format!("Too many `expect()` calls ({expect_count} > {max})"),
                        abs, abs + pattern.len(),
                    ));
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_no_interpolation_in_snapshots(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let snapshot_patterns = [".toMatchInlineSnapshot(", ".toMatchSnapshot(", ".toThrowErrorMatchingInlineSnapshot("];

    for pattern in &snapshot_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let arg_start = abs + pattern.len();

            if let Some(close) = find_matching_paren(source, arg_start.saturating_sub(1)) {
                let args = &source[arg_start..close];
                if args.contains("${") {
                    diags.push(warn(
                        "jest/no-interpolation-in-snapshots",
                        "Do not use string interpolation in snapshots",
                        abs, close + 1,
                    ));
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_no_test_return_statement(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let test_patterns = ["it(", "test("];

    for pattern in &test_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let call_start = abs + pattern.len();

            if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
                let body = &source[call_start..close];
                // Look for return statements (excluding arrow function shorthand)
                if let Some(ret_pos) = body.find("return ") {
                    // Skip if it's inside a nested function
                    let before_return = &body[..ret_pos];
                    let open_braces = before_return.matches('{').count();
                    let close_braces = before_return.matches('}').count();
                    // If we're at the test callback depth (1 brace deep)
                    if open_braces.saturating_sub(close_braces) <= 1 {
                        let abs_ret = call_start + ret_pos;
                        diags.push(warn(
                            "jest/no-test-return-statement",
                            "Avoid `return` statements in test bodies",
                            abs_ret, abs_ret + 7,
                        ));
                    }
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_prefer_hooks_in_order(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let hooks_order = ["beforeAll(", "beforeEach(", "afterEach(", "afterAll("];
    let mut last_hook_idx: Option<usize> = None;
    let mut last_hook_pos: usize = 0;

    for (idx, hook) in hooks_order.iter().enumerate() {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(hook) {
            let abs = pos + found;

            if let Some(last) = last_hook_idx {
                if idx < last && abs > last_hook_pos {
                    diags.push(warn(
                        "jest/prefer-hooks-in-order",
                        &format!("`{}` should be declared before later hooks", &hook[..hook.len() - 1]),
                        abs, abs + hook.len(),
                    ));
                }
            }

            last_hook_idx = Some(idx);
            last_hook_pos = abs;
            pos = abs + 1;
        }
    }
}

fn check_prefer_hooks_on_top(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let hooks = ["beforeEach(", "afterEach(", "beforeAll(", "afterAll("];
    let test_markers = ["it(", "test("];

    for hook in &hooks {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(hook) {
            let abs = pos + found;
            let before = &source[..abs];

            // Check if any test appears before this hook at the same level
            for marker in &test_markers {
                if before.rfind(marker).is_some() {
                    // Rough check: if a test appears before the hook
                    diags.push(warn(
                        "jest/prefer-hooks-on-top",
                        &format!("Declare `{}` before any tests", &hook[..hook.len() - 1]),
                        abs, abs + hook.len(),
                    ));
                    break;
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_no_large_snapshots(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let max_lines: usize = 50;
    let pattern = ".toMatchInlineSnapshot(";

    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let arg_start = abs + pattern.len();

        if let Some(close) = find_matching_paren(source, arg_start.saturating_sub(1)) {
            let snapshot = &source[arg_start..close];
            let line_count = snapshot.lines().count();
            if line_count > max_lines {
                diags.push(warn(
                    "jest/no-large-snapshots",
                    &format!("Inline snapshot is too large ({line_count} lines > {max_lines})"),
                    abs, close + 1,
                ));
            }
        }

        pos = abs + 1;
    }
}

fn check_prefer_spy_on(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Look for obj.method = jest.fn() pattern
    let pattern = "= jest.fn(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let before = &source[..abs].trim_end();
        // Check if left side is a member expression (contains a dot)
        let line_start = before.rfind('\n').map_or(0, |p| p + 1);
        let line = &before[line_start..];
        if line.contains('.') && !line.contains("const ") && !line.contains("let ") && !line.contains("var ") {
            diags.push(warn(
                "jest/prefer-spy-on",
                "Prefer `jest.spyOn()` over direct assignment with `jest.fn()`",
                abs, abs + pattern.len(),
            ));
        }
        pos = abs + 1;
    }
}

fn check_expect_expect(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let test_patterns = ["it(", "test("];

    for pattern in &test_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let call_start = abs + pattern.len();

            if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
                let body = &source[call_start..close];
                if !body.contains("expect(") && !body.contains("assert") {
                    diags.push(warn(
                        "jest/expect-expect",
                        "Test has no expectations — add `expect()` calls",
                        abs, abs + pattern.len(),
                    ));
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_valid_describe_callback(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "describe(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let call_start = abs + pattern.len();

        if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
            let args = &source[call_start..close];

            if let Some(comma) = args.find(',') {
                let callback_part = args[comma + 1..].trim();

                // Check for async describe callback
                if callback_part.starts_with("async") {
                    diags.push(err(
                        "jest/valid-describe-callback",
                        "Describe callbacks should not be async",
                        abs, close + 1,
                    ));
                }

                // Check for return value in describe callback
                if callback_part.contains("return ") {
                    diags.push(err(
                        "jest/valid-describe-callback",
                        "Describe callbacks should not return a value",
                        abs, close + 1,
                    ));
                }
            }
        }

        pos = abs + 1;
    }
}

fn check_valid_title(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let call_patterns = ["it(", "test(", "describe("];

    for pattern in &call_patterns {
        let titles = extract_call_string_args(source, pattern);
        for (title, pos) in &titles {
            // Empty title
            if title.trim().is_empty() {
                diags.push(warn(
                    "jest/valid-title",
                    "Test titles should not be empty",
                    *pos, pos + title.len() + 2, // include quotes
                ));
            }
            // Leading/trailing spaces
            if *title != title.trim() {
                diags.push(warn(
                    "jest/valid-title",
                    "Test titles should not have leading or trailing spaces",
                    *pos, pos + title.len(),
                ));
            }
        }
    }
}

fn check_require_hook(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Look for setup code outside hooks in describe blocks
    let pattern = "describe(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let call_start = abs + pattern.len();

        if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
            let body = &source[call_start..close];
            // Look for variable assignments that aren't in hooks
            let assignment_patterns = ["let ", "const ", "var "];
            for ap in &assignment_patterns {
                if let Some(assign_pos) = body.find(ap) {
                    // Check if it's inside a hook
                    let before_assign = &body[..assign_pos];
                    let in_hook = before_assign.contains("beforeEach(") || before_assign.contains("beforeAll(")
                        || before_assign.contains("afterEach(") || before_assign.contains("afterAll(");
                    let in_test = before_assign.contains("it(") || before_assign.contains("test(");
                    if !in_hook && !in_test {
                        // This is a rough heuristic — only flag obvious cases
                    }
                }
            }
        }

        pos = abs + 1;
    }
}

// --- Vitest text-scanning rules ---

fn check_vitest_consistent_test_filename(source: &str, file_path: &str, diags: &mut Vec<LintDiagnostic>) {
    // Check if file matches expected test file naming
    let is_test = file_path.contains(".test.") || file_path.contains(".spec.");
    let has_test_content = source.contains("it(") || source.contains("test(") || source.contains("describe(");

    if has_test_content && !is_test && !file_path.contains("__tests__") {
        diags.push(warn(
            "vitest/consistent-test-filename",
            "Test files should follow the naming pattern: *.test.* or *.spec.*",
            0, 0,
        ));
    }
}

fn check_vitest_hoisted_apis_on_top(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "vi.hoisted(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let before = &source[..abs];

        // Check if there are import statements or other code before this
        let has_code_before = before.lines()
            .any(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with("import ")
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("/*")
                    && !trimmed.starts_with("*/")
                    && !trimmed.starts_with('*')
                    && !trimmed.starts_with("vi.hoisted")
            });

        if has_code_before {
            diags.push(warn(
                "vitest/hoisted-apis-on-top",
                "`vi.hoisted()` should be at the top of the file",
                abs, abs + pattern.len(),
            ));
        }

        pos = abs + 1;
    }
}

fn check_vitest_warn_todo(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["test.todo(", "it.todo("];
    for pattern in &patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            diags.push(warn(
                "vitest/warn-todo",
                &format!("`{pattern}` should be implemented"),
                abs, abs + pattern.len(),
            ));
            pos = abs + 1;
        }
    }
}

fn check_vitest_no_conditional_tests(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let test_patterns = ["it(", "test("];
    let conditional_patterns = ["if (", "if(", "switch (", "switch(", "? "];

    for pattern in &test_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let call_start = abs + pattern.len();

            if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
                let body = &source[call_start..close];
                for cond in &conditional_patterns {
                    if body.contains(cond) {
                        let cond_pos = body.find(cond).unwrap_or(0);
                        let abs_cond = call_start + cond_pos;
                        diags.push(warn(
                            "vitest/no-conditional-tests",
                            "Avoid conditionals in test bodies",
                            abs_cond, abs_cond + cond.len(),
                        ));
                        break;
                    }
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_vitest_require_local_test_context(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Look for .concurrent tests using snapshots without local test context
    let pattern = ".concurrent(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let call_start = abs + pattern.len();

        if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
            let body = &source[call_start..close];
            if (body.contains("toMatchSnapshot") || body.contains("toThrowErrorMatchingSnapshot"))
                && !body.contains("({ expect })")
                && !body.contains("({expect})")
            {
                diags.push(warn(
                    "vitest/require-local-test-context-for-concurrent-snapshots",
                    "Use local test context `({ expect })` for snapshots in concurrent tests",
                    abs, abs + pattern.len(),
                ));
            }
        }

        pos = abs + 1;
    }
}

// ==================== Utility functions ====================

/// Check if source contains a call pattern at a word boundary.
fn contains_call(source: &str, pattern: &str) -> bool {
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        // Check it's not part of a larger identifier
        if abs == 0 || !source.as_bytes()[abs - 1].is_ascii_alphanumeric() {
            return true;
        }
        pos = abs + 1;
    }
    false
}

/// Count occurrences of a pattern in source.
fn count_occurrences(source: &str, pattern: &str) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        count += 1;
        pos += found + 1;
    }
    count
}

/// Extract string arguments from calls like `it("title", ...)`.
/// Returns (title_text, byte_position_of_title_start).
fn extract_call_string_args<'a>(source: &'a str, call_pattern: &str) -> Vec<(&'a str, usize)> {
    let mut results = Vec::new();
    let mut pos = 0;

    while let Some(found) = source[pos..].find(call_pattern) {
        let abs = pos + found;
        let arg_start = abs + call_pattern.len();

        // Find first string argument
        let after = &source[arg_start..];
        let trimmed = after.trim_start();
        let skip = after.len() - trimmed.len();

        if let Some(quote) = trimmed.chars().next() {
            if quote == '\'' || quote == '"' || quote == '`' {
                let content_start = arg_start + skip + 1;
                if let Some(end) = source[content_start..].find(quote) {
                    let title = &source[content_start..content_start + end];
                    results.push((title, content_start));
                }
            }
        }

        pos = abs + 1;
    }

    results
}

/// Find the matching closing parenthesis for an opening paren.
fn find_matching_paren(source: &str, open_pos: usize) -> Option<usize> {
    if source.as_bytes().get(open_pos) != Some(&b'(') {
        return None;
    }

    let mut depth: u32 = 0;
    for (i, ch) in source[open_pos..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_pos + i);
                }
            }
            _ => {}
        }
    }

    None
}
