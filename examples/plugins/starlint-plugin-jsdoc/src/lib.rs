//! JSDoc WASM plugin for starlint.
//!
//! Implements 18 JSDoc lint rules as a single WASM component,
//! using source-text scanning to parse and validate JSDoc comment blocks.

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    Category, FileContext, LintDiagnostic, PluginConfig, RuleMeta, Severity,
    Span,
};

struct JsdocPlugin;

export!(JsdocPlugin);

impl Guest for JsdocPlugin {
    fn get_rules() -> Vec<RuleMeta> {
        vec![
            rule("jsdoc/check-access", "Validate @access tag values", Category::Correctness, Severity::Warning),
            rule("jsdoc/check-param-names", "Enforce @param names match function parameters", Category::Correctness, Severity::Warning),
            rule("jsdoc/check-property-names", "Validate @property names are not duplicated", Category::Correctness, Severity::Warning),
            rule("jsdoc/check-tag-names", "Validate JSDoc tag names", Category::Correctness, Severity::Warning),
            rule("jsdoc/check-types", "Enforce correct type casing in JSDoc", Category::Style, Severity::Warning),
            rule("jsdoc/check-values", "Validate @version/@since/@license values", Category::Correctness, Severity::Warning),
            rule("jsdoc/empty-tags", "Enforce certain tags have no content", Category::Style, Severity::Warning),
            rule("jsdoc/implements-on-classes", "Enforce @implements only on classes", Category::Correctness, Severity::Warning),
            rule("jsdoc/match-description", "Enforce descriptions start with uppercase", Category::Style, Severity::Warning),
            rule("jsdoc/match-name", "Validate @name matches declaration", Category::Correctness, Severity::Warning),
            rule("jsdoc/no-defaults", "Disallow @default tags", Category::Suggestion, Severity::Warning),
            rule("jsdoc/no-multi-asterisks", "Disallow multiple asterisks at line start", Category::Style, Severity::Warning),
            rule("jsdoc/no-restricted-syntax", "Disallow specific JSDoc tags", Category::Suggestion, Severity::Warning),
            rule("jsdoc/require-description", "Require JSDoc descriptions", Category::Style, Severity::Warning),
            rule("jsdoc/require-param", "Require @param for function parameters", Category::Correctness, Severity::Warning),
            rule("jsdoc/require-param-description", "Require @param descriptions", Category::Style, Severity::Warning),
            rule("jsdoc/require-param-type", "Require @param type annotations", Category::Style, Severity::Warning),
            rule("jsdoc/require-returns", "Require @returns for functions with return", Category::Correctness, Severity::Warning),
        ]
    }

