//! Storybook WASM plugin for starlint.
//!
//! Implements all 15 `storybook/*` lint rules as a single WASM component,
//! using a mix of source-text scanning and AST node inspection.

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    AstNode, Category, LintDiagnostic, NodeBatch, NodeInterest, PluginConfig, RuleMeta, Severity,
    Span,
};

struct StorybookPlugin;

export!(StorybookPlugin);

impl Guest for StorybookPlugin {
    fn get_rules() -> Vec<RuleMeta> {
        vec![
            rule("storybook/await-interactions", "Interactions should be awaited", Category::Correctness, Severity::Warning),
            rule("storybook/context-in-play-function", "Pass context when calling play functions", Category::Correctness, Severity::Warning),
            rule("storybook/csf-component", "CSF meta should include a component property", Category::Correctness, Severity::Warning),
            rule("storybook/default-exports", "Story files should have a default export", Category::Correctness, Severity::Warning),
            rule("storybook/hierarchy-separator", "Use / instead of | as hierarchy separator", Category::Style, Severity::Warning),
            rule("storybook/meta-inline-properties", "Meta should only have inline properties", Category::Style, Severity::Warning),
            rule("storybook/meta-satisfies-type", "Meta should use satisfies for type safety", Category::Suggestion, Severity::Warning),
            rule("storybook/no-redundant-story-name", "Story name is redundant when it matches export name", Category::Style, Severity::Warning),
            rule("storybook/no-stories-of", "storiesOf is deprecated, use CSF", Category::Correctness, Severity::Error),
            rule("storybook/no-title-property-in-meta", "Do not define title in meta, use auto-title", Category::Style, Severity::Warning),
            rule("storybook/no-uninstalled-addons", "Verify storybook addons are installed", Category::Correctness, Severity::Warning),
            rule("storybook/prefer-pascal-case", "Story exports should use PascalCase", Category::Style, Severity::Warning),
            rule("storybook/story-exports", "Story files must have at least one named export", Category::Correctness, Severity::Warning),
            rule("storybook/use-storybook-expect", "Use expect from @storybook/test", Category::Correctness, Severity::Warning),
            rule("storybook/use-storybook-testing-library", "Use @storybook/test instead of @testing-library", Category::Correctness, Severity::Warning),
        ]
    }

    fn get_node_interests() -> NodeInterest {
        // SOURCE_TEXT for text-scanning rules + AST nodes for the 2 AST-based rules.
        NodeInterest::SOURCE_TEXT
            | NodeInterest::CALL_EXPRESSION
            | NodeInterest::IMPORT_DECLARATION
    }

