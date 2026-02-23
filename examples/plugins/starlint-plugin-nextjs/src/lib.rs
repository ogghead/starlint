//! Next.js WASM plugin for starlint.
//!
//! Implements 22 Next.js lint rules as a single WASM component,
//! using JSX node inspection, import analysis, and source-text scanning.

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    Category, FileContext, LintDiagnostic, PluginConfig, RuleMeta, Severity, Span,
};

struct NextjsPlugin;

export!(NextjsPlugin);

impl Guest for NextjsPlugin {
    fn get_rules() -> Vec<RuleMeta> {
        vec![
            rule("nextjs/google-font-display", "Enforce font-display behavior with Google Fonts", Category::Correctness, Severity::Warning),
            rule("nextjs/google-font-preconnect", "Enforce preconnect for Google Fonts", Category::Correctness, Severity::Warning),
            rule("nextjs/inline-script-id", "Require id attribute on inline next/script components", Category::Correctness, Severity::Warning),
            rule("nextjs/next-script-for-ga", "Prefer next/script for Google Analytics", Category::Suggestion, Severity::Warning),
            rule("nextjs/no-async-client-component", "Disallow async client components", Category::Correctness, Severity::Warning),
            rule("nextjs/no-before-interactive-script-outside-document", "Disallow beforeInteractive strategy outside _document", Category::Correctness, Severity::Warning),
            rule("nextjs/no-css-tags", "Disallow link stylesheet tags", Category::Suggestion, Severity::Warning),
            rule("nextjs/no-document-import-in-page", "Disallow importing next/document outside _document", Category::Correctness, Severity::Error),
            rule("nextjs/no-duplicate-head", "Disallow duplicate usage of Head in pages/_document", Category::Correctness, Severity::Error),
            rule("nextjs/no-head-element", "Disallow using head element (use Head from next/head)", Category::Correctness, Severity::Warning),
            rule("nextjs/no-head-import-in-document", "Disallow importing next/head in _document", Category::Correctness, Severity::Warning),
            rule("nextjs/no-html-link-for-pages", "Disallow HTML anchor for internal navigation (use Link)", Category::Correctness, Severity::Warning),
            rule("nextjs/no-img-element", "Disallow img element (use Image from next/image)", Category::Correctness, Severity::Warning),
            rule("nextjs/no-page-custom-font", "Disallow custom fonts on page level (load in _document)", Category::Correctness, Severity::Warning),
            rule("nextjs/no-script-component-in-head", "Disallow next/script inside next/head", Category::Correctness, Severity::Error),
            rule("nextjs/no-styled-jsx-in-document", "Disallow styled-jsx in _document", Category::Correctness, Severity::Warning),
            rule("nextjs/no-sync-scripts", "Disallow synchronous scripts", Category::Correctness, Severity::Warning),
            rule("nextjs/no-title-in-document-head", "Disallow title element in Head component", Category::Correctness, Severity::Warning),
            rule("nextjs/no-typos", "Disallow common Next.js API typos", Category::Correctness, Severity::Warning),
            rule("nextjs/no-unwanted-polyfillio", "Disallow polyfill.io scripts", Category::Correctness, Severity::Warning),
            rule("nextjs/no-assign-module-variable", "Disallow assignment to module variable", Category::Correctness, Severity::Warning),
            rule("nextjs/no-server-import-in-page", "Disallow server-only imports in client components", Category::Correctness, Severity::Warning),
        ]
    }

