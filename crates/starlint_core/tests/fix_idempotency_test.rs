//! Fix idempotency tests.
//!
//! Verifies that running lint+fix twice produces no changes on the second pass.
//! For every rule that emits fixes, we lint source → apply fixes → re-lint →
//! assert zero fixable diagnostics remain.

use std::path::Path;

use starlint_core::diagnostic::OutputFormat;
use starlint_core::engine::LintSession;
use starlint_core::fix::apply_fixes;
use starlint_core::rule::NativeRule;
use starlint_core::rules;

/// Maximum fix passes before giving up (matches CLI constant).
const MAX_FIX_PASSES: usize = 10;

/// Lint source, apply fixes in a convergence loop, and assert the result is
/// stable (no fixable diagnostics remain after convergence).
///
/// The multi-pass loop mirrors the CLI's `apply_fixes_to_files`: overlapping
/// fixes that get skipped on one pass are picked up on the next.
fn assert_fix_idempotent(rules: Vec<Box<dyn NativeRule>>, source: &str, label: &str) {
    let session = LintSession::new(rules, OutputFormat::Pretty);
    let file = Path::new("test.js");

    // First pass: lint and collect diagnostics.
    let result = session.lint_single_file(file, source);
    let fixable_count = result.diagnostics.iter().filter(|d| d.fix.is_some()).count();
    assert!(
        fixable_count > 0,
        "{label}: source should trigger at least one fixable diagnostic, got 0"
    );

    // Multi-pass convergence loop: apply fixes, re-lint, repeat.
    let mut current = source.to_owned();
    let mut diagnostics = result.diagnostics;

    for pass in 0..MAX_FIX_PASSES {
        let fixed = apply_fixes(&current, &diagnostics);
        if fixed == current {
            break;
        }
        current = fixed;

        let relint = session.lint_single_file(file, &current);
        let fixable: Vec<_> = relint
            .diagnostics
            .iter()
            .filter(|d| d.fix.is_some())
            .collect();

        if fixable.is_empty() {
            break;
        }

        assert!(
            pass < MAX_FIX_PASSES - 1,
            "{label}: fixes did not converge after {MAX_FIX_PASSES} passes, \
             still have {} fixable diagnostics from: {:?}",
            fixable.len(),
            fixable
                .iter()
                .map(|d| &d.rule_name)
                .collect::<Vec<_>>()
        );

        diagnostics = relint.diagnostics;
    }

    assert_ne!(
        current, source,
        "{label}: fixes should have modified the source"
    );

    // Final verification: one more lint should produce zero fixable diagnostics.
    let final_result = session.lint_single_file(file, &current);
    let final_fixable: Vec<_> = final_result
        .diagnostics
        .iter()
        .filter(|d| d.fix.is_some())
        .collect();
    assert!(
        final_fixable.is_empty(),
        "{label}: after convergence, should have zero fixable diagnostics, found {} from: {:?}",
        final_fixable.len(),
        final_fixable
            .iter()
            .map(|d| &d.rule_name)
            .collect::<Vec<_>>()
    );

    // apply_fixes on converged source should be a no-op.
    let noop = apply_fixes(&current, &final_result.diagnostics);
    assert_eq!(
        noop, current,
        "{label}: applying fixes to converged source should be a no-op"
    );
}

// ---------------------------------------------------------------------------
// Per-rule idempotency tests (SafeFix rules)
// ---------------------------------------------------------------------------

#[test]
fn fix_idempotent_no_debugger() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_debugger::NoDebugger)],
        "debugger;\nconst x = 1;\ndebugger;",
        "no_debugger",
    );
}

#[test]
fn fix_idempotent_empty_brace_spaces() {
    assert_fix_idempotent(
        vec![Box::new(rules::empty_brace_spaces::EmptyBraceSpaces)],
        "const a = { };\nconst b = {   };",
        "empty_brace_spaces",
    );
}

#[test]
fn fix_idempotent_no_console_spaces() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_console_spaces::NoConsoleSpaces)],
        "console.log(' hello');\nconsole.warn('world ');",
        "no_console_spaces",
    );
}

#[test]
fn fix_idempotent_no_extra_semi() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_extra_semi::NoExtraSemi)],
        "const x = 1;;\nconst y = 2;;",
        "no_extra_semi",
    );
}

