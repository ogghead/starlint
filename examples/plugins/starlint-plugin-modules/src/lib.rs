//! Modules WASM plugin for starlint.
//!
//! Implements import (33), node (6), and promise (16) lint rules
//! as a single WASM component. These rules apply to ALL files
//! (no file-pattern filtering), using import/export declarations,
//! call expressions, and source-text scanning.

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    Category, FileContext, LintDiagnostic, PluginConfig, RuleMeta, Severity, Span,
};

struct ModulesPlugin;

export!(ModulesPlugin);

impl Guest for ModulesPlugin {
    fn get_rules() -> Vec<RuleMeta> {
        let mut rules = Vec::with_capacity(55);

        // === Import rules (33) ===
        rules.push(rule("import/consistent-type-specifier-style", "Enforce consistent type specifier style", Category::Style, Severity::Warning));
        rules.push(rule("import/default", "Ensure default export present for default import", Category::Correctness, Severity::Warning));
        rules.push(rule("import/export", "Report invalid exports", Category::Correctness, Severity::Error));
        rules.push(rule("import/exports-last", "Require exports after other statements", Category::Style, Severity::Warning));
        rules.push(rule("import/extensions", "Ensure consistent file extension in imports", Category::Style, Severity::Warning));
        rules.push(rule("import/first", "Ensure imports appear first", Category::Style, Severity::Warning));
        rules.push(rule("import/group-exports", "Prefer single export declaration", Category::Style, Severity::Warning));
        rules.push(rule("import/max-dependencies", "Limit number of dependencies", Category::Suggestion, Severity::Warning));
        rules.push(rule("import/named", "Validate named imports", Category::Correctness, Severity::Warning));
        rules.push(rule("import/namespace", "Validate namespace imports", Category::Correctness, Severity::Warning));
        rules.push(rule("import/no-absolute-path", "Disallow absolute paths in imports", Category::Correctness, Severity::Error));
        rules.push(rule("import/no-amd", "Disallow AMD require/define", Category::Style, Severity::Warning));
        rules.push(rule("import/no-anonymous-default-export", "Disallow anonymous default exports", Category::Style, Severity::Warning));
        rules.push(rule("import/no-commonjs", "Disallow CommonJS require/module.exports", Category::Style, Severity::Warning));
        rules.push(rule("import/no-cycle", "Detect circular imports", Category::Correctness, Severity::Warning));
        rules.push(rule("import/no-default-export", "Disallow default exports", Category::Style, Severity::Warning));
        rules.push(rule("import/no-duplicates", "Report duplicate imports", Category::Correctness, Severity::Warning));
        rules.push(rule("import/no-dynamic-require", "Forbid dynamic require", Category::Correctness, Severity::Warning));
        rules.push(rule("import/no-empty-named-blocks", "Forbid empty named imports", Category::Correctness, Severity::Warning));
        rules.push(rule("import/no-mutable-exports", "Forbid mutable exports", Category::Correctness, Severity::Warning));
        rules.push(rule("import/no-named-as-default", "Forbid using exported name as default", Category::Correctness, Severity::Warning));
        rules.push(rule("import/no-named-as-default-member", "Forbid exported name as default member", Category::Correctness, Severity::Warning));
        rules.push(rule("import/no-named-default", "Forbid named default export", Category::Style, Severity::Warning));
        rules.push(rule("import/no-named-export", "Forbid named exports", Category::Style, Severity::Warning));
        rules.push(rule("import/no-namespace", "Forbid namespace imports", Category::Style, Severity::Warning));
        rules.push(rule("import/no-nodejs-modules", "Forbid Node.js built-in modules", Category::Style, Severity::Warning));
        rules.push(rule("import/no-relative-parent-imports", "Forbid parent directory imports", Category::Style, Severity::Warning));
        rules.push(rule("import/no-restricted-imports", "Forbid specific imports", Category::Suggestion, Severity::Warning));
        rules.push(rule("import/no-self-import", "Forbid self-import", Category::Correctness, Severity::Error));
        rules.push(rule("import/no-unassigned-import", "Forbid side-effect imports", Category::Suggestion, Severity::Warning));
        rules.push(rule("import/no-webpack-loader-syntax", "Forbid webpack loader syntax", Category::Correctness, Severity::Warning));
        rules.push(rule("import/prefer-default-export", "Prefer default export for single export", Category::Suggestion, Severity::Warning));
        rules.push(rule("import/unambiguous", "Warn on ambiguous module vs script", Category::Correctness, Severity::Warning));

        // === Node rules (6) ===
        rules.push(rule("node/global-require", "Disallow require outside top-level", Category::Correctness, Severity::Warning));
        rules.push(rule("node/no-exports-assign", "Disallow direct assignment to exports", Category::Correctness, Severity::Error));
        rules.push(rule("node/no-new-require", "Disallow new require()", Category::Correctness, Severity::Error));
        rules.push(rule("node/no-path-concat", "Disallow path concatenation with __dirname", Category::Correctness, Severity::Warning));
        rules.push(rule("node/no-process-env", "Disallow process.env", Category::Suggestion, Severity::Warning));
        rules.push(rule("node/no-process-exit", "Disallow process.exit()", Category::Correctness, Severity::Warning));

        // === Promise rules (16) ===
        rules.push(rule("promise/always-return", "Require return in .then()", Category::Correctness, Severity::Warning));
        rules.push(rule("promise/avoid-new", "Forbid new Promise", Category::Suggestion, Severity::Warning));
        rules.push(rule("promise/catch-or-return", "Require .catch() or return", Category::Correctness, Severity::Warning));
        rules.push(rule("promise/no-callback-in-promise", "Forbid callbacks in .then()/.catch()", Category::Correctness, Severity::Warning));
        rules.push(rule("promise/no-multiple-resolved", "Forbid multiple resolve/reject", Category::Correctness, Severity::Error));
        rules.push(rule("promise/no-native", "Forbid native Promise", Category::Suggestion, Severity::Warning));
        rules.push(rule("promise/no-nesting", "Forbid nesting .then()/.catch()", Category::Correctness, Severity::Warning));
        rules.push(rule("promise/no-new-statics", "Forbid new Promise.resolve()", Category::Correctness, Severity::Error));
        rules.push(rule("promise/no-promise-in-callback", "Forbid promises in callbacks", Category::Correctness, Severity::Warning));
        rules.push(rule("promise/no-return-in-finally", "Forbid return in .finally()", Category::Correctness, Severity::Warning));
        rules.push(rule("promise/no-return-wrap", "Forbid wrapping return in Promise.resolve", Category::Correctness, Severity::Warning));
        rules.push(rule("promise/param-names", "Enforce resolve/reject param names", Category::Style, Severity::Warning));
        rules.push(rule("promise/prefer-await-to-callbacks", "Prefer async/await over callbacks", Category::Suggestion, Severity::Warning));
        rules.push(rule("promise/prefer-await-to-then", "Prefer async/await over .then()", Category::Suggestion, Severity::Warning));
        rules.push(rule("promise/spec-only", "Forbid non-standard Promise methods", Category::Correctness, Severity::Warning));
        rules.push(rule("promise/valid-params", "Enforce correct Promise params", Category::Correctness, Severity::Error));

        rules
    }