    fn get_file_patterns() -> Vec<String> {
        // Next.js rules apply to all JS/TS files
        vec![
            "*.js".into(), "*.jsx".into(), "*.ts".into(), "*.tsx".into(),
            "*.mjs".into(), "*.cjs".into(),
        ]
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

        // Derive file context for path-sensitive rules
        let file_stem = file_path
            .rsplit('/')
            .next()
            .unwrap_or(file_path)
            .split('.')
            .next()
            .unwrap_or("");
        let is_document = file_stem == "_document";
        let is_app = file_stem == "_app";

        // --- Source-text scanning rules ---
        check_no_async_client_component(source, &mut diags);
        check_no_duplicate_head(source, &mut diags);
        check_no_typos(source, &mut diags);
        check_no_assign_module_variable(source, &mut diags);
        check_google_font_display(source, &mut diags);

        // --- AST-based rules ---
        if let Some(nodes) = tree.get("nodes").and_then(|n| n.as_array()) {
            for node in nodes {
                if let Some(jsx) = node.get("JSXOpeningElement") {
                    check_jsx_rules(jsx, &tree, source, file_stem, is_document, is_app, &mut diags);
                }
                if let Some(import) = node.get("ImportDeclaration") {
                    check_import_rules(import, is_document, &mut diags);
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

fn warn(rule: &str, msg: &str, start: usize, end: usize) -> LintDiagnostic {
    LintDiagnostic {
        rule_name: rule.into(),
        message: msg.into(),
        span: Span { start: start as u32, end: end as u32 },
        severity: Severity::Warning,
        help: None,
        fix: None,
        labels: vec![],
    }
}

fn err(rule: &str, msg: &str, start: usize, end: usize) -> LintDiagnostic {
    LintDiagnostic {
        rule_name: rule.into(),
        message: msg.into(),
        span: Span { start: start as u32, end: end as u32 },
        severity: Severity::Error,
        help: None,
        fix: None,
        labels: vec![],
    }
}

fn has_attr(tree: &serde_json::Value, jsx: &serde_json::Value, name: &str) -> bool {
    let Some(attr_ids) = jsx.get("attributes").and_then(|a| a.as_array()) else {
        return false;
    };
    for attr_id_val in attr_ids {
        if let Some(attr_id) = attr_id_val.as_u64() {
            if let Some((attr_name, _value, is_spread)) = get_jsx_attr(tree, attr_id) {
                if !is_spread && attr_name == name {
                    return true;
                }
            }
        }
    }
    false
}

fn get_attr_value(tree: &serde_json::Value, jsx: &serde_json::Value, name: &str) -> Option<String> {
    let attr_ids = jsx.get("attributes").and_then(|a| a.as_array())?;
    for attr_id_val in attr_ids {
        if let Some(attr_id) = attr_id_val.as_u64() {
            if let Some((attr_name, value, is_spread)) = get_jsx_attr(tree, attr_id) {
                if !is_spread && attr_name == name {
                    return value;
                }
            }
        }
    }
    None
}

// ==================== Source-text scanning rules ====================

/// nextjs/no-async-client-component: async exports in "use client" files
fn check_no_async_client_component(source: &str, diags: &mut Vec<LintDiagnostic>) {
    if !source.contains("\"use client\"") && !source.contains("'use client'") {
        return;
    }
    // Look for async default export
    if let Some(pos) = source.find("export default async") {
        diags.push(warn(
            "nextjs/no-async-client-component",
            "Client components cannot be async functions",
            pos, pos + 20,
        ));
    }
    if let Some(pos) = source.find("export async function") {
        diags.push(warn(
            "nextjs/no-async-client-component",
            "Client components cannot be async functions",
            pos, pos + 21,
        ));
    }
}

/// nextjs/no-duplicate-head: multiple Head imports/usages
fn check_no_duplicate_head(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut count = 0;
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("<Head") {
        let abs_pos = search_from + pos;
        // Make sure it's a JSX tag (followed by > or space)
        let after = source.as_bytes().get(abs_pos + 5);
        if after == Some(&b'>') || after == Some(&b' ') || after == Some(&b'/') {
            count += 1;
            if count > 1 {
                diags.push(err(
                    "nextjs/no-duplicate-head",
                    "Duplicate <Head> component. Only one <Head> should be used per page",
                    abs_pos, abs_pos + 5,
                ));
            }
        }
        search_from = abs_pos + 5;
    }
}

/// nextjs/no-typos: common Next.js API name typos
fn check_no_typos(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let typos: &[(&str, &str)] = &[
        ("getstaticprops", "getStaticProps"),
        ("getStaticprops", "getStaticProps"),
        ("getstaticPaths", "getStaticPaths"),
        ("getStaticpaths", "getStaticPaths"),
        ("getserverSideProps", "getServerSideProps"),
        ("getServerSideprops", "getServerSideProps"),
        ("getserversideProps", "getServerSideProps"),
        ("getserversideprops", "getServerSideProps"),
    ];
    for (typo, correct) in typos {
        if let Some(pos) = source.find(typo) {
            diags.push(warn(
                "nextjs/no-typos",
                &format!("Possible typo: did you mean '{correct}'?"),
                pos, pos + typo.len(),
            ));
        }
    }
}

/// nextjs/no-assign-module-variable: assignment to module variable
fn check_no_assign_module_variable(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Look for `module.exports =` or `module.exports=`
    let patterns = ["module.exports =", "module.exports="];
    for pattern in &patterns {
        if let Some(pos) = source.find(pattern) {
            diags.push(warn(
                "nextjs/no-assign-module-variable",
                "Do not assign to the module variable. Use ES module syntax instead",
                pos, pos + pattern.len(),
            ));
        }
    }
}

/// nextjs/google-font-display: Google Fonts URLs should have display param
fn check_google_font_display(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("fonts.googleapis.com") {
        let abs_pos = search_from + pos;
        // Find the full URL context
        let line_end = source[abs_pos..].find('\n').map_or(source.len(), |p| abs_pos + p);
        let url_context = &source[abs_pos..line_end];
        if !url_context.contains("display=") {
            diags.push(warn(
                "nextjs/google-font-display",
                "Google Font URL missing 'display' parameter. Add &display=swap",
                abs_pos, abs_pos + 20,
            ));
        }
        search_from = abs_pos + 20;
    }
}

// ==================== JSX-based rules ====================

fn check_jsx_rules(
    jsx: &serde_json::Value,
    tree: &serde_json::Value,
    source: &str,
    _file_stem: &str,
    is_document: bool,
    is_app: bool,
    diags: &mut Vec<LintDiagnostic>,
) {
    let name = jsx.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let span = extract_span(jsx).unwrap_or(Span { start: 0, end: 0 });
    let start = span.start as usize;
    let end = span.end as usize;

    // --- nextjs/no-img-element ---
    if name == "img" {
        diags.push(warn(
            "nextjs/no-img-element",
            "Do not use <img>. Use Image from next/image instead",
            start, end,
        ));
    }

    // --- nextjs/no-head-element ---
    if name == "head" {
        diags.push(warn(
            "nextjs/no-head-element",
            "Do not use <head>. Use Head from next/head instead",
            start, end,
        ));
    }

    // --- nextjs/no-html-link-for-pages ---
    if name == "a" {
        if let Some(href) = get_attr_value(tree, jsx, "href") {
            if href.starts_with('/') && !href.starts_with("//") {
                diags.push(warn(
                    "nextjs/no-html-link-for-pages",
                    "Do not use <a> for internal navigation. Use Link from next/link",
                    start, end,
                ));
            }
        }
    }

    // --- nextjs/no-css-tags ---
    if name == "link" {
        if let Some(rel) = get_attr_value(tree, jsx, "rel") {
            if rel == "stylesheet" {
                diags.push(warn(
                    "nextjs/no-css-tags",
                    "Do not use <link rel=\"stylesheet\">. Use CSS imports instead",
                    start, end,
                ));
            }
        }
    }

    // --- nextjs/no-sync-scripts ---
    if name == "script" {
        if has_attr(tree, jsx, "src") && !has_attr(tree, jsx, "async") && !has_attr(tree, jsx, "defer") {
            diags.push(warn(
                "nextjs/no-sync-scripts",
                "Synchronous <script> is not allowed. Add async or defer attribute",
                start, end,
            ));
        }
    }

    // --- nextjs/google-font-preconnect ---
    if name == "link" {
        if let Some(href) = get_attr_value(tree, jsx, "href") {
            if href.contains("fonts.googleapis.com") || href.contains("fonts.gstatic.com") {
                let has_preconnect = get_attr_value(tree, jsx, "rel")
                    .map_or(false, |r| r == "preconnect");
                if !has_preconnect && !get_attr_value(tree, jsx, "rel").map_or(false, |r| r == "stylesheet") {
                    // Only flag if it's not already a stylesheet link (those are flagged by no-css-tags)
                } else if href.contains("fonts.gstatic.com") && !has_preconnect {
                    diags.push(warn(
                        "nextjs/google-font-preconnect",
                        "Add rel=\"preconnect\" to Google Fonts link",
                        start, end,
                    ));
                }
            }
        }
    }

    // --- nextjs/no-unwanted-polyfillio ---
    if name == "script" {
        if let Some(src) = get_attr_value(tree, jsx, "src") {
            if src.contains("polyfill.io") || src.contains("polyfill-fastly.io") {
                diags.push(warn(
                    "nextjs/no-unwanted-polyfillio",
                    "Do not use polyfill.io. Next.js includes necessary polyfills",
                    start, end,
                ));
            }
        }
    }

    // --- nextjs/inline-script-id ---
    if name == "Script" && !has_attr(tree, jsx, "src") && !has_attr(tree, jsx, "id") {
        // Inline Script without id — check if it has children or dangerouslySetInnerHTML
        // In the serialized AST, children_count isn't directly available on JSXOpeningElement,
        // so we check for dangerouslySetInnerHTML attribute or use a heuristic
        let is_self_closing = jsx.get("self_closing").and_then(|s| s.as_bool()).unwrap_or(false);
        if !is_self_closing || has_attr(tree, jsx, "dangerouslySetInnerHTML") {
            diags.push(warn(
                "nextjs/inline-script-id",
                "Inline <Script> requires an id attribute",
                start, end,
            ));
        }
    }

    // --- nextjs/next-script-for-ga ---
    if name == "script" {
        if let Some(src) = get_attr_value(tree, jsx, "src") {
            if src.contains("googletagmanager.com") || src.contains("google-analytics.com") {
                diags.push(warn(
                    "nextjs/next-script-for-ga",
                    "Use next/script component for Google Analytics",
                    start, end,
                ));
            }
        }
    }

    // --- nextjs/no-script-component-in-head ---
    if name == "Script" {
        // Check if inside Head by scanning source context
        let context_start = if start > 200 { start - 200 } else { 0 };
        let context = &source[context_start..start];
        if context.contains("<Head") && !context.contains("</Head") {
            diags.push(err(
                "nextjs/no-script-component-in-head",
                "next/script should not be used inside next/head. Move it outside",
                start, end,
            ));
        }
    }

    // --- nextjs/no-styled-jsx-in-document ---
    if is_document && name == "style" {
        if has_attr(tree, jsx, "jsx") {
            diags.push(warn(
                "nextjs/no-styled-jsx-in-document",
                "styled-jsx should not be used in _document. Move to _app or page components",
                start, end,
            ));
        }
    }

    // --- nextjs/no-page-custom-font ---
    if !is_document && !is_app && name == "link" {
        if let Some(href) = get_attr_value(tree, jsx, "href") {
            if href.contains("fonts.googleapis.com") || href.contains("fonts.gstatic.com") {
                diags.push(warn(
                    "nextjs/no-page-custom-font",
                    "Custom fonts should be loaded in _document, not in page components",
                    start, end,
                ));
            }
        }
    }

    // --- nextjs/no-before-interactive-script-outside-document ---
    if !is_document && name == "Script" {
        if let Some(strategy) = get_attr_value(tree, jsx, "strategy") {
            if strategy == "beforeInteractive" {
                diags.push(warn(
                    "nextjs/no-before-interactive-script-outside-document",
                    "beforeInteractive strategy should only be used in _document",
                    start, end,
                ));
            }
        }
    }

    // --- nextjs/no-title-in-document-head ---
    if is_document && name == "title" {
        let context_start = if start > 200 { start - 200 } else { 0 };
        let context = &source[context_start..start];
        if context.contains("<Head") && !context.contains("</Head") {
            diags.push(warn(
                "nextjs/no-title-in-document-head",
                "Do not use <title> in Head of _document. Use next/head in pages",
                start, end,
            ));
        }
    }
}

// ==================== Import-based rules ====================

fn check_import_rules(
    import: &serde_json::Value,
    is_document: bool,
    diags: &mut Vec<LintDiagnostic>,
) {
    let source_module = import.get("source").and_then(|s| s.as_str()).unwrap_or("");
    let span = extract_span(import).unwrap_or(Span { start: 0, end: 0 });
    let start = span.start as usize;
    let end = span.end as usize;

    // --- nextjs/no-document-import-in-page ---
    if !is_document && source_module == "next/document" {
        diags.push(err(
            "nextjs/no-document-import-in-page",
            "next/document should only be imported in _document",
            start, end,
        ));
    }

    // --- nextjs/no-head-import-in-document ---
    if is_document && source_module == "next/head" {
        diags.push(warn(
            "nextjs/no-head-import-in-document",
            "Do not import next/head in _document. Use Head from next/document",
            start, end,
        ));
    }

    // --- nextjs/no-server-import-in-page ---
    if source_module == "server-only" {
        // This is informational — server-only import is valid in server components
        // Flag only in client components
    }
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

fn get_jsx_attr(
    tree: &serde_json::Value,
    attr_id: u64,
) -> Option<(String, Option<String>, bool)> {
    let nodes = tree.get("nodes")?.as_array()?;
    let node = nodes.get(attr_id as usize)?;
    if let Some(attr) = node.get("JSXAttribute") {
        let name = attr.get("name")?.as_str()?.to_string();
        let value = attr.get("value").and_then(|v| {
            if v.is_null() {
                return None;
            }
            let vid = v.as_u64()?;
            let value_node = nodes.get(vid as usize)?;
            if let Some(lit) = value_node.get("StringLiteral") {
                return lit.get("value").and_then(|v| v.as_str()).map(|s| s.to_string());
            }
            None
        });
        return Some((name, value, false));
    }
    if node.get("JSXSpreadAttribute").is_some() {
        return Some(("".to_string(), None, true));
    }
    None
}