#[test]
fn fix_idempotent_no_zero_fractions() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_zero_fractions::NoZeroFractions)],
        "const x = 1.0;\nconst y = 2.0;",
        "no_zero_fractions",
    );
}

#[test]
fn fix_idempotent_number_literal_case() {
    assert_fix_idempotent(
        vec![Box::new(rules::number_literal_case::NumberLiteralCase)],
        "const x = 0XFF;\nconst y = 0B1010;",
        "number_literal_case",
    );
}

#[test]
fn fix_idempotent_no_useless_rename() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_useless_rename::NoUselessRename)],
        "import { foo as foo } from 'bar';",
        "no_useless_rename",
    );
}

#[test]
fn fix_idempotent_no_useless_catch() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_useless_catch::NoUselessCatch)],
        "try { doSomething(); } catch (e) { throw e; }",
        "no_useless_catch",
    );
}

#[test]
fn fix_idempotent_prefer_optional_catch_binding() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_optional_catch_binding::PreferOptionalCatchBinding,
        )],
        "try { doSomething(); } catch (unused) { console.log('failed'); }",
        "prefer_optional_catch_binding",
    );
}

#[test]
fn fix_idempotent_prefer_string_trim_start_end() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_string_trim_start_end::PreferStringTrimStartEnd,
        )],
        "const a = s.trimLeft();\nconst b = s.trimRight();",
        "prefer_string_trim_start_end",
    );
}

#[test]
fn fix_idempotent_no_empty() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_empty::NoEmpty)],
        "try { doSomething(); } catch (e) {}",
        "no_empty",
    );
}

#[test]
fn fix_idempotent_no_console() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_console::NoConsole)],
        "console.log('debug');\nconsole.warn('warning');",
        "no_console",
    );
}

#[test]
fn fix_idempotent_prefer_number_properties() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_number_properties::PreferNumberProperties,
        )],
        "const a = isNaN(x);\nconst b = isFinite(y);",
        "prefer_number_properties",
    );
}

#[test]
fn fix_idempotent_prefer_includes() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_includes::PreferIncludes)],
        "const found = arr.indexOf(x) !== -1;",
        "prefer_includes",
    );
}

#[test]
fn fix_idempotent_throw_new_error() {
    assert_fix_idempotent(
        vec![Box::new(rules::throw_new_error::ThrowNewError)],
        "throw Error('oops');",
        "throw_new_error",
    );
}

#[test]
fn fix_idempotent_new_for_builtins() {
    assert_fix_idempotent(
        vec![Box::new(rules::new_for_builtins::NewForBuiltins)],
        "const m = Map();\nconst s = Set();",
        "new_for_builtins",
    );
}

// ---------------------------------------------------------------------------
// Per-rule idempotency tests (SuggestionFix rules)
// ---------------------------------------------------------------------------

#[test]
fn fix_idempotent_eqeqeq() {
    assert_fix_idempotent(
        vec![Box::new(rules::eqeqeq::Eqeqeq)],
        "if (a == b && c != d) {}",
        "eqeqeq",
    );
}

#[test]
fn fix_idempotent_no_var() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_var::NoVar)],
        "var x = 1;\nvar y = 2;",
        "no_var",
    );
}

#[test]
fn fix_idempotent_no_typeof_undefined() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_typeof_undefined::NoTypeofUndefined)],
        "if (typeof x === 'undefined') {}",
        "no_typeof_undefined",
    );
}

#[test]
fn fix_idempotent_no_unneeded_ternary() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_unneeded_ternary::NoUnneededTernary)],
        "const a = x ? true : false;",
        "no_unneeded_ternary",
    );
}

#[test]
fn fix_idempotent_no_lonely_if() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_lonely_if::NoLonelyIf)],
        "if (a) { foo(); } else { if (b) { bar(); } }",
        "no_lonely_if",
    );
}

// ---------------------------------------------------------------------------
// Phase 1: Trivial Token/Keyword Replacement
// ---------------------------------------------------------------------------

#[test]
fn fix_idempotent_prefer_const() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_const::PreferConst)],
        "let x = 1;\nlet y = 2;",
        "prefer_const",
    );
}

#[test]
fn fix_idempotent_text_encoding_identifier_case() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::text_encoding_identifier_case::TextEncodingIdentifierCase,
        )],
        "const enc = 'UTF-8';",
        "text_encoding_identifier_case",
    );
}