    fn get_file_patterns() -> Vec<String> {
        vec![
            "*.stories.*".into(),
            "*.story.*".into(),
        ]
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(batch: NodeBatch) -> Vec<LintDiagnostic> {
        let source = &batch.file.source_text;
        let file_path = &batch.file.file_path;
        let ext = &batch.file.extension;
        let mut diags = Vec::new();

        // --- Text-scanning rules ---
        check_default_exports(source, &mut diags);
        check_story_exports(source, &mut diags);
        check_hierarchy_separator(source, &mut diags);
        check_csf_component(source, &mut diags);
        check_await_interactions(source, &mut diags);
        check_context_in_play_function(source, &mut diags);
        check_no_title_property_in_meta(source, &mut diags);
        check_meta_inline_properties(source, &mut diags);
        check_prefer_pascal_case(source, &mut diags);
        check_no_redundant_story_name(source, &mut diags);
        check_use_storybook_expect(source, &mut diags);
        check_no_uninstalled_addons(source, file_path, &mut diags);

        // TypeScript-only rule.
        if ext == "ts" || ext == "tsx" {
            check_meta_satisfies_type(source, &mut diags);
        }

        // --- AST-based rules ---
        for node in &batch.nodes {
            match node {
                AstNode::CallExpr(call) => {
                    // storybook/no-stories-of
                    if call.callee_path == "storiesOf" {
                        diags.push(diag(
                            "storybook/no-stories-of",
                            "`storiesOf` is deprecated — use CSF (Component Story Format) instead",
                            call.span,
                            Severity::Error,
                            None,
                        ));
                    }
                }
                AstNode::ImportDecl(import) => {
                    // storybook/use-storybook-testing-library
                    if import.source.starts_with("@testing-library/") {
                        diags.push(diag(
                            "storybook/use-storybook-testing-library",
                            "Import from `@storybook/test` instead of `@testing-library/` directly",
                            import.span,
                            Severity::Warning,
                            Some("Replace `@testing-library/` imports with `@storybook/test`".into()),
                        ));
                    }
                }
                _ => {}
            }
        }

        diags
    }
}

// --- Helpers ---

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

// --- Text-scanning rule implementations ---

fn check_default_exports(source: &str, diags: &mut Vec<LintDiagnostic>) {
    if !source.contains("export default") {
        diags.push(warn(
            "storybook/default-exports",
            "Story files should have a default export (CSF meta)",
            0, 0,
        ));
    }
}

fn check_story_exports(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let has_named = source.contains("export const ")
        || source.contains("export let ")
        || source.contains("export function ")
        || source.contains("export class ");
    let has_reexport = source.contains("export {");

    if !has_named && !has_reexport {
        diags.push(warn(
            "storybook/story-exports",
            "Story files must contain at least one named story export",
            0, 0,
        ));
    }
}

fn check_hierarchy_separator(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["title: '", "title: \"", "title:'", "title:\""];

    for pattern in &patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let value_start = abs + pattern.len();
            let quote = pattern.as_bytes()[pattern.len() - 1];

            if let Some(close) = source[value_start..].find(quote as char) {
                let title_value = &source[value_start..value_start + close];
                if title_value.contains('|') {
                    diags.push(warn(
                        "storybook/hierarchy-separator",
                        "Use `/` instead of `|` as hierarchy separator in title",
                        value_start, value_start + close,
                    ));
                }
                pos = value_start + close + 1;
            } else {
                pos = value_start;
            }
        }
    }
}

fn check_csf_component(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let Some(default_pos) = source.find("export default") else { return };
    let after = &source[default_pos..];
    let Some(brace_off) = after.find('{') else { return };
    let obj_start = default_pos + brace_off;

    if !source[obj_start..].contains("component") {
        let end = default_pos + "export default".len();
        diags.push(warn(
            "storybook/csf-component",
            "CSF meta should include a `component` property",
            default_pos, end,
        ));
    }
}

fn check_await_interactions(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["userEvent.", "within("];

    for pattern in &patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let lookback_start = abs.saturating_sub(20);
            let before = source[lookback_start..abs].trim_end();

            if !before.ends_with("await") {
                diags.push(warn(
                    "storybook/await-interactions",
                    "Interactions should be awaited in play functions",
                    abs, abs + pattern.len(),
                ));
            }
            pos = abs + 1;
        }
    }
}

fn check_context_in_play_function(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = ".play()";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        diags.push(warn(
            "storybook/context-in-play-function",
            "Pass the context argument when calling another story's play function",
            abs, abs + pattern.len(),
        ));
        pos = abs + 1;
    }
}

fn check_no_title_property_in_meta(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let Some(default_pos) = source.find("export default") else { return };
    let after = &source[default_pos..];
    let Some(brace_off) = after.find('{') else { return };
    let obj_start = default_pos + brace_off;

    // Find matching closing brace.
    let mut depth: u32 = 0;
    let mut obj_end = obj_start;
    for (i, ch) in source[obj_start..].char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                obj_end = obj_start + i;
                break;
            }
        }
    }

    let meta_body = &source[obj_start..=obj_end.min(source.len().saturating_sub(1))];

    if let Some(offset) = meta_body.find("title:").or_else(|| meta_body.find("title :")) {
        let abs = obj_start + offset;
        diags.push(warn(
            "storybook/no-title-property-in-meta",
            "Do not define a `title` in meta — use auto-title instead",
            abs, abs + "title:".len(),
        ));
    }
}