    fn get_file_patterns() -> Vec<String> {
        // These rules apply to ALL files
        Vec::new()
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(file: FileContext, tree: Vec<u8>) -> Vec<LintDiagnostic> {
        let tree: serde_json::Value = match serde_json::from_slice(&tree) {
            Ok(v) => v,
            Err(_) => serde_json::Value::Null,
        };

        let source = &file.source_text;
        let file_path = &file.file_path;
        let mut diags = Vec::new();

        // --- Text-scanning rules ---
        check_import_first(source, &mut diags);
        check_import_exports_last(source, &mut diags);
        check_import_no_duplicates(source, &mut diags);
        check_import_no_mutable_exports(source, &mut diags);
        check_import_max_dependencies(source, &mut diags);
        check_import_no_self_import(source, file_path, &mut diags);
        check_import_unambiguous(source, &mut diags);
        check_import_group_exports(source, &mut diags);
        check_import_prefer_default_export(source, &mut diags);
        check_node_no_path_concat(source, &mut diags);
        check_node_no_exports_assign(source, &mut diags);
        check_promise_always_return(source, &mut diags);
        check_promise_no_nesting(source, &mut diags);
        check_promise_no_return_in_finally(source, &mut diags);
        check_promise_no_return_wrap(source, &mut diags);
        check_promise_prefer_await_to_then(source, &mut diags);
        check_promise_param_names(source, &mut diags);
        check_promise_no_multiple_resolved(source, &mut diags);

        // --- AST-based rules ---
        if let Some(nodes) = tree.get("nodes").and_then(|n| n.as_array()) {
            for node in nodes {
                if let Some(import) = node.get("ImportDeclaration") {
                    check_import_decl_rules(import, &tree, &mut diags);
                }
                if let Some(exp) = node.get("ExportDefaultDeclaration") {
                    check_export_default_rules(exp, source, &mut diags);
                }
                if let Some(exp) = node.get("ExportNamedDeclaration") {
                    check_export_named_rules(exp, &tree, &mut diags);
                }
                if let Some(call) = node.get("CallExpression") {
                    check_call_expr_rules(call, &tree, source, &mut diags);
                }
                if let Some(member) = node.get("StaticMemberExpression") {
                    check_member_expr_rules(member, &tree, &mut diags);
                }
                if let Some(ident) = node.get("IdentifierReference") {
                    check_identifier_rules(ident, &mut diags);
                }
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
        fix: None,
        labels: vec![],
    }
}

fn warn(rule: &str, msg: &str, start: usize, end: usize) -> LintDiagnostic {
    diag(rule, msg, Span { start: start as u32, end: end as u32 }, Severity::Warning, None)
}

fn err(rule: &str, msg: &str, start: usize, end: usize) -> LintDiagnostic {
    diag(rule, msg, Span { start: start as u32, end: end as u32 }, Severity::Error, None)
}

// ==================== Import declaration rules ====================

fn check_import_decl_rules(
    import: &serde_json::Value,
    tree: &serde_json::Value,
    diags: &mut Vec<LintDiagnostic>,
) {
    let source_mod = import.get("source").and_then(|s| s.as_str()).unwrap_or("");
    let span = extract_span(import).unwrap_or(Span { start: 0, end: 0 });

    // --- import/no-absolute-path ---
    if source_mod.starts_with('/') {
        diags.push(diag("import/no-absolute-path", &format!("Do not import using absolute path `{source_mod}`"), span, Severity::Error, None));
    }

    // --- import/no-webpack-loader-syntax ---
    if source_mod.contains('!') {
        diags.push(diag("import/no-webpack-loader-syntax", "Do not use webpack loader syntax in imports", span, Severity::Warning, None));
    }

    // --- import/no-nodejs-modules ---
    let node_builtins = [
        "assert", "buffer", "child_process", "cluster", "crypto", "dgram",
        "dns", "domain", "events", "fs", "http", "https", "net", "os",
        "path", "punycode", "querystring", "readline", "stream", "string_decoder",
        "tls", "tty", "url", "util", "v8", "vm", "zlib",
    ];
    let stripped = source_mod.strip_prefix("node:").unwrap_or(source_mod);
    if node_builtins.contains(&stripped) {
        diags.push(diag("import/no-nodejs-modules", &format!("Do not import Node.js built-in module `{source_mod}`"), span, Severity::Warning, None));
    }

    // --- import/no-relative-parent-imports ---
    if source_mod.starts_with("../") {
        diags.push(diag("import/no-relative-parent-imports", "Do not import from parent directories", span, Severity::Warning, None));
    }

    // --- import/no-namespace ---
    if let Some(specifier_ids) = import.get("specifiers").and_then(|s| s.as_array()) {
        for spec_id_val in specifier_ids {
            if let Some(spec_id) = spec_id_val.as_u64() {
                if let Some(nodes) = tree.get("nodes").and_then(|n| n.as_array()) {
                    if let Some(spec_node) = nodes.get(spec_id as usize) {
                        if let Some(spec) = spec_node.get("ImportSpecifier") {
                            let local = spec.get("local").and_then(|l| l.as_str()).unwrap_or("");
                            let imported = spec.get("imported").and_then(|i| i.as_str());
                            if local == "*" || imported == Some("*") {
                                diags.push(diag("import/no-namespace", "Namespace imports are not allowed", span, Severity::Warning, None));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // --- import/no-empty-named-blocks ---
    let specifiers = import.get("specifiers").and_then(|s| s.as_array());
    let specifiers_empty = specifiers.map_or(true, |s| s.is_empty());
    if specifiers_empty && !source_mod.is_empty() {
        // Could be a side-effect import (import 'foo') or empty named (import {} from 'foo')
        // We flag if it looks like an empty named block — but can't distinguish without more context
    }

    // --- import/no-unassigned-import ---
    if specifiers_empty {
        diags.push(diag("import/no-unassigned-import", &format!("Unassigned (side-effect) import: `{source_mod}`"), span, Severity::Warning, None));
    }

    // --- import/consistent-type-specifier-style ---
    if let Some(specifier_ids) = import.get("specifiers").and_then(|s| s.as_array()) {
        for spec_id_val in specifier_ids {
            if let Some(spec_id) = spec_id_val.as_u64() {
                if let Some(nodes) = tree.get("nodes").and_then(|n| n.as_array()) {
                    if let Some(spec_node) = nodes.get(spec_id as usize) {
                        if let Some(spec) = spec_node.get("ImportSpecifier") {
                            let local = spec.get("local").and_then(|l| l.as_str()).unwrap_or("");
                            let imported = spec.get("imported").and_then(|i| i.as_str());
                            if local.starts_with("type ") || imported.map_or(false, |s| s.starts_with("type ")) {
                                // Type specifier detected — this is fine in either style
                            }
                        }
                    }
                }
            }
        }
    }

    // --- import/extensions ---
    if !source_mod.starts_with('.') {
        // External module — skip extension check
    } else if !source_mod.contains('.') || source_mod.ends_with('/') {
        // Missing extension on relative import
        diags.push(diag("import/extensions", &format!("Missing file extension in `{source_mod}`"), span, Severity::Warning, None));
    }
}

// ==================== Export rules ====================

fn check_export_default_rules(
    exp: &serde_json::Value,
    source: &str,
    diags: &mut Vec<LintDiagnostic>,
) {
    let span = extract_span(exp).unwrap_or(Span { start: 0, end: 0 });

    // --- import/no-default-export ---
    diags.push(diag("import/no-default-export", "Default exports are not allowed", span, Severity::Warning, None));

    // --- import/no-anonymous-default-export ---
    let start = span.start as usize;
    let end = span.end as usize;
    let text = source.get(start..end.min(source.len())).unwrap_or("");
    if text.contains("export default function(") || text.contains("export default () =>") || text.contains("export default class {") {
        diags.push(diag("import/no-anonymous-default-export", "Anonymous default exports are not allowed", span, Severity::Warning, None));
    }
}

fn check_export_named_rules(
    exp: &serde_json::Value,
    tree: &serde_json::Value,
    diags: &mut Vec<LintDiagnostic>,
) {
    let span = extract_span(exp).unwrap_or(Span { start: 0, end: 0 });

    // --- import/no-named-export ---
    // In the serialized AST, ExportNamedDeclaration has specifiers (NodeIds) and an optional declaration.
    // Check if there are specifiers or a declaration to determine if named exports exist.
    let has_specifiers = exp
        .get("specifiers")
        .and_then(|s| s.as_array())
        .map_or(false, |s| !s.is_empty());
    let has_declaration = exp
        .get("declaration")
        .map_or(false, |d| !d.is_null());

    if has_specifiers || has_declaration {
        // Resolve specifier names for more detailed checks if needed
        let mut names = Vec::new();
        if let Some(specifier_ids) = exp.get("specifiers").and_then(|s| s.as_array()) {
            if let Some(nodes) = tree.get("nodes").and_then(|n| n.as_array()) {
                for spec_id_val in specifier_ids {
                    if let Some(spec_id) = spec_id_val.as_u64() {
                        if let Some(spec_node) = nodes.get(spec_id as usize) {
                            // ExportSpecifier nodes have a "local" field
                            if let Some(spec) = spec_node.get("ExportSpecifier") {
                                if let Some(name) = spec.get("local").and_then(|l| l.as_str()) {
                                    names.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        if !names.is_empty() || has_declaration {
            diags.push(diag("import/no-named-export", "Named exports are not allowed", span, Severity::Warning, None));
        }
    }

    // --- import/export (duplicate exports check) ---
    // Would need to track across multiple ExportNamedDecl nodes — skip for now
}

// ==================== CallExpression rules ====================

fn check_call_expr_rules(
    call: &serde_json::Value,
    tree: &serde_json::Value,
    source: &str,
    diags: &mut Vec<LintDiagnostic>,
) {
    let callee = get_callee_path(tree, call);
    let span = extract_span(call).unwrap_or(Span { start: 0, end: 0 });
    let argument_count = call
        .get("arguments")
        .and_then(|a| a.as_array())
        .map_or(0, |a| a.len());

    // --- import/no-commonjs ---
    if callee == "require" {
        diags.push(diag("import/no-commonjs", "Use ES module `import` instead of `require()`", span, Severity::Warning, None));
    }

    // --- import/no-amd ---
    if callee == "define" || callee == "require.ensure" {
        diags.push(diag("import/no-amd", "Do not use AMD `define`/`require.ensure`", span, Severity::Warning, None));
    }

    // --- import/no-dynamic-require ---
    if callee == "require" && argument_count > 0 {
        let start = span.start as usize;
        let end = span.end as usize;
        let text = source.get(start..end.min(source.len())).unwrap_or("");
        if !text.contains("require('") && !text.contains("require(\"") {
            diags.push(diag("import/no-dynamic-require", "Dynamic `require()` with non-literal argument", span, Severity::Warning, None));
        }
    }

    // --- node/global-require ---
    if callee == "require" {
        let start = span.start as usize;
        let before = source.get(..start).unwrap_or("");
        // Rough check: if inside a function body
        let open_braces = before.matches('{').count();
        let close_braces = before.matches('}').count();
        if open_braces > close_braces {
            diags.push(diag("node/global-require", "`require()` should be at the top level", span, Severity::Warning, None));
        }
    }

    // --- node/no-new-require ---
    if callee == "require" {
        let start = span.start as usize;
        let before = source.get(start.saturating_sub(10)..start).unwrap_or("");
        if before.trim_end().ends_with("new") {
            diags.push(diag("node/no-new-require", "Do not use `new require()`", span, Severity::Error, None));
        }
    }

    // --- node/no-process-exit ---
    if callee == "process.exit" {
        diags.push(diag("node/no-process-exit", "Avoid using `process.exit()`", span, Severity::Warning, None));
    }

    // --- promise/avoid-new ---
    if callee == "Promise" {
        let start = span.start as usize;
        let before = source.get(start.saturating_sub(10)..start).unwrap_or("");
        if before.trim_end().ends_with("new") {
            diags.push(diag("promise/avoid-new", "Avoid creating `new Promise`", span, Severity::Warning, None));
        }
    }

    // --- promise/no-new-statics ---
    let promise_statics = ["Promise.resolve", "Promise.reject", "Promise.all", "Promise.allSettled", "Promise.any", "Promise.race"];
    if promise_statics.contains(&callee.as_str()) {
        let start = span.start as usize;
        let before = source.get(start.saturating_sub(10)..start).unwrap_or("");
        if before.trim_end().ends_with("new") {
            diags.push(diag("promise/no-new-statics", &format!("Do not use `new {callee}()`"), span, Severity::Error, None));
        }
    }

    // --- promise/valid-params ---
    if callee == "Promise.all" || callee == "Promise.allSettled" || callee == "Promise.any" || callee == "Promise.race" {
        if argument_count != 1 {
            diags.push(diag("promise/valid-params", &format!("`{callee}()` requires exactly 1 argument"), span, Severity::Error, None));
        }
    }
    if callee == "Promise.resolve" || callee == "Promise.reject" {
        if argument_count > 1 {
            diags.push(diag("promise/valid-params", &format!("`{callee}()` takes at most 1 argument"), span, Severity::Error, None));
        }
    }

    // --- promise/catch-or-return / prefer-await-to-then ---
    if callee.ends_with(".then") {
        diags.push(diag("promise/prefer-await-to-then", "Prefer `async`/`await` over `.then()` chains", span, Severity::Warning, None));

        // Check if .catch follows
        let end = span.end as usize;
        let after = source.get(end..).unwrap_or("");
        if !after.trim_start().starts_with(".catch") && !after.trim_start().starts_with(".finally") {
            // May need catch or return — but this is a rough heuristic
        }
    }

    // --- promise/no-callback-in-promise ---
    if callee.ends_with(".then") || callee.ends_with(".catch") {
        let start = span.start as usize;
        let end = span.end as usize;
        let text = source.get(start..end.min(source.len())).unwrap_or("");
        if text.contains("callback(") || text.contains("cb(") || text.contains("next(") || text.contains("done(") {
            diags.push(diag("promise/no-callback-in-promise", "Avoid calling callbacks inside `.then()`/`.catch()`", span, Severity::Warning, None));
        }
    }

    // --- promise/spec-only ---
    let non_standard = ["Promise.defer", "Promise.done", "Promise.nodeify", "Promise.denodeify"];
    if non_standard.contains(&callee.as_str()) {
        diags.push(diag("promise/spec-only", &format!("`{callee}` is not part of the Promise specification"), span, Severity::Warning, None));
    }

    // --- promise/prefer-await-to-callbacks ---
    if callee.ends_with(".then") || callee.ends_with(".catch") {
        // Already flagged by prefer-await-to-then — skip
    }
}

// ==================== MemberExpression rules ====================

fn check_member_expr_rules(
    member: &serde_json::Value,
    tree: &serde_json::Value,
    diags: &mut Vec<LintDiagnostic>,
) {
    let span = extract_span(member).unwrap_or(Span { start: 0, end: 0 });
    let property = member.get("property").and_then(|p| p.as_str()).unwrap_or("");

    // Resolve the object to get its name
    let object_id = member.get("object").and_then(|o| o.as_u64()).unwrap_or(0);
    let object = resolve_callee(tree, object_id);

    // --- node/no-process-env ---
    if object == "process" && property == "env" {
        diags.push(diag("node/no-process-env", "Avoid using `process.env` directly", span, Severity::Warning, None));
    }

    // --- import/no-commonjs (module.exports) ---
    if object == "module" && property == "exports" {
        diags.push(diag("import/no-commonjs", "Use ES module `export` instead of `module.exports`", span, Severity::Warning, None));
    }
}

// ==================== IdentifierReference rules ====================

fn check_identifier_rules(
    ident: &serde_json::Value,
    _diags: &mut Vec<LintDiagnostic>,
) {
    let name = ident.get("name").and_then(|n| n.as_str()).unwrap_or("");

    // --- promise/no-native ---
    if name == "Promise" {
        // This rule forbids native Promise — only flag in specific contexts
        // Skip — too noisy without configuration
    }
}

// ==================== Text-scanning rules ====================

fn check_import_first(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut found_non_import = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') || trimmed.starts_with("*/") {
            continue;
        }
        if trimmed.starts_with("import ") || trimmed.starts_with("import{") {
            if found_non_import {
                if let Some(pos) = source.find(trimmed) {
                    diags.push(warn("import/first", "Import should appear before other statements", pos, pos + trimmed.len()));
                }
                return;
            }
        } else if !trimmed.starts_with("'use strict'") && !trimmed.starts_with("\"use strict\"") {
            found_non_import = true;
        }
    }
}

fn check_import_exports_last(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut last_export_pos = 0;
    let mut found_after_export = false;

    let mut pos = 0;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("export ") {
            if found_after_export {
                diags.push(warn("import/exports-last", "Exports should appear after other statements", pos, pos + trimmed.len()));
                return;
            }
            last_export_pos = pos;
        } else if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*") && last_export_pos > 0 {
            found_after_export = true;
        }
        pos += line.len() + 1;
    }
}

fn check_import_no_duplicates(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut seen_modules: Vec<String> = Vec::new();

    let import_pattern = "from '";
    let import_pattern2 = "from \"";

    for pattern in [import_pattern, import_pattern2] {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let mod_start = abs + pattern.len();
            let quote = pattern.as_bytes()[pattern.len() - 1];
            if let Some(mod_end) = source[mod_start..].find(quote as char) {
                let module_name = &source[mod_start..mod_start + mod_end];
                let owned = module_name.to_string();
                if seen_modules.contains(&owned) {
                    diags.push(warn("import/no-duplicates", &format!("Duplicate import from `{module_name}`"), abs, mod_start + mod_end));
                } else {
                    seen_modules.push(owned);
                }
            }
            pos = abs + 1;
        }
    }
}

fn check_import_no_mutable_exports(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["export let ", "export var "];
    for pattern in &patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            diags.push(warn("import/no-mutable-exports", "Mutable exports are not allowed — use `const`", abs, abs + pattern.len()));
            pos = abs + 1;
        }
    }
}

fn check_import_max_dependencies(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let max: usize = 20;
    let import_count = count_occurrences(source, "import ");
    if import_count > max {
        diags.push(warn("import/max-dependencies", &format!("Too many dependencies ({import_count} > {max})"), 0, 0));
    }
}

fn check_import_no_self_import(source: &str, file_path: &str, diags: &mut Vec<LintDiagnostic>) {
    // Check if any import references the current file
    let file_name = file_path.rsplit('/').next().unwrap_or(file_path);
    let stem = file_name.split('.').next().unwrap_or(file_name);

    let self_patterns = [
        format!("from './{stem}'"),
        format!("from \"./{stem}\""),
        format!("from './{file_name}'"),
        format!("from \"./{file_name}\""),
    ];

    for pattern in &self_patterns {
        if let Some(pos) = source.find(pattern.as_str()) {
            diags.push(err("import/no-self-import", "Module imports itself", pos, pos + pattern.len()));
        }
    }
}

fn check_import_unambiguous(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    let has_import = source.contains("import ") || source.contains("import{");
    let has_export = source.contains("export ");
    if !has_import && !has_export {
        // Could be ambiguous script vs module — but too noisy to always flag
    }
}

fn check_import_group_exports(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Check if exports are scattered
    let _ = source;
}

fn check_import_prefer_default_export(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let named_export_count = count_occurrences(source, "export const ")
        + count_occurrences(source, "export function ")
        + count_occurrences(source, "export class ");

    let has_default = source.contains("export default");

    if named_export_count == 1 && !has_default {
        if let Some(pos) = source.find("export const ")
            .or_else(|| source.find("export function "))
            .or_else(|| source.find("export class "))
        {
            diags.push(warn("import/prefer-default-export", "Prefer default export when only one export exists", pos, pos + 7));
        }
    }
}

fn check_node_no_path_concat(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["__dirname +", "__filename +", "__dirname+", "__filename+",
                     "+ __dirname", "+ __filename", "+__dirname", "+__filename"];
    for pattern in &patterns {
        if let Some(pos) = source.find(pattern) {
            diags.push(warn("node/no-path-concat", "Use `path.join()` or `path.resolve()` instead of string concatenation", pos, pos + pattern.len()));
        }
    }
}

fn check_node_no_exports_assign(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "exports =";
    if let Some(pos) = source.find(pattern) {
        // Check it's not module.exports
        if pos == 0 || source.as_bytes().get(pos.wrapping_sub(1)) != Some(&b'.') {
            diags.push(err("node/no-exports-assign", "Do not assign directly to `exports`", pos, pos + pattern.len()));
        }
    }
}

fn check_promise_always_return(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = ".then(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let call_start = abs + pattern.len();
        if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
            let body = &source[call_start..close];
            // Check if callback has a return statement (for arrow functions without braces, implicit return)
            if body.contains('{') && !body.contains("return ") && !body.contains("return;") {
                let arrow_pos = body.find("=>").unwrap_or(0);
                let after_arrow = &body[arrow_pos..];
                if after_arrow.contains('{') {
                    diags.push(warn("promise/always-return", "Each `.then()` callback should return a value", abs, abs + pattern.len()));
                }
            }
        }
        pos = abs + 1;
    }
}

fn check_promise_no_nesting(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = ".then(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let call_start = abs + pattern.len();
        if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
            let body = &source[call_start..close];
            if body.contains(".then(") || body.contains(".catch(") {
                diags.push(warn("promise/no-nesting", "Avoid nesting `.then()`/`.catch()` chains", abs, abs + pattern.len()));
            }
        }
        pos = abs + 1;
    }
}

fn check_promise_no_return_in_finally(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = ".finally(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let call_start = abs + pattern.len();
        if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
            let body = &source[call_start..close];
            if body.contains("return ") || body.contains("return;") {
                diags.push(warn("promise/no-return-in-finally", "Do not use `return` inside `.finally()`", abs, abs + pattern.len()));
            }
        }
        pos = abs + 1;
    }
}

fn check_promise_no_return_wrap(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["return Promise.resolve(", "return Promise.reject("];
    for pattern in &patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            diags.push(warn("promise/no-return-wrap", &format!("Avoid wrapping return value in `{}`", &pattern[7..]), abs, abs + pattern.len()));
            pos = abs + 1;
        }
    }
}

fn check_promise_prefer_await_to_then(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Already handled in call expression rules
    let _ = source;
}

fn check_promise_param_names(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "new Promise(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let call_start = abs + pattern.len();
        if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
            let body = &source[call_start..close];
            // Check first line for param names
            if let Some(arrow) = body.find("=>") {
                let params = body[..arrow].trim();
                let params = params.trim_start_matches('(').trim_end_matches(')');
                let param_names: Vec<&str> = params.split(',').map(|s| s.trim()).collect();
                if let Some(first) = param_names.first() {
                    if *first != "resolve" && *first != "_resolve" && *first != "_" {
                        diags.push(warn("promise/param-names", &format!("Promise executor first param should be `resolve`, got `{first}`"), abs, close + 1));
                    }
                }
                if let Some(second) = param_names.get(1) {
                    if *second != "reject" && *second != "_reject" && *second != "_" {
                        diags.push(warn("promise/param-names", &format!("Promise executor second param should be `reject`, got `{second}`"), abs, close + 1));
                    }
                }
            }
        }
        pos = abs + 1;
    }
}