#[test]
fn fix_idempotent_unicode_bom() {
    assert_fix_idempotent(
        vec![Box::new(rules::unicode_bom::UnicodeBom)],
        "\u{FEFF}const x = 1;",
        "unicode_bom",
    );
}

#[test]
fn fix_idempotent_prefer_code_point() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_code_point::PreferCodePoint)],
        "const x = str.charCodeAt(0);\nconst y = String.fromCharCode(65);",
        "prefer_code_point",
    );
}

#[test]
fn fix_idempotent_prefer_string_slice() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_string_slice::PreferStringSlice)],
        "const a = s.substr(1, 3);\nconst b = s.substring(1, 3);",
        "prefer_string_slice",
    );
}

#[test]
fn fix_idempotent_no_useless_return() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_useless_return::NoUselessReturn)],
        "function foo() { doSomething(); return; }",
        "no_useless_return",
    );
}

#[test]
fn fix_idempotent_no_unused_labels() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_unused_labels::NoUnusedLabels)],
        "A: var foo = 0;",
        "no_unused_labels",
    );
}

#[test]
fn fix_idempotent_no_extra_label() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_extra_label::NoExtraLabel)],
        "loop1: while (true) { break; }",
        "no_extra_label",
    );
}

// ---------------------------------------------------------------------------
// Phase 2: Simple Deletion & Insertion
// ---------------------------------------------------------------------------

#[test]
fn fix_idempotent_no_irregular_whitespace() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::no_irregular_whitespace::NoIrregularWhitespace,
        )],
        "var\u{00A0}x = 1;",
        "no_irregular_whitespace",
    );
}

#[test]
fn fix_idempotent_missing_throw() {
    assert_fix_idempotent(
        vec![Box::new(rules::missing_throw::MissingThrow)],
        "new Error('oops');",
        "missing_throw",
    );
}

#[test]
fn fix_idempotent_no_instanceof_array() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_instanceof_array::NoInstanceofArray)],
        "if (x instanceof Array) {}",
        "no_instanceof_array",
    );
}

#[test]
fn fix_idempotent_no_extra_boolean_cast() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_extra_boolean_cast::NoExtraBooleanCast)],
        "if (!!x) {}",
        "no_extra_boolean_cast",
    );
}

#[test]
fn fix_idempotent_no_extra_bind() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_extra_bind::NoExtraBind)],
        "var f = (() => {}).bind(this);",
        "no_extra_bind",
    );
}

#[test]
fn fix_idempotent_escape_case() {
    assert_fix_idempotent(
        vec![Box::new(rules::escape_case::EscapeCase)],
        r"var s = '\xff';",
        "escape_case",
    );
}

#[test]
fn fix_idempotent_no_hex_escape() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_hex_escape::NoHexEscape)],
        r"var s = '\x41';",
        "no_hex_escape",
    );
}

#[test]
fn fix_idempotent_no_useless_computed_key() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::no_useless_computed_key::NoUselessComputedKey,
        )],
        "var obj = { [\"foo\"]: 1 };",
        "no_useless_computed_key",
    );
}

#[test]
fn fix_idempotent_no_useless_escape() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_useless_escape::NoUselessEscape)],
        r#"var x = "hell\o";"#,
        "no_useless_escape",
    );
}

#[test]
fn fix_idempotent_no_useless_concat() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_useless_concat::NoUselessConcat)],
        "var x = 'a' + 'b';",
        "no_useless_concat",
    );
}

// ---------------------------------------------------------------------------
// Phase 3A: Operator/Expression Rules
// ---------------------------------------------------------------------------

#[test]
fn fix_idempotent_bad_bitwise_operator() {
    assert_fix_idempotent(
        vec![Box::new(rules::bad_bitwise_operator::BadBitwiseOperator)],
        "if (a > 1 | b > 2) {}",
        "bad_bitwise_operator",
    );
}

#[test]
fn fix_idempotent_double_comparisons() {
    assert_fix_idempotent(
        vec![Box::new(rules::double_comparisons::DoubleComparisons)],
        "if (a >= b && a <= b) {}",
        "double_comparisons",
    );
}

#[test]
fn fix_idempotent_misrefactored_assign_op() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::misrefactored_assign_op::MisrefactoredAssignOp,
        )],
        "a -= a - b;",
        "misrefactored_assign_op",
    );
}