fn check_meta_inline_properties(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let Some(default_pos) = source.find("export default") else { return };
    let after = &source[default_pos..];
    let Some(brace_off) = after.find('{') else { return };
    let obj_start = default_pos + brace_off;

    let mut depth: u32 = 0;
    let mut obj_end = obj_start;
    for (i, ch) in source[obj_start..].char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                obj_end = obj_start + i;
                break;
            }
        }
    }

    let meta_body = &source[obj_start..=obj_end.min(source.len().saturating_sub(1))];

    if let Some(offset) = meta_body.find("...") {
        let abs = obj_start + offset;
        diags.push(warn(
            "storybook/meta-inline-properties",
            "Meta should only have inline properties, avoid spread syntax",
            abs, abs + 3,
        ));
    }
}

fn check_prefer_pascal_case(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["export const ", "export let "];

    for pattern in &patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let name_start = abs + pattern.len();
            let after = &source[name_start..];

            let name_end = after
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(after.len());
            let export_name = &after[..name_end];

            if !export_name.is_empty()
                && export_name != "default"
                && !export_name.starts_with("__")
                && !is_pascal_case(export_name)
            {
                diags.push(warn(
                    "storybook/prefer-pascal-case",
                    "Story exports should use PascalCase",
                    name_start, name_start + export_name.len(),
                ));
            }

            pos = abs + 1;
        }
    }
}

fn is_pascal_case(s: &str) -> bool {
    let Some(first) = s.chars().next() else { return false };
    first.is_ascii_uppercase() && !s.contains('-')
}

fn check_no_redundant_story_name(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "export const ";
    let mut pos = 0;

    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let after = &source[abs + pattern.len()..];

        let name_end = after.find([' ', '=', ':']).unwrap_or(0);
        let export_name = after[..name_end].trim();

        if !export_name.is_empty() {
            if let Some(brace_off) = after[name_end..].find('{') {
                let obj_body = &after[name_end + brace_off..];
                for quote in ['\'', '"'] {
                    let name_pat = format!("name: {quote}{export_name}{quote}");
                    if let Some(match_off) = obj_body.find(name_pat.as_str()) {
                        let abs_name = abs + pattern.len() + name_end + brace_off + match_off;
                        diags.push(warn(
                            "storybook/no-redundant-story-name",
                            "Story name property is redundant when it matches the export name",
                            abs_name, abs_name + name_pat.len(),
                        ));
                    }
                }
            }
        }

        pos = abs + 1;
    }
}

fn check_meta_satisfies_type(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let Some(default_pos) = source.find("export default") else { return };
    let after = &source[default_pos..];

    if !after.contains("satisfies") {
        let end = default_pos + "export default".len();
        diags.push(warn(
            "storybook/meta-satisfies-type",
            "Meta should use `satisfies Meta` for type safety",
            default_pos, end,
        ));
    }
}

fn check_use_storybook_expect(source: &str, diags: &mut Vec<LintDiagnostic>) {
    if !source.contains("expect(") {
        return;
    }

    let has_storybook_expect = source.contains("@storybook/test")
        || source.contains("@storybook/jest")
        || source.contains("@storybook/expect");

    if !has_storybook_expect {
        if let Some(pos) = source.find("expect(") {
            diags.push(warn(
                "storybook/use-storybook-expect",
                "Import `expect` from `@storybook/test` instead of using generic `expect`",
                pos, pos + "expect(".len(),
            ));
        }
    }
}

fn check_no_uninstalled_addons(source: &str, file_path: &str, diags: &mut Vec<LintDiagnostic>) {
    // Only applies to storybook config files.
    let is_config = file_path.contains(".storybook") && file_path.contains("main");
    if !is_config || !source.contains("addons") {
        return;
    }

    let prefixes = ["@storybook/addon-", "storybook-addon-"];
    for prefix in &prefixes {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(prefix) {
            let abs = pos + found;
            let remaining = &source[abs..];
            let addon_end = remaining.find(['\'', '"', '`']).unwrap_or(prefix.len());
            let addon_name_len = addon_end;

            diags.push(warn(
                "storybook/no-uninstalled-addons",
                "Verify that this storybook addon is installed in your dependencies",
                abs, abs + addon_name_len,
            ));
            pos = abs + 1;
        }
    }
}