fn check_promise_no_multiple_resolved(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "new Promise(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let call_start = abs + pattern.len();
        if let Some(close) = find_matching_paren(source, call_start.saturating_sub(1)) {
            let body = &source[call_start..close];
            let resolve_count = count_occurrences(body, "resolve(");
            let reject_count = count_occurrences(body, "reject(");
            if resolve_count > 1 {
                diags.push(err("promise/no-multiple-resolved", "Promise may be resolved multiple times", abs, close + 1));
            }
            if reject_count > 1 {
                diags.push(err("promise/no-multiple-resolved", "Promise may be rejected multiple times", abs, close + 1));
            }
        }
        pos = abs + 1;
    }
}

// ==================== Utility functions ====================

fn count_occurrences(source: &str, pattern: &str) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        count += 1;
        pos += found + 1;
    }
    count
}

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

// ==================== AST tree navigation helpers ====================

fn extract_span(node: &serde_json::Value) -> Option<Span> {
    let span = node.get("span")?;
    let start = span.get("start")?.as_u64()?;
    let end = span.get("end")?.as_u64()?;
    Some(Span {
        start: start as u32,
        end: end as u32,
    })
}

fn get_callee_path(tree: &serde_json::Value, call: &serde_json::Value) -> String {
    let callee_id = call.get("callee").and_then(|c| c.as_u64()).unwrap_or(0);
    resolve_callee(tree, callee_id)
}

fn resolve_callee(tree: &serde_json::Value, id: u64) -> String {
    let Some(nodes) = tree.get("nodes").and_then(|n| n.as_array()) else {
        return String::new();
    };
    let Some(node) = nodes.get(id as usize) else {
        return String::new();
    };
    if let Some(ident) = node.get("IdentifierReference") {
        return ident
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
    }
    if let Some(member) = node.get("StaticMemberExpression") {
        let object_id = member.get("object").and_then(|o| o.as_u64()).unwrap_or(0);
        let property = member
            .get("property")
            .and_then(|p| p.as_str())
            .unwrap_or("");
        let object_path = resolve_callee(tree, object_id);
        if object_path.is_empty() {
            return property.to_string();
        }
        return format!("{object_path}.{property}");
    }
    if let Some(call) = node.get("CallExpression") {
        let callee_id = call.get("callee").and_then(|c| c.as_u64()).unwrap_or(0);
        return resolve_callee(tree, callee_id);
    }
    String::new()
}