#[test]
fn fix_idempotent_operator_assignment() {
    assert_fix_idempotent(
        vec![Box::new(rules::operator_assignment::OperatorAssignment)],
        "x = x + 1;",
        "operator_assignment",
    );
}

#[test]
fn fix_idempotent_yoda() {
    assert_fix_idempotent(
        vec![Box::new(rules::yoda::Yoda)],
        "if ('red' === color) {}",
        "yoda",
    );
}

#[test]
fn fix_idempotent_no_eq_null() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_eq_null::NoEqNull)],
        "if (x == null) {}",
        "no_eq_null",
    );
}

#[test]
fn fix_idempotent_no_null() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_null::NoNull)],
        "var z = null;",
        "no_null",
    );
}

#[test]
fn fix_idempotent_no_object_constructor() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::no_object_constructor::NoObjectConstructor,
        )],
        "var obj = new Object();",
        "no_object_constructor",
    );
}

#[test]
fn fix_idempotent_no_array_constructor() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_array_constructor::NoArrayConstructor)],
        "var arr = new Array(1, 2, 3);",
        "no_array_constructor",
    );
}

#[test]
fn fix_idempotent_no_new_array() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_new_array::NoNewArray)],
        "var a = new Array(10);",
        "no_new_array",
    );
}

// ---------------------------------------------------------------------------
// Phase 3B: Insertion/Substitution Rules
// ---------------------------------------------------------------------------

#[test]
fn fix_idempotent_radix() {
    assert_fix_idempotent(
        vec![Box::new(rules::radix::Radix)],
        "var n = parseInt('071');",
        "radix",
    );
}

#[test]
fn fix_idempotent_require_array_join_separator() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::require_array_join_separator::RequireArrayJoinSeparator,
        )],
        "[1, 2, 3].join();",
        "require_array_join_separator",
    );
}

#[test]
fn fix_idempotent_prefer_date_now() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_date_now::PreferDateNow)],
        "const t = new Date().getTime();",
        "prefer_date_now",
    );
}

#[test]
fn fix_idempotent_prefer_exponentiation_operator() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_exponentiation_operator::PreferExponentiationOperator,
        )],
        "var x = Math.pow(2, 3);",
        "prefer_exponentiation_operator",
    );
}

#[test]
fn fix_idempotent_prefer_math_trunc() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_math_trunc::PreferMathTrunc)],
        "const n = ~~x;",
        "prefer_math_trunc",
    );
}

#[test]
fn fix_idempotent_no_implicit_coercion() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_implicit_coercion::NoImplicitCoercion)],
        "var b = !!x;",
        "no_implicit_coercion",
    );
}

#[test]
fn fix_idempotent_numeric_separators_style() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::numeric_separators_style::NumericSeparatorsStyle,
        )],
        "const x = 10000;",
        "numeric_separators_style",
    );
}

#[test]
fn fix_idempotent_prefer_string_raw() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_string_raw::PreferStringRaw)],
        r"var x = `foo\nbar`;",
        "prefer_string_raw",
    );
}

// ---------------------------------------------------------------------------
// Phase 3C: Medium-complexity Rules
// ---------------------------------------------------------------------------

#[test]
fn fix_idempotent_no_regex_spaces() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_regex_spaces::NoRegexSpaces)],
        "var re = /foo  bar/;",
        "no_regex_spaces",
    );
}

#[test]
fn fix_idempotent_prefer_numeric_literals() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_numeric_literals::PreferNumericLiterals,
        )],
        "parseInt('1A', 16);",
        "prefer_numeric_literals",
    );
}

#[test]
fn fix_idempotent_prefer_object_has_own() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_object_has_own::PreferObjectHasOwn,
        )],
        "Object.prototype.hasOwnProperty.call(obj, 'key');",
        "prefer_object_has_own",
    );
}

#[test]
fn fix_idempotent_no_div_regex() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_div_regex::NoDivRegex)],
        "var r = /=foo/;",
        "no_div_regex",
    );
}

#[test]
fn fix_idempotent_prefer_object_spread() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_object_spread::PreferObjectSpread)],
        "var x = Object.assign({}, foo);",
        "prefer_object_spread",
    );
}