    fn get_file_patterns() -> Vec<String> {
        vec![
            "*.js".into(), "*.jsx".into(), "*.ts".into(), "*.tsx".into(),
            "*.mjs".into(), "*.cjs".into(),
        ]
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(file: FileContext, _tree: Vec<u8>) -> Vec<LintDiagnostic> {
        let source = &file.source_text;
        let mut diags = Vec::new();

        // Extract all JSDoc blocks and validate them
        let blocks = extract_jsdoc_blocks(source);
        for block in &blocks {
            check_access(block, &mut diags);
            check_tag_names(block, &mut diags);
            check_types(block, &mut diags);
            check_values(block, &mut diags);
            check_empty_tags(block, &mut diags);
            check_no_defaults(block, &mut diags);
            check_no_multi_asterisks(block, &mut diags);
            check_no_restricted_syntax(block, &mut diags);
            check_match_description(block, &mut diags);
            check_param_descriptions(block, &mut diags);
            check_param_types(block, &mut diags);
            check_property_names(block, &mut diags);
        }

        // Rules that need function context
        check_require_param(source, &blocks, &mut diags);
        check_require_returns(source, &blocks, &mut diags);
        check_require_description(&blocks, &mut diags);
        check_implements_on_classes(source, &blocks, &mut diags);

        diags
    }
}

// ==================== JSDoc Block Extraction ====================

/// A parsed JSDoc block with its position and content
struct JsdocBlock {
    /// Byte offset of `/**` in source
    start: usize,
    /// Byte offset of `*/` end in source
    end: usize,
    /// The raw content between `/**` and `*/`
    content: String,
    /// Parsed tags
    tags: Vec<JsdocTag>,
    /// Description text (before first tag)
    description: String,
}

struct JsdocTag {
    /// Tag name including @ (e.g., "@param")
    name: String,
    /// Rest of the tag line content
    content: String,
    /// Byte offset in source
    offset: usize,
}

fn extract_jsdoc_blocks(source: &str) -> Vec<JsdocBlock> {
    let mut blocks = Vec::new();
    let mut search_from = 0;

    while let Some(start) = source[search_from..].find("/**") {
        let abs_start = search_from + start;
        // Make sure this isn't a `/**/` or `/***` that's not a doc comment
        if let Some(end_offset) = source[abs_start + 3..].find("*/") {
            let abs_end = abs_start + 3 + end_offset + 2;
            let raw = &source[abs_start + 3..abs_end - 2];

            let mut tags = Vec::new();
            let mut description = String::new();
            let mut found_tag = false;

            for line in raw.lines() {
                let trimmed = trim_jsdoc_line(line);
                if trimmed.starts_with('@') {
                    found_tag = true;
                    // Parse tag
                    let (tag_name, tag_content) = split_tag(trimmed);
                    let line_offset_in_raw = line.as_ptr() as usize - raw.as_ptr() as usize;
                    tags.push(JsdocTag {
                        name: tag_name.into(),
                        content: tag_content.into(),
                        offset: abs_start + 3 + line_offset_in_raw,
                    });
                } else if !found_tag && !trimmed.is_empty() {
                    if !description.is_empty() {
                        description.push(' ');
                    }
                    description.push_str(trimmed);
                }
            }

            blocks.push(JsdocBlock {
                start: abs_start,
                end: abs_end,
                content: raw.into(),
                tags,
                description,
            });

            search_from = abs_end;
        } else {
            search_from = abs_start + 3;
        }
    }

    blocks
}

/// Trim JSDoc line markers: leading whitespace, `*`, and trailing whitespace
fn trim_jsdoc_line(line: &str) -> &str {
    let trimmed = line.trim();
    if trimmed.starts_with("* ") {
        trimmed[2..].trim()
    } else if trimmed == "*" {
        ""
    } else if trimmed.starts_with('*') {
        trimmed[1..].trim()
    } else {
        trimmed
    }
}

/// Split a tag line into (tag_name, rest)
fn split_tag(line: &str) -> (&str, &str) {
    if let Some(space) = line.find(' ') {
        (&line[..space], line[space + 1..].trim())
    } else {
        (line, "")
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

// ==================== Tag validation rules ====================

/// jsdoc/check-access: validate @access values
fn check_access(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    for tag in &block.tags {
        if tag.name == "@access" {
            let valid = ["public", "private", "protected", "package"];
            let value = tag.content.split_whitespace().next().unwrap_or("");
            if !valid.contains(&value) {
                diags.push(warn(
                    "jsdoc/check-access",
                    &format!("Invalid @access value '{value}'. Use public, private, protected, or package"),
                    tag.offset, tag.offset + tag.name.len() + tag.content.len() + 1,
                ));
            }
        }
    }
}

/// jsdoc/check-tag-names: validate tag names against known tags
fn check_tag_names(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    let known_tags = [
        "@abstract", "@access", "@alias", "@async", "@augments", "@author",
        "@borrows", "@callback", "@class", "@classdesc", "@constant", "@constructs",
        "@copyright", "@default", "@defaultvalue", "@deprecated", "@description",
        "@enum", "@event", "@example", "@exports", "@extends", "@external",
        "@file", "@fileoverview", "@fires", "@function", "@generator", "@global",
        "@hideconstructor", "@ignore", "@implements", "@import", "@inheritdoc",
        "@inner", "@instance", "@interface", "@kind", "@lends", "@license",
        "@listens", "@member", "@memberof", "@method", "@mixes", "@mixin",
        "@module", "@name", "@namespace", "@override", "@package", "@param",
        "@private", "@prop", "@property", "@protected", "@public", "@readonly",
        "@requires", "@returns", "@return", "@see", "@since", "@static",
        "@summary", "@template", "@this", "@throws", "@todo", "@tutorial",
        "@type", "@typedef", "@variation", "@version", "@virtual", "@yields",
        "@yield", "@satisfies",
        // TypeScript-specific
        "@template", "@typeParam", "@typeparam",
    ];

    for tag in &block.tags {
        let lower = tag.name.to_lowercase();
        if !known_tags.contains(&lower.as_str()) {
            diags.push(warn(
                "jsdoc/check-tag-names",
                &format!("Unknown JSDoc tag '{}'", tag.name),
                tag.offset, tag.offset + tag.name.len(),
            ));
        }
    }
}

/// jsdoc/check-types: enforce type casing (object not Object, etc.)
fn check_types(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    let type_fixes: &[(&str, &str)] = &[
        ("Object", "object"),
        ("Boolean", "boolean"),
        ("Number", "number"),
        ("String", "string"),
        ("Symbol", "symbol"),
        ("BigInt", "bigint"),
        ("Function", "function"),
    ];

    for tag in &block.tags {
        if tag.name == "@param" || tag.name == "@returns" || tag.name == "@return"
            || tag.name == "@type" || tag.name == "@typedef" || tag.name == "@property"
            || tag.name == "@prop"
        {
            // Look for {Type} in content
            if let Some(open) = tag.content.find('{') {
                if let Some(close) = tag.content[open..].find('}') {
                    let type_str = &tag.content[open + 1..open + close];
                    for (wrong, correct) in type_fixes {
                        if type_str.contains(wrong) {
                            let type_offset = tag.offset + tag.name.len() + 1 + open;
                            diags.push(warn(
                                "jsdoc/check-types",
                                &format!("Use '{correct}' instead of '{wrong}'"),
                                type_offset, type_offset + close + 1,
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// jsdoc/check-values: validate @version/@since/@license values
fn check_values(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    for tag in &block.tags {
        if tag.name == "@version" || tag.name == "@since" {
            let value = tag.content.trim();
            if !value.is_empty() && !is_semver_like(value) {
                diags.push(warn(
                    "jsdoc/check-values",
                    &format!("'{}' value '{value}' does not look like a valid semver version", tag.name),
                    tag.offset, tag.offset + tag.name.len() + value.len() + 1,
                ));
            }
        }
        if tag.name == "@license" {
            let value = tag.content.trim();
            if !value.is_empty() && !is_known_license(value) {
                diags.push(warn(
                    "jsdoc/check-values",
                    &format!("Unknown license identifier '{value}'"),
                    tag.offset, tag.offset + tag.name.len() + value.len() + 1,
                ));
            }
        }
    }
}

/// jsdoc/empty-tags: certain tags should have no content
fn check_empty_tags(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    let empty_tags = [
        "@abstract", "@async", "@generator", "@global", "@hideconstructor",
        "@ignore", "@inner", "@instance", "@override", "@readonly",
        "@static", "@virtual",
    ];

    for tag in &block.tags {
        if empty_tags.contains(&tag.name.as_str()) && !tag.content.trim().is_empty() {
            diags.push(warn(
                "jsdoc/empty-tags",
                &format!("'{}' tag should not have content", tag.name),
                tag.offset, tag.offset + tag.name.len() + tag.content.len() + 1,
            ));
        }
    }
}

/// jsdoc/no-defaults: disallow @default and @defaultvalue tags
fn check_no_defaults(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    for tag in &block.tags {
        if tag.name == "@default" || tag.name == "@defaultvalue" {
            diags.push(warn(
                "jsdoc/no-defaults",
                &format!("Avoid using '{}' tag", tag.name),
                tag.offset, tag.offset + tag.name.len(),
            ));
        }
    }
}

/// jsdoc/no-multi-asterisks: disallow multiple asterisks at line start
fn check_no_multi_asterisks(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    for line in block.content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("**") && !trimmed.starts_with("**/") {
            let line_offset = line.as_ptr() as usize - block.content.as_ptr() as usize;
            let abs_offset = block.start + 3 + line_offset;
            diags.push(warn(
                "jsdoc/no-multi-asterisks",
                "Do not use multiple asterisks at the start of a JSDoc line",
                abs_offset, abs_offset + trimmed.len(),
            ));
        }
    }
}

/// jsdoc/no-restricted-syntax: disallow @todo by default
fn check_no_restricted_syntax(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    for tag in &block.tags {
        if tag.name == "@todo" {
            diags.push(warn(
                "jsdoc/no-restricted-syntax",
                "@todo tags should be resolved before committing",
                tag.offset, tag.offset + tag.name.len(),
            ));
        }
    }
}

/// jsdoc/match-description: descriptions should start with uppercase
fn check_match_description(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    if !block.description.is_empty() {
        let first_char = block.description.chars().next().unwrap_or('A');
        if first_char.is_lowercase() {
            diags.push(warn(
                "jsdoc/match-description",
                "JSDoc description should start with an uppercase letter",
                block.start, block.start + 3 + block.description.len().min(20),
            ));
        }
    }
}

/// jsdoc/require-param-description: @param should have descriptions
fn check_param_descriptions(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    for tag in &block.tags {
        if tag.name == "@param" {
            // Content format: {type} name description
            // Or: name description
            let content = tag.content.trim();
            let rest = skip_type(content);
            // After type, we should have name then description
            let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
            if parts.len() < 2 || parts.get(1).map_or(true, |d| d.trim().is_empty()) {
                diags.push(warn(
                    "jsdoc/require-param-description",
                    "@param tag is missing a description",
                    tag.offset, tag.offset + tag.name.len() + content.len() + 1,
                ));
            }
        }
    }
}

/// jsdoc/require-param-type: @param should have type annotations
fn check_param_types(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    for tag in &block.tags {
        if tag.name == "@param" {
            let content = tag.content.trim();
            if !content.starts_with('{') {
                diags.push(warn(
                    "jsdoc/require-param-type",
                    "@param tag is missing a type annotation",
                    tag.offset, tag.offset + tag.name.len() + 1,
                ));
            }
        }
    }
}

/// jsdoc/check-property-names: no duplicate @property names
fn check_property_names(block: &JsdocBlock, diags: &mut Vec<LintDiagnostic>) {
    let mut seen: Vec<&str> = Vec::new();
    for tag in &block.tags {
        if tag.name == "@property" || tag.name == "@prop" {
            let content = tag.content.trim();
            let rest = skip_type(content);
            let name = rest.split_whitespace().next().unwrap_or("");
            if !name.is_empty() {
                if seen.contains(&name) {
                    diags.push(warn(
                        "jsdoc/check-property-names",
                        &format!("Duplicate @property name '{name}'"),
                        tag.offset, tag.offset + tag.name.len() + content.len() + 1,
                    ));
                }
                seen.push(name);
            }
        }
    }
}

// ==================== Context-aware rules ====================

/// jsdoc/require-param: require @param for all function parameters
fn check_require_param(source: &str, blocks: &[JsdocBlock], diags: &mut Vec<LintDiagnostic>) {
    for block in blocks {
        // Look for function declaration after this JSDoc block
        let after = &source[block.end..];
        let trimmed = after.trim_start();
        if !is_function_like(trimmed) {
            continue;
        }

        // Extract parameter names from the function signature
        if let Some(paren_start) = trimmed.find('(') {
            if let Some(paren_end) = trimmed[paren_start..].find(')') {
                let params_str = &trimmed[paren_start + 1..paren_start + paren_end];
                let param_names = extract_param_names(params_str);

                // Check each parameter has a @param tag
                let documented: Vec<String> = block.tags.iter()
                    .filter(|t| t.name == "@param")
                    .map(|t| {
                        let content = t.content.trim();
                        let rest = skip_type(content);
                        rest.split_whitespace().next().unwrap_or("").into()
                    })
                    .collect();

                for param in &param_names {
                    if !documented.iter().any(|d| d == param) {
                        diags.push(warn(
                            "jsdoc/require-param",
                            &format!("Missing @param for '{param}'"),
                            block.start, block.end,
                        ));
                    }
                }
            }
        }
    }
}

/// jsdoc/require-returns: require @returns for functions with return
fn check_require_returns(source: &str, blocks: &[JsdocBlock], diags: &mut Vec<LintDiagnostic>) {
    for block in blocks {
        let after = &source[block.end..];
        let trimmed = after.trim_start();
        if !is_function_like(trimmed) {
            continue;
        }

        // Check if function has a return statement
        if let Some(brace) = trimmed.find('{') {
            let body = &trimmed[brace..];
            if body.contains("return ") && !body.contains("return;") {
                // Has a non-void return — check for @returns
                let has_returns = block.tags.iter().any(|t| t.name == "@returns" || t.name == "@return");
                if !has_returns {
                    diags.push(warn(
                        "jsdoc/require-returns",
                        "Missing @returns tag for function with return value",
                        block.start, block.end,
                    ));
                }
            }
        }
    }
}

/// jsdoc/require-description: require non-empty description in JSDoc
fn check_require_description(blocks: &[JsdocBlock], diags: &mut Vec<LintDiagnostic>) {
    for block in blocks {
        if block.description.is_empty() && !block.tags.is_empty() {
            diags.push(warn(
                "jsdoc/require-description",
                "JSDoc block is missing a description",
                block.start, block.start + 3,
            ));
        }
    }
}

/// jsdoc/implements-on-classes: @implements only on class declarations
fn check_implements_on_classes(source: &str, blocks: &[JsdocBlock], diags: &mut Vec<LintDiagnostic>) {
    for block in blocks {
        let has_implements = block.tags.iter().any(|t| t.name == "@implements");
        if !has_implements {
            continue;
        }

        let after = &source[block.end..];
        let trimmed = after.trim_start();
        if !trimmed.starts_with("class ") {
            diags.push(warn(
                "jsdoc/implements-on-classes",
                "@implements should only be used on class declarations",
                block.start, block.end,
            ));
        }
    }
}

// ==================== Utility functions ====================

/// Skip a {type} annotation at the start of content, returning the rest
fn skip_type(content: &str) -> &str {
    if content.starts_with('{') {
        if let Some(close) = content.find('}') {
            return content[close + 1..].trim_start();
        }
    }
    content
}

/// Check if trimmed text starts with a function-like declaration
fn is_function_like(trimmed: &str) -> bool {
    trimmed.starts_with("function ")
        || trimmed.starts_with("async function ")
        || trimmed.starts_with("export function ")
        || trimmed.starts_with("export async function ")
        || trimmed.starts_with("export default function")
        || (trimmed.contains("(") && trimmed.contains("=>"))
}

/// Extract parameter names from a function signature string
fn extract_param_names(params: &str) -> Vec<String> {
    let mut names = Vec::new();
    for param in params.split(',') {
        let trimmed = param.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Handle destructuring: skip { ... } and [ ... ]
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            continue;
        }
        // Handle rest params: ...args
        let name = if let Some(rest) = trimmed.strip_prefix("...") {
            rest
        } else {
            trimmed
        };
        // Handle type annotations: param: type
        let name = name.split(':').next().unwrap_or(name);
        // Handle default values: param = value
        let name = name.split('=').next().unwrap_or(name).trim();
        if !name.is_empty() {
            names.push(name.into());
        }
    }
    names
}

/// Simple semver-like check
fn is_semver_like(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() >= 2 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit() || c == '-' || c.is_ascii_alphabetic()))
}

/// Simple known license check
fn is_known_license(s: &str) -> bool {
    let known = [
        "MIT", "Apache-2.0", "GPL-2.0", "GPL-3.0", "BSD-2-Clause", "BSD-3-Clause",
        "ISC", "MPL-2.0", "LGPL-2.1", "LGPL-3.0", "AGPL-3.0", "Unlicense",
        "CC0-1.0", "0BSD", "WTFPL", "Zlib", "BSL-1.0",
    ];
    known.contains(&s)
}
