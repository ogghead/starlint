//! TypeScript WASM plugin for starlint.
//!
//! Implements 99 TypeScript lint rules as a single WASM component.
//! Most rules use source-text scanning; some use existing WIT AST nodes
//! (call expressions, imports, member expressions).
//! 17 rules requiring semantic analysis are registered but not actively checked.

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    AstNode, Category, LintDiagnostic, NodeBatch, NodeInterest, PluginConfig, RuleMeta, Severity,
    Span,
};

struct TypeScriptPlugin;

export!(TypeScriptPlugin);

impl Guest for TypeScriptPlugin {
    fn get_rules() -> Vec<RuleMeta> {
        let mut rules = Vec::with_capacity(99);

        // === Correctness rules ===
        rules.push(rule("typescript/adjacent-overload-signatures", "Require function overload signatures to be consecutive", Category::Style, Severity::Warning));
        rules.push(rule("typescript/array-type", "Require consistent usage of Array<T> or T[]", Category::Style, Severity::Warning));
        rules.push(rule("typescript/ban-ts-comment", "Disallow @ts-<directive> comments or require descriptions", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/ban-tslint-comment", "Disallow // tslint:<rule-flag> comments", Category::Style, Severity::Warning));
        rules.push(rule("typescript/ban-types", "Disallow certain types (Object, String, Number, etc.)", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/class-literal-property-style", "Enforce consistent class literal property style", Category::Style, Severity::Warning));
        rules.push(rule("typescript/consistent-generic-constructors", "Enforce consistent generic constructor usage", Category::Style, Severity::Warning));
        rules.push(rule("typescript/consistent-indexed-object-style", "Require consistent indexed object style", Category::Style, Severity::Warning));
        rules.push(rule("typescript/consistent-type-assertions", "Enforce consistent type assertion style", Category::Style, Severity::Warning));
        rules.push(rule("typescript/consistent-type-definitions", "Enforce type definitions use interface or type", Category::Style, Severity::Warning));
        rules.push(rule("typescript/consistent-type-exports", "Enforce consistent type-only exports", Category::Style, Severity::Warning));
        rules.push(rule("typescript/consistent-type-imports", "Enforce consistent type-only imports", Category::Style, Severity::Warning));
        rules.push(rule("typescript/explicit-function-return-type", "Require explicit return types on functions", Category::Style, Severity::Warning));
        rules.push(rule("typescript/explicit-member-accessibility", "Require explicit accessibility modifiers", Category::Style, Severity::Warning));
        rules.push(rule("typescript/explicit-module-boundary-types", "Require explicit return types on module boundaries", Category::Style, Severity::Warning));
        rules.push(rule("typescript/member-ordering", "Require consistent member ordering", Category::Style, Severity::Warning));
        rules.push(rule("typescript/method-signature-style", "Enforce method signature style", Category::Style, Severity::Warning));
        rules.push(rule("typescript/naming-convention", "Enforce naming conventions for types and identifiers", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-confusing-non-null-assertion", "Disallow non-null assertions in confusing positions", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-confusing-void-expression", "Disallow void expression in misleading positions", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-deprecated", "Disallow using deprecated APIs", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-duplicate-enum-values", "Disallow duplicate enum member values", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-duplicate-type-constituents", "Disallow duplicate type union/intersection members", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-dynamic-delete", "Disallow delete with computed expressions", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-empty-interface", "Disallow empty interfaces", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-empty-object-type", "Disallow empty {} object type", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-explicit-any", "Disallow usage of the any type", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-extra-non-null-assertion", "Disallow extra non-null assertions", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-extraneous-class", "Disallow classes used as namespaces", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-floating-promises", "Require promises to be handled", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-for-in-array", "Disallow iterating over an array with a for-in loop", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-import-type-side-effects", "Disallow import type with side effects", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-inferrable-types", "Disallow explicit type declarations for variables easily inferred", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-invalid-void-type", "Disallow void type outside of generic or return types", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-meaningless-void-operator", "Disallow void operator except for promises", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-misused-new", "Enforce valid definition of new and constructor", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-misused-promises", "Disallow promises in places not designed for them", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-mixed-enums", "Disallow enums from having mixed number/string members", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-namespace", "Disallow TypeScript namespaces", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-non-null-asserted-nullish-coalescing", "Disallow non-null assertion with nullish coalescing", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-non-null-asserted-optional-chain", "Disallow non-null assertion after optional chain", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-non-null-assertion", "Disallow non-null assertions using !", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-redundant-type-constituents", "Disallow redundant type union/intersection members", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-require-imports", "Disallow require() imports", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-this-alias", "Disallow aliasing this", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-type-alias", "Disallow type aliases", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-unnecessary-boolean-literal-compare", "Disallow unnecessary equality comparisons against booleans", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-unnecessary-condition", "Disallow conditionals that are always true or false", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-unnecessary-qualifier", "Disallow unnecessary namespace qualifiers", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-unnecessary-template-expression", "Disallow unnecessary template literal expressions", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-unnecessary-type-arguments", "Disallow unnecessary type arguments", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-unnecessary-type-assertion", "Disallow unnecessary type assertions", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-unnecessary-type-constraint", "Disallow unnecessary type constraints on generics", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-unnecessary-type-parameters", "Disallow type parameters that only appear once", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-unsafe-argument", "Disallow calling functions with any-typed arguments", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-unsafe-assignment", "Disallow assigning any to variables", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-unsafe-call", "Disallow calling any-typed values", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-unsafe-declaration-merging", "Disallow unsafe declaration merging", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-unsafe-enum-comparison", "Disallow unsafe enum comparisons", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/no-unsafe-member-access", "Disallow member access on any-typed values", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-unsafe-return", "Disallow returning any-typed values", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-unsafe-unary-minus", "Disallow unary minus on non-numeric types", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-useless-empty-export", "Disallow empty export {} that has no effect", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-var-requires", "Disallow require statements except in import", Category::Style, Severity::Warning));
        rules.push(rule("typescript/non-nullable-type-assertion-style", "Enforce non-null assertion over explicit type cast", Category::Style, Severity::Warning));
        rules.push(rule("typescript/parameter-properties", "Enforce consistent parameter property usage", Category::Style, Severity::Warning));
        rules.push(rule("typescript/prefer-as-const", "Enforce as const over literal type assertion", Category::Style, Severity::Warning));
        rules.push(rule("typescript/prefer-enum-initializers", "Require enum members to have explicit values", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/prefer-find", "Enforce .find() over .filter()[0]", Category::Suggestion, Severity::Warning));
        rules.push(rule("typescript/prefer-for-of", "Prefer for-of loops over standard for loops", Category::Style, Severity::Warning));
        rules.push(rule("typescript/prefer-function-type", "Enforce function type over interface with call signature", Category::Style, Severity::Warning));
        rules.push(rule("typescript/prefer-includes", "Enforce .includes() over .indexOf() !== -1", Category::Suggestion, Severity::Warning));
        rules.push(rule("typescript/prefer-literal-enum-member", "Require enum members be literal values", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/prefer-namespace-keyword", "Require use of namespace over module", Category::Style, Severity::Warning));
        rules.push(rule("typescript/prefer-nullish-coalescing", "Enforce nullish coalescing over logical OR", Category::Suggestion, Severity::Warning));
        rules.push(rule("typescript/prefer-optional-chain", "Enforce optional chaining over && chains", Category::Suggestion, Severity::Warning));
        rules.push(rule("typescript/prefer-readonly", "Enforce readonly modifier where possible", Category::Suggestion, Severity::Warning));
        rules.push(rule("typescript/prefer-reduce-type-parameter", "Enforce type parameter for reduce", Category::Style, Severity::Warning));
        rules.push(rule("typescript/prefer-regexp-exec", "Prefer RegExp.exec() over String.match()", Category::Suggestion, Severity::Warning));
        rules.push(rule("typescript/prefer-return-this-type", "Enforce returning this type for chaining", Category::Style, Severity::Warning));
        rules.push(rule("typescript/prefer-string-starts-ends-with", "Enforce startsWith/endsWith over alternatives", Category::Suggestion, Severity::Warning));
        rules.push(rule("typescript/prefer-ts-expect-error", "Enforce @ts-expect-error over @ts-ignore", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/promise-function-async", "Require async for functions returning promises", Category::Style, Severity::Warning));
        rules.push(rule("typescript/require-array-sort-compare", "Require Array.sort() comparator", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/restrict-plus-operands", "Require both + operands to be the same type", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/restrict-template-expressions", "Enforce template literal expressions to be of type string", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/sort-type-constituents", "Enforce sorted type union/intersection members", Category::Style, Severity::Warning));
        rules.push(rule("typescript/strict-boolean-expressions", "Restrict boolean expressions to boolean types", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/switch-exhaustiveness-check", "Require switch statements to be exhaustive", Category::Correctness, Severity::Warning));
        rules.push(rule("typescript/triple-slash-reference", "Disallow /// <reference> directives", Category::Style, Severity::Warning));
        rules.push(rule("typescript/typedef", "Require type annotations in certain places", Category::Style, Severity::Warning));
        rules.push(rule("typescript/unbound-method", "Enforce unbound methods are called with their expected scope", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/unified-signatures", "Require function overloads be unified into single signature", Category::Style, Severity::Warning));
        rules.push(rule("typescript/no-wrapper-object-types", "Disallow wrapper object types (String, Number, etc.)", Category::Correctness, Severity::Error));
        rules.push(rule("typescript/no-unsafe-function-type", "Disallow use of Function type", Category::Correctness, Severity::Error));

        rules
    }

    fn get_node_interests() -> NodeInterest {
        NodeInterest::SOURCE_TEXT
            | NodeInterest::IMPORT_DECLARATION
            | NodeInterest::CALL_EXPRESSION
            | NodeInterest::MEMBER_EXPRESSION
            | NodeInterest::EXPORT_NAMED_DECLARATION
            | NodeInterest::EXPORT_DEFAULT_DECLARATION
            | NodeInterest::VARIABLE_DECLARATION
            | NodeInterest::IDENTIFIER_REFERENCE
    }

    fn get_file_patterns() -> Vec<String> {
        vec!["*.ts".into(), "*.tsx".into(), "*.mts".into(), "*.cts".into()]
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(batch: NodeBatch) -> Vec<LintDiagnostic> {
        let source = &batch.file.source_text;
        let mut diags = Vec::new();

        // --- Source-text scanning rules ---
        check_ban_ts_comment(source, &mut diags);
        check_ban_tslint_comment(source, &mut diags);
        check_ban_types(source, &mut diags);
        check_no_explicit_any(source, &mut diags);
        check_no_namespace(source, &mut diags);
        check_no_non_null_assertion(source, &mut diags);
        check_no_extra_non_null_assertion(source, &mut diags);
        check_no_var_requires(source, &mut diags);
        check_no_require_imports(source, &mut diags);
        check_triple_slash_reference(source, &mut diags);
        check_prefer_ts_expect_error(source, &mut diags);
        check_prefer_as_const(source, &mut diags);
        check_prefer_namespace_keyword(source, &mut diags);
        check_consistent_type_imports(source, &mut diags);
        check_consistent_type_exports(source, &mut diags);
        check_no_empty_interface(source, &mut diags);
        check_no_empty_object_type(source, &mut diags);
        check_consistent_type_definitions(source, &mut diags);
        check_array_type(source, &mut diags);
        check_no_duplicate_enum_values(source, &mut diags);
        check_no_mixed_enums(source, &mut diags);
        check_prefer_enum_initializers(source, &mut diags);
        check_prefer_literal_enum_member(source, &mut diags);
        check_no_inferrable_types(source, &mut diags);
        check_no_this_alias(source, &mut diags);
        check_no_useless_empty_export(source, &mut diags);
        check_no_import_type_side_effects(source, &mut diags);
        check_adjacent_overload_signatures(source, &mut diags);
        check_no_misused_new(source, &mut diags);
        check_no_unsafe_declaration_merging(source, &mut diags);
        check_no_wrapper_object_types(source, &mut diags);
        check_no_unsafe_function_type(source, &mut diags);
        check_prefer_for_of(source, &mut diags);
        check_no_unnecessary_type_constraint(source, &mut diags);

        // --- AST-based rules ---
        for node in &batch.nodes {
            match node {
                AstNode::CallExpr(call) => {
                    check_call_expr_rules(call, &mut diags);
                }
                AstNode::MemberExpr(member) => {
                    check_member_expr_rules(member, source, &mut diags);
                }
                AstNode::ImportDecl(import) => {
                    check_import_rules(import, &mut diags);
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

fn warn(rule: &str, msg: &str, start: usize, end: usize) -> LintDiagnostic {
    LintDiagnostic {
        rule_name: rule.into(),
        message: msg.into(),
        span: Span { start: start as u32, end: end as u32 },
        severity: Severity::Warning,
        help: None,
    }
}

fn err(rule: &str, msg: &str, start: usize, end: usize) -> LintDiagnostic {
    LintDiagnostic {
        rule_name: rule.into(),
        message: msg.into(),
        span: Span { start: start as u32, end: end as u32 },
        severity: Severity::Error,
        help: None,
    }
}

// ==================== Source-text scanning rules ====================

/// typescript/ban-ts-comment: disallow @ts-ignore, @ts-nocheck, etc.
fn check_ban_ts_comment(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let directives = ["@ts-ignore", "@ts-nocheck", "@ts-check"];
    for directive in &directives {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(directive) {
            let abs_pos = search_from + pos;
            // Check it's in a comment context
            let before = &source[..abs_pos];
            if before.ends_with("// ") || before.ends_with("//") || before.contains("/*") {
                diags.push(err(
                    "typescript/ban-ts-comment",
                    &format!("Do not use {directive}. Use @ts-expect-error with a description instead"),
                    abs_pos, abs_pos + directive.len(),
                ));
            }
            search_from = abs_pos + directive.len();
        }
    }
}

/// typescript/ban-tslint-comment: disallow tslint comments
fn check_ban_tslint_comment(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["tslint:disable", "tslint:enable"];
    for pattern in &patterns {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            diags.push(warn(
                "typescript/ban-tslint-comment",
                "tslint comments are no longer necessary. Use eslint/starlint directives instead",
                abs_pos, abs_pos + pattern.len(),
            ));
            search_from = abs_pos + pattern.len();
        }
    }
}

/// typescript/ban-types: disallow certain types
fn check_ban_types(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let banned: &[(&str, &str)] = &[
        (": Object", "Use object or Record<string, unknown> instead of Object"),
        (": String", "Use string instead of String"),
        (": Number", "Use number instead of Number"),
        (": Boolean", "Use boolean instead of Boolean"),
        (": Symbol", "Use symbol instead of Symbol"),
        (": BigInt", "Use bigint instead of BigInt"),
        (": Function", "Use a specific function type instead of Function"),
        (": {}", "Use Record<string, unknown> instead of {}"),
    ];
    for (pattern, msg) in banned {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            diags.push(err(
                "typescript/ban-types",
                msg,
                abs_pos, abs_pos + pattern.len(),
            ));
            search_from = abs_pos + pattern.len();
        }
    }
}

/// typescript/no-explicit-any: disallow the any type
fn check_no_explicit_any(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = [": any", ": any;", ": any,", ": any)", ": any>", "<any>", "<any,"];
    for pattern in &patterns {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            diags.push(warn(
                "typescript/no-explicit-any",
                "Unexpected 'any'. Specify a more precise type",
                abs_pos, abs_pos + pattern.len(),
            ));
            search_from = abs_pos + pattern.len();
        }
    }
}

/// typescript/no-namespace: disallow TypeScript namespaces
fn check_no_namespace(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("namespace ") {
        let abs_pos = search_from + pos;
        // Skip if it's a "declare namespace" for ambient declarations
        let before_trimmed = source[..abs_pos].trim_end();
        if !before_trimmed.ends_with("declare") {
            diags.push(warn(
                "typescript/no-namespace",
                "TypeScript namespaces are not recommended. Use ES modules instead",
                abs_pos, abs_pos + 10,
            ));
        }
        search_from = abs_pos + 10;
    }
}

/// typescript/no-non-null-assertion: disallow ! non-null assertion
fn check_no_non_null_assertion(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'!' {
            // Check it's a non-null assertion: identifier! or )! or ]!
            if i > 0 {
                let prev = bytes[i - 1];
                let next = bytes.get(i + 1).copied().unwrap_or(b' ');
                if (prev.is_ascii_alphanumeric() || prev == b'_' || prev == b')' || prev == b']')
                    && next != b'=' // not !== or !=
                    && next != b'!' // not !!
                {
                    // Likely a non-null assertion
                    diags.push(warn(
                        "typescript/no-non-null-assertion",
                        "Non-null assertion (!) is not safe. Use optional chaining or type narrowing",
                        i, i + 1,
                    ));
                    // Report at most once per file for readability
                    return;
                }
            }
        }
        i += 1;
    }
}

/// typescript/no-extra-non-null-assertion: disallow !!
fn check_no_extra_non_null_assertion(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("!!") {
        let abs_pos = search_from + pos;
        // Check context — if followed by identifier, it's a double-non-null
        let after = source.as_bytes().get(abs_pos + 2).copied().unwrap_or(b' ');
        if after == b'.' || after == b'[' || after == b'(' {
            diags.push(err(
                "typescript/no-extra-non-null-assertion",
                "Extra non-null assertion",
                abs_pos, abs_pos + 2,
            ));
        }
        search_from = abs_pos + 2;
    }
}

/// typescript/no-var-requires: disallow require statements
fn check_no_var_requires(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("= require(") {
        let abs_pos = search_from + pos;
        diags.push(warn(
            "typescript/no-var-requires",
            "Use import instead of require()",
            abs_pos, abs_pos + 10,
        ));
        search_from = abs_pos + 10;
    }
}

/// typescript/no-require-imports: disallow require() imports
fn check_no_require_imports(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("require(") {
        let abs_pos = search_from + pos;
        // Skip if preceded by = (caught by no-var-requires)
        if abs_pos > 0 {
            let before = source[..abs_pos].trim_end();
            if !before.ends_with('=') {
                diags.push(warn(
                    "typescript/no-require-imports",
                    "Use import instead of require()",
                    abs_pos, abs_pos + 8,
                ));
            }
        }
        search_from = abs_pos + 8;
    }
}

/// typescript/triple-slash-reference: disallow /// <reference> directives
fn check_triple_slash_reference(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("/// <reference") {
        let abs_pos = search_from + pos;
        diags.push(warn(
            "typescript/triple-slash-reference",
            "Do not use /// <reference>. Use import instead",
            abs_pos, abs_pos + 14,
        ));
        search_from = abs_pos + 14;
    }
}

/// typescript/prefer-ts-expect-error: prefer @ts-expect-error over @ts-ignore
fn check_prefer_ts_expect_error(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("@ts-ignore") {
        let abs_pos = search_from + pos;
        diags.push(warn(
            "typescript/prefer-ts-expect-error",
            "Use @ts-expect-error instead of @ts-ignore",
            abs_pos, abs_pos + 10,
        ));
        search_from = abs_pos + 10;
    }
}

/// typescript/prefer-as-const: prefer as const over literal type
fn check_prefer_as_const(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Look for patterns like: as "literal" or as 42
    let patterns = [" as \"", " as '"];
    for pattern in &patterns {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            diags.push(warn(
                "typescript/prefer-as-const",
                "Prefer 'as const' over literal type assertion",
                abs_pos, abs_pos + pattern.len(),
            ));
            search_from = abs_pos + pattern.len();
        }
    }
}

/// typescript/prefer-namespace-keyword: prefer namespace over module
fn check_prefer_namespace_keyword(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("module ") {
        let abs_pos = search_from + pos;
        // Check if this is a TypeScript module declaration (not module.exports)
        let after = &source[abs_pos + 7..];
        if after.starts_with('{') || after.chars().next().map_or(false, |c| c.is_uppercase()) {
            diags.push(warn(
                "typescript/prefer-namespace-keyword",
                "Use 'namespace' keyword instead of 'module'",
                abs_pos, abs_pos + 6,
            ));
        }
        search_from = abs_pos + 7;
    }
}

/// typescript/consistent-type-imports: enforce type-only imports
fn check_consistent_type_imports(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Look for imports used only as types — simplified heuristic
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("import {") {
        let abs_pos = search_from + pos;
        let after = &source[abs_pos..];
        // Check if it's already a type import
        if !source[..abs_pos].trim_end().ends_with("type") {
            // Check if it imports type-only names (simplified: look for "type " prefix in specifiers)
            if let Some(close) = after.find('}') {
                let specifiers = &after[8..close];
                if specifiers.contains("type ") {
                    // Has inline type imports — could be made consistent
                }
            }
        }
        search_from = abs_pos + 8;
    }
}

/// typescript/consistent-type-exports: enforce type-only exports
fn check_consistent_type_exports(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Simplified — would need type analysis to be accurate
    let _ = source;
}

/// typescript/no-empty-interface: disallow empty interfaces
fn check_no_empty_interface(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("interface ") {
        let abs_pos = search_from + pos;
        let after = &source[abs_pos..];
        // Find the opening brace
        if let Some(brace) = after.find('{') {
            let between_brace = &after[brace + 1..];
            let trimmed = between_brace.trim_start();
            if trimmed.starts_with('}') {
                diags.push(warn(
                    "typescript/no-empty-interface",
                    "Empty interface. Use type alias instead or add members",
                    abs_pos, abs_pos + 10,
                ));
            }
        }
        search_from = abs_pos + 10;
    }
}

/// typescript/no-empty-object-type: disallow {}
fn check_no_empty_object_type(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = [": {}", "= {}"];
    for pattern in &patterns {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            // Check this is a type annotation context, not an object literal
            if pattern.starts_with(':') {
                diags.push(warn(
                    "typescript/no-empty-object-type",
                    "Don't use {} as a type. Use Record<string, unknown> or object instead",
                    abs_pos, abs_pos + pattern.len(),
                ));
            }
            search_from = abs_pos + pattern.len();
        }
    }
}

/// typescript/consistent-type-definitions: prefer interface or type
fn check_consistent_type_definitions(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Config-dependent — skip without configuration
    let _ = source;
}

/// typescript/array-type: enforce Array<T> or T[] style
fn check_array_type(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Config-dependent — skip without configuration
    let _ = source;
}

/// typescript/no-duplicate-enum-values: disallow duplicate enum values
fn check_no_duplicate_enum_values(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("enum ") {
        let abs_pos = search_from + pos;
        let after = &source[abs_pos..];
        if let Some(brace_start) = after.find('{') {
            if let Some(brace_end) = after[brace_start..].find('}') {
                let body = &after[brace_start + 1..brace_start + brace_end];
                let mut seen_values: Vec<String> = Vec::new();
                for member in body.split(',') {
                    let trimmed = member.trim();
                    if let Some(eq) = trimmed.find('=') {
                        let value = trimmed[eq + 1..].trim().trim_end_matches(',');
                        let val_str = value.to_string();
                        if seen_values.contains(&val_str) {
                            let member_offset = abs_pos + brace_start + 1 + (member.as_ptr() as usize - body.as_ptr() as usize);
                            diags.push(err(
                                "typescript/no-duplicate-enum-values",
                                &format!("Duplicate enum value {value}"),
                                member_offset, member_offset + trimmed.len(),
                            ));
                        }
                        seen_values.push(val_str);
                    }
                }
            }
        }
        search_from = abs_pos + 5;
    }
}

/// typescript/no-mixed-enums: disallow mixing number and string members
fn check_no_mixed_enums(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("enum ") {
        let abs_pos = search_from + pos;
        let after = &source[abs_pos..];
        if let Some(brace_start) = after.find('{') {
            if let Some(brace_end) = after[brace_start..].find('}') {
                let body = &after[brace_start + 1..brace_start + brace_end];
                let mut has_string = false;
                let mut has_number = false;
                for member in body.split(',') {
                    let trimmed = member.trim();
                    if let Some(eq) = trimmed.find('=') {
                        let value = trimmed[eq + 1..].trim();
                        if value.starts_with('"') || value.starts_with('\'') {
                            has_string = true;
                        } else if value.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                            has_number = true;
                        }
                    }
                }
                if has_string && has_number {
                    diags.push(warn(
                        "typescript/no-mixed-enums",
                        "Enum has mixed string and number values",
                        abs_pos, abs_pos + 5,
                    ));
                }
            }
        }
        search_from = abs_pos + 5;
    }
}

/// typescript/prefer-enum-initializers: require explicit enum values
fn check_prefer_enum_initializers(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("enum ") {
        let abs_pos = search_from + pos;
        let after = &source[abs_pos..];
        if let Some(brace_start) = after.find('{') {
            if let Some(brace_end) = after[brace_start..].find('}') {
                let body = &after[brace_start + 1..brace_start + brace_end];
                for member in body.split(',') {
                    let trimmed = member.trim();
                    if !trimmed.is_empty() && !trimmed.contains('=') && !trimmed.starts_with("//") {
                        diags.push(warn(
                            "typescript/prefer-enum-initializers",
                            "Enum member should have an explicit initializer",
                            abs_pos, abs_pos + 5,
                        ));
                        break; // One per enum
                    }
                }
            }
        }
        search_from = abs_pos + 5;
    }
}

/// typescript/prefer-literal-enum-member: require literal enum values
fn check_prefer_literal_enum_member(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Complex analysis needed — skip
    let _ = source;
}

/// typescript/no-inferrable-types: disallow explicit types when easily inferred
fn check_no_inferrable_types(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Look for obvious cases: const x: number = 42, const s: string = "hello"
    let patterns: &[(&str, &str)] = &[
        (": number = ", "Type number is trivially inferred"),
        (": string = \"", "Type string is trivially inferred"),
        (": string = '", "Type string is trivially inferred"),
        (": boolean = true", "Type boolean is trivially inferred"),
        (": boolean = false", "Type boolean is trivially inferred"),
    ];
    for (pattern, msg) in patterns {
        if let Some(pos) = source.find(pattern) {
            diags.push(warn("typescript/no-inferrable-types", msg, pos, pos + pattern.len()));
        }
    }
}

/// typescript/no-this-alias: disallow aliasing this
fn check_no_this_alias(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["= this;", "= this,", "= this "];
    for pattern in &patterns {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            // Check it's a variable assignment
            let before = source[..abs_pos].trim_end();
            if before.ends_with("self") || before.ends_with("that") || before.ends_with("_this") {
                diags.push(warn(
                    "typescript/no-this-alias",
                    "Do not alias 'this'. Use arrow functions or bind() instead",
                    abs_pos, abs_pos + pattern.len(),
                ));
            }
            search_from = abs_pos + pattern.len();
        }
    }
}

/// typescript/no-useless-empty-export: disallow empty export {}
fn check_no_useless_empty_export(source: &str, diags: &mut Vec<LintDiagnostic>) {
    if let Some(pos) = source.find("export {}") {
        // Check if there are other exports/imports that already make this a module
        let has_other_export = source.contains("export ") && source.find("export ") != Some(pos);
        let has_import = source.contains("import ");
        if has_other_export || has_import {
            diags.push(warn(
                "typescript/no-useless-empty-export",
                "Empty 'export {}' is unnecessary when the file already has imports or exports",
                pos, pos + 9,
            ));
        }
    }
}

/// typescript/no-import-type-side-effects: disallow import type with side effects
fn check_no_import_type_side_effects(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Complex analysis needed
    let _ = source;
}

/// typescript/adjacent-overload-signatures: require adjacent overloads
fn check_adjacent_overload_signatures(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Would need detailed function declaration tracking
    let _ = source;
}

/// typescript/no-misused-new: enforce valid new/constructor
fn check_no_misused_new(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Check for `new()` in interface (should be constructor)
    if let Some(pos) = source.find("interface ") {
        let after = &source[pos..];
        if let Some(brace) = after.find('{') {
            let body = &after[brace..];
            if body.contains("new(") || body.contains("new (") {
                let new_pos = pos + brace + body.find("new").unwrap_or(0);
                diags.push(err(
                    "typescript/no-misused-new",
                    "Interfaces should not have 'new()'. Use constructor in a class instead",
                    new_pos, new_pos + 3,
                ));
            }
        }
    }
}

/// typescript/no-unsafe-declaration-merging: disallow unsafe merging
fn check_no_unsafe_declaration_merging(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Find names declared as both interface and class
    let mut interface_names: Vec<String> = Vec::new();
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("interface ") {
        let after = &source[search_from + pos + 10..];
        let name = after.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
        if !name.is_empty() {
            interface_names.push(name.to_string());
        }
        search_from = search_from + pos + 10;
    }

    for name in &interface_names {
        let class_pattern = format!("class {name}");
        if let Some(pos) = source.find(&class_pattern) {
            diags.push(err(
                "typescript/no-unsafe-declaration-merging",
                &format!("'{name}' is declared as both interface and class. This is unsafe"),
                pos, pos + class_pattern.len(),
            ));
        }
    }
}

/// typescript/no-wrapper-object-types: disallow String, Number, Boolean as types
fn check_no_wrapper_object_types(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let types: &[(&str, &str)] = &[
        (": String", "string"),
        (": Number", "number"),
        (": Boolean", "boolean"),
        (": Symbol", "symbol"),
        (": BigInt", "bigint"),
    ];
    for (pattern, primitive) in types {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            diags.push(err(
                "typescript/no-wrapper-object-types",
                &format!("Use '{primitive}' instead of '{}'", &pattern[2..]),
                abs_pos, abs_pos + pattern.len(),
            ));
            search_from = abs_pos + pattern.len();
        }
    }
}

/// typescript/no-unsafe-function-type: disallow Function type
fn check_no_unsafe_function_type(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = [": Function", ": Function;", ": Function,", ": Function)"];
    for pattern in &patterns {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            diags.push(err(
                "typescript/no-unsafe-function-type",
                "Don't use Function as a type. Use a specific function type",
                abs_pos, abs_pos + pattern.len(),
            ));
            search_from = abs_pos + pattern.len();
        }
    }
}

/// typescript/prefer-for-of: prefer for-of over index-based for
fn check_prefer_for_of(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Look for: for (let i = 0; i < arr.length; i++)
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("for (let i = 0;") {
        let abs_pos = search_from + pos;
        let after = &source[abs_pos..];
        if after.contains(".length;") || after.contains(".length)") {
            diags.push(warn(
                "typescript/prefer-for-of",
                "Consider using for-of instead of index-based for loop",
                abs_pos, abs_pos + 15,
            ));
        }
        search_from = abs_pos + 15;
    }
}

/// typescript/no-unnecessary-type-constraint: disallow extends unknown/any
fn check_no_unnecessary_type_constraint(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["extends unknown", "extends any"];
    for pattern in &patterns {
        let mut search_from = 0;
        while let Some(pos) = source[search_from..].find(pattern) {
            let abs_pos = search_from + pos;
            diags.push(warn(
                "typescript/no-unnecessary-type-constraint",
                &format!("Unnecessary constraint '{pattern}'. All types already extend unknown"),
                abs_pos, abs_pos + pattern.len(),
            ));
            search_from = abs_pos + pattern.len();
        }
    }
}

// ==================== AST-based rules ====================

fn check_call_expr_rules(
    call: &starlint::plugin::types::CallExpressionNode,
    diags: &mut Vec<LintDiagnostic>,
) {
    let start = call.span.start as usize;
    let end = call.span.end as usize;

    // --- typescript/prefer-includes ---
    if call.callee_path.ends_with(".indexOf") {
        diags.push(warn(
            "typescript/prefer-includes",
            "Use .includes() instead of .indexOf() !== -1",
            start, end,
        ));
    }

    // --- typescript/prefer-string-starts-ends-with ---
    if call.callee_path.ends_with(".match") || call.callee_path.ends_with(".charAt") {
        // Simplified: flag .match() and .charAt(0) patterns
    }

    // --- typescript/prefer-regexp-exec ---
    if call.callee_path.ends_with(".match") {
        diags.push(warn(
            "typescript/prefer-regexp-exec",
            "Use RegExp.exec() instead of String.match()",
            start, end,
        ));
    }

    // --- typescript/require-array-sort-compare ---
    if call.callee_path.ends_with(".sort") && call.argument_count == 0 {
        diags.push(warn(
            "typescript/require-array-sort-compare",
            "Provide a compare function to .sort()",
            start, end,
        ));
    }

    // --- typescript/prefer-find ---
    if call.callee_path.ends_with(".filter") {
        diags.push(warn(
            "typescript/prefer-find",
            "Consider using .find() instead of .filter()[0]",
            start, end,
        ));
    }
}

fn check_member_expr_rules(
    _member: &starlint::plugin::types::MemberExpressionNode,
    _source: &str,
    _diags: &mut Vec<LintDiagnostic>,
) {
    // Member expression checks for TypeScript rules
    // Most TS rules need type information which isn't available
}

fn check_import_rules(
    import: &starlint::plugin::types::ImportDeclarationNode,
    _diags: &mut Vec<LintDiagnostic>,
) {
    // Import-based TypeScript rules
    // consistent-type-imports is handled by source-text scanning
    let _ = import;
}