#[test]
fn fix_idempotent_prefer_reflect_apply() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_reflect_apply::PreferReflectApply,
        )],
        "foo.apply(null, args);",
        "prefer_reflect_apply",
    );
}

#[test]
fn fix_idempotent_no_new_buffer() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_new_buffer::NoNewBuffer)],
        "var b = new Buffer(10);",
        "no_new_buffer",
    );
}

// ---------------------------------------------------------------------------
// Combined multi-rule tests
// ---------------------------------------------------------------------------

#[test]
fn fix_idempotent_combined_multi_rule() {
    assert_fix_idempotent(
        vec![
            Box::new(rules::no_debugger::NoDebugger),
            Box::new(rules::empty_brace_spaces::EmptyBraceSpaces),
            Box::new(rules::no_extra_semi::NoExtraSemi),
            Box::new(rules::eqeqeq::Eqeqeq),
            Box::new(rules::no_var::NoVar),
            Box::new(rules::no_zero_fractions::NoZeroFractions),
        ],
        "\
debugger;
const a = { };
var y = 1.0;;
if (a == b) {}
",
        "combined_multi_rule",
    );
}

// ── Phase 3D: High-complexity rules ──────────────────────────────────────

#[test]
fn fix_idempotent_no_else_return() {
    assert_fix_idempotent(
        vec![Box::new(rules::no_else_return::NoElseReturn)],
        "function f(x) { if (x) { return 1; } else { return 2; } }",
        "no_else_return",
    );
}

#[test]
fn fix_idempotent_prefer_native_coercion_functions() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_native_coercion_functions::PreferNativeCoercionFunctions,
        )],
        "arr.map(x => Number(x));",
        "prefer_native_coercion_functions",
    );
}

#[test]
fn fix_idempotent_prefer_prototype_methods() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_prototype_methods::PreferPrototypeMethods)],
        "[].forEach.call(obj, fn);",
        "prefer_prototype_methods",
    );
}

#[test]
fn fix_idempotent_prefer_string_starts_ends_with() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_string_starts_ends_with::PreferStringStartsEndsWith,
        )],
        "if (str.indexOf('foo') === 0) {}",
        "prefer_string_starts_ends_with_indexof",
    );
}

#[test]
fn fix_idempotent_prefer_string_starts_ends_with_regex() {
    assert_fix_idempotent(
        vec![Box::new(
            rules::prefer_string_starts_ends_with::PreferStringStartsEndsWith,
        )],
        "if (/^foo/.test(str)) {}",
        "prefer_string_starts_ends_with_regex",
    );
}

#[test]
fn fix_idempotent_prefer_template() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_template::PreferTemplate)],
        "var x = 'hello ' + name;",
        "prefer_template",
    );
}

#[test]
fn fix_idempotent_prefer_ternary_return() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_ternary::PreferTernary)],
        "function f(x) { if (x) { return a; } else { return b; } }",
        "prefer_ternary_return",
    );
}

#[test]
fn fix_idempotent_prefer_ternary_assign() {
    assert_fix_idempotent(
        vec![Box::new(rules::prefer_ternary::PreferTernary)],
        "var a; if (x) { a = 1; } else { a = 2; }",
        "prefer_ternary_assign",
    );
}

#[test]
fn fix_idempotent_all_rules() {
    // Includes `console.log(' hello')` which triggers both no-console and
    // no-console-spaces with overlapping spans — the multi-pass convergence
    // loop handles this by picking up the skipped fix on the next pass.
    assert_fix_idempotent(
        rules::all_rules(),
        "\
debugger;
const a = { };
console.log(' hello');
var y = 1.0;;
if (a == b) {}
const m = Map();
throw Error('oops');
const x = 0XFF;
const enc = 'UTF-8';
const cp = str.charCodeAt(0);
const sub = str.substr(1);
loop1: while (true) { break; }
if (a > 1 | b > 2) {}
if (a >= b && a <= b) {}
n -= n - m;
q = q + 1;
if ('red' === color) {}
var obj = new Object();
var arr = new Array(1, 2, 3);
var pn = parseInt('071');
const ts = new Date().getTime();
var pw = Math.pow(2, 3);
const big = 10000;
arr.map(x => Number(x));
[].forEach.call(obj, fn);
var greeting = 'hi ' + user;
",
        "all_rules",
    );
}
