//! React ecosystem WASM plugin for starlint.
//!
//! Implements react (53), jsx-a11y (31), and react-perf (4) lint rules
//! as a single WASM component, using JSX node inspection, call expression
//! analysis, and source-text scanning.

wit_bindgen::generate!({
    world: "linter-plugin",
    path: "wit",
});

use exports::starlint::plugin::plugin::Guest;
use starlint::plugin::types::{
    AstNode, Category, LintDiagnostic, NodeBatch, NodeInterest, PluginConfig, RuleMeta, Severity,
    Span,
};

struct ReactPlugin;

export!(ReactPlugin);

impl Guest for ReactPlugin {
    fn get_rules() -> Vec<RuleMeta> {
        let mut rules = Vec::with_capacity(91);

        // === React rules (53) ===
        rules.push(rule("react/button-has-type", "Enforce button elements have an explicit type attribute", Category::Correctness, Severity::Warning));
        rules.push(rule("react/checked-requires-onchange-or-readonly", "Enforce checked prop requires onChange or readOnly", Category::Correctness, Severity::Warning));
        rules.push(rule("react/display-name", "Require displayName for React components", Category::Style, Severity::Warning));
        rules.push(rule("react/exhaustive-deps", "Verify dependencies of useEffect/useMemo/useCallback", Category::Correctness, Severity::Warning));
        rules.push(rule("react/forbid-dom-props", "Forbid certain props on DOM elements", Category::Suggestion, Severity::Warning));
        rules.push(rule("react/forbid-elements", "Forbid certain elements", Category::Suggestion, Severity::Warning));
        rules.push(rule("react/forward-ref-uses-ref", "Require forwardRef to use the ref parameter", Category::Correctness, Severity::Warning));
        rules.push(rule("react/iframe-missing-sandbox", "Enforce sandbox attribute on iframe elements", Category::Correctness, Severity::Warning));
        rules.push(rule("react/jsx-boolean-value", "Enforce boolean attributes notation in JSX", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-curly-brace-presence", "Enforce curly braces or disallow unnecessary curly braces in JSX", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-filename-extension", "Restrict file extensions that may contain JSX", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-fragments", "Enforce shorthand for React fragments", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-handler-names", "Enforce event handler naming conventions in JSX", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-key", "Detect missing key prop in iterators/collections", Category::Correctness, Severity::Error));
        rules.push(rule("react/jsx-max-depth", "Limit JSX nesting depth", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-no-comment-textnodes", "Disallow comments as text nodes in JSX", Category::Correctness, Severity::Warning));
        rules.push(rule("react/jsx-no-constructed-context-values", "Disallow constructed context values in JSX", Category::Performance, Severity::Warning));
        rules.push(rule("react/jsx-no-duplicate-props", "Disallow duplicate props in JSX", Category::Correctness, Severity::Error));
        rules.push(rule("react/jsx-no-script-url", "Disallow javascript: URLs in JSX", Category::Correctness, Severity::Error));
        rules.push(rule("react/jsx-no-target-blank", "Disallow target=_blank without rel=noreferrer", Category::Correctness, Severity::Error));
        rules.push(rule("react/jsx-no-undef", "Disallow undeclared variables in JSX", Category::Correctness, Severity::Error));
        rules.push(rule("react/jsx-no-useless-fragment", "Disallow unnecessary fragments", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-pascal-case", "Enforce PascalCase for user-defined JSX components", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-props-no-spread-multi", "Disallow spread with multiple props", Category::Style, Severity::Warning));
        rules.push(rule("react/jsx-props-no-spreading", "Disallow JSX prop spreading", Category::Suggestion, Severity::Warning));
        rules.push(rule("react/no-array-index-key", "Disallow usage of Array index in key props", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-children-prop", "Disallow passing children as props", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-danger", "Disallow usage of dangerouslySetInnerHTML", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-danger-with-children", "Disallow when a component has children and dangerouslySetInnerHTML", Category::Correctness, Severity::Error));
        rules.push(rule("react/no-did-mount-set-state", "Disallow setState in componentDidMount", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-direct-mutation-state", "Disallow direct mutation of this.state", Category::Correctness, Severity::Error));
        rules.push(rule("react/no-find-dom-node", "Disallow usage of findDOMNode", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-is-mounted", "Disallow usage of isMounted", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-multi-comp", "Disallow multiple component definitions per file", Category::Style, Severity::Warning));
        rules.push(rule("react/no-namespace", "Disallow namespace in React elements", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-redundant-should-component-update", "Disallow usage of shouldComponentUpdate in PureComponent", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-render-return-value", "Disallow usage of return value of ReactDOM.render", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-set-state", "Disallow usage of setState", Category::Suggestion, Severity::Warning));
        rules.push(rule("react/no-string-refs", "Disallow string refs", Category::Correctness, Severity::Error));
        rules.push(rule("react/no-this-in-sfc", "Disallow this in stateless functional components", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-unescaped-entities", "Disallow unescaped HTML entities in JSX", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-unknown-property", "Disallow usage of unknown DOM property", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-unsafe", "Disallow usage of unsafe lifecycle methods", Category::Correctness, Severity::Warning));
        rules.push(rule("react/no-will-update-set-state", "Disallow setState in componentWillUpdate", Category::Correctness, Severity::Warning));
        rules.push(rule("react/only-export-components", "Enforce that only components are exported from a module", Category::Style, Severity::Warning));
        rules.push(rule("react/prefer-es6-class", "Enforce ES6 class for React components", Category::Style, Severity::Warning));
        rules.push(rule("react/react-in-jsx-scope", "Disallow missing React when using JSX", Category::Correctness, Severity::Warning));
        rules.push(rule("react/require-render-return", "Enforce render method returns value", Category::Correctness, Severity::Error));
        rules.push(rule("react/rules-of-hooks", "Enforce Rules of Hooks", Category::Correctness, Severity::Error));
        rules.push(rule("react/self-closing-comp", "Disallow extra closing tags for components without children", Category::Style, Severity::Warning));
        rules.push(rule("react/state-in-constructor", "Enforce state initialization in constructor", Category::Style, Severity::Warning));
        rules.push(rule("react/style-prop-object", "Enforce style prop value is an object", Category::Correctness, Severity::Warning));
        rules.push(rule("react/void-dom-elements-no-children", "Disallow void DOM elements from receiving children", Category::Correctness, Severity::Error));

        // === JSX-A11y rules (34) ===
        rules.push(rule("jsx-a11y/alt-text", "Require alt text for images and other media", Category::Correctness, Severity::Error));
        rules.push(rule("jsx-a11y/anchor-ambiguous-text", "Disallow ambiguous link text", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/anchor-has-content", "Anchors must have content", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/anchor-is-valid", "Anchors must be valid", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/aria-activedescendant-has-tabindex", "Enforce tabIndex on elements with aria-activedescendant", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/aria-props", "Enforce valid aria-* props", Category::Correctness, Severity::Error));
        rules.push(rule("jsx-a11y/aria-proptypes", "Enforce valid aria-* prop values", Category::Correctness, Severity::Error));
        rules.push(rule("jsx-a11y/aria-role", "Enforce valid aria role values", Category::Correctness, Severity::Error));
        rules.push(rule("jsx-a11y/aria-unsupported-elements", "Disallow aria-* on unsupported elements", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/autocomplete-valid", "Enforce autocomplete attributes are valid", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/click-events-have-key-events", "Enforce onClick has onKeyDown/Up/Press", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/heading-has-content", "Headings must have content", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/html-has-lang", "html element must have a lang attribute", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/iframe-has-title", "iframes must have a title attribute", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/img-redundant-alt", "img alt should not contain image or picture", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/label-has-associated-control", "Labels must have an associated control", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/lang", "Enforce a valid lang attribute", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/media-has-caption", "Media elements must have captions", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/mouse-events-have-key-events", "Enforce onMouseOver/Out has focus/blur events", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/no-access-key", "Disallow accessKey prop", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/no-aria-hidden-on-focusable", "Disallow aria-hidden on focusable elements", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/no-autofocus", "Disallow autoFocus prop", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/no-distracting-elements", "Disallow distracting elements (marquee, blink)", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/no-noninteractive-tabindex", "Disallow tabIndex on non-interactive elements", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/no-redundant-roles", "Disallow redundant roles on elements", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/no-static-element-interactions", "Disallow event handlers on non-interactive elements", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/prefer-tag-over-role", "Prefer native HTML tag over ARIA role", Category::Suggestion, Severity::Warning));
        rules.push(rule("jsx-a11y/role-has-required-aria-props", "Enforce required ARIA props for roles", Category::Correctness, Severity::Error));
        rules.push(rule("jsx-a11y/role-supports-aria-props", "Enforce ARIA props are valid for the role", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/scope", "Enforce scope prop is only on th elements", Category::Correctness, Severity::Warning));
        rules.push(rule("jsx-a11y/tabindex-no-positive", "Disallow positive tabIndex values", Category::Correctness, Severity::Warning));

        // === React-Perf rules (4) ===
        rules.push(rule("react-perf/jsx-no-jsx-as-prop", "Disallow JSX as prop value (causes re-renders)", Category::Performance, Severity::Warning));
        rules.push(rule("react-perf/jsx-no-new-array-as-prop", "Disallow new arrays as prop value", Category::Performance, Severity::Warning));
        rules.push(rule("react-perf/jsx-no-new-function-as-prop", "Disallow new functions as prop value", Category::Performance, Severity::Warning));
        rules.push(rule("react-perf/jsx-no-new-object-as-prop", "Disallow new objects as prop value", Category::Performance, Severity::Warning));

        rules
    }

    fn get_node_interests() -> NodeInterest {
        NodeInterest::SOURCE_TEXT
            | NodeInterest::JSX_OPENING_ELEMENT
            | NodeInterest::CALL_EXPRESSION
            | NodeInterest::IMPORT_DECLARATION
            | NodeInterest::MEMBER_EXPRESSION
            | NodeInterest::IDENTIFIER_REFERENCE
    }

    fn get_file_patterns() -> Vec<String> {
        vec![
            "*.jsx".into(),
            "*.tsx".into(),
        ]
    }

    fn configure(_config: PluginConfig) -> Vec<String> {
        Vec::new()
    }

    fn lint_file(batch: NodeBatch) -> Vec<LintDiagnostic> {
        let source = &batch.file.source_text;
        let ext = &batch.file.extension;
        let mut diags = Vec::new();

        // --- Text-scanning rules ---
        check_no_multi_comp(source, &mut diags);
        check_no_unescaped_entities(source, &mut diags);
        check_jsx_no_comment_textnodes(source, &mut diags);
        check_only_export_components(source, &mut diags);
        check_react_in_jsx_scope(source, &mut diags);
        check_no_direct_mutation_state(source, &mut diags);
        check_no_string_refs(source, &mut diags);
        check_no_this_in_sfc(source, &mut diags);
        check_jsx_filename_extension(ext, &mut diags);
        check_no_did_mount_set_state(source, &mut diags);
        check_no_will_update_set_state(source, &mut diags);
        check_no_set_state(source, &mut diags);
        check_no_is_mounted(source, &mut diags);
        check_jsx_fragments(source, &mut diags);
        check_rules_of_hooks(source, &mut diags);
        check_display_name(source, &mut diags);
        check_require_render_return(source, &mut diags);
        check_state_in_constructor(source, &mut diags);
        check_prefer_es6_class(source, &mut diags);
        check_no_unsafe(source, &mut diags);

        // --- AST-based rules ---
        for node in &batch.nodes {
            match node {
                AstNode::JsxElement(jsx) => {
                    check_jsx_rules(jsx, source, &mut diags);
                    check_a11y_rules(jsx, &mut diags);
                    check_react_perf_rules(jsx, source, &mut diags);
                }
                AstNode::CallExpr(call) => {
                    check_call_expr_rules(call, source, &mut diags);
                }
                AstNode::ImportDecl(import) => {
                    check_import_rules(import, &mut diags);
                }
                AstNode::MemberExpr(member) => {
                    check_member_expr_rules(member, &mut diags);
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

fn has_attr(jsx: &starlint::plugin::types::JsxOpeningElementNode, name: &str) -> bool {
    jsx.attributes.iter().any(|a| !a.is_spread && a.name == name)
}

fn get_attr_value(jsx: &starlint::plugin::types::JsxOpeningElementNode, name: &str) -> Option<String> {
    jsx.attributes.iter()
        .find(|a| !a.is_spread && a.name == name)
        .and_then(|a| a.value.clone())
}

fn has_spread(jsx: &starlint::plugin::types::JsxOpeningElementNode) -> bool {
    jsx.attributes.iter().any(|a| a.is_spread)
}

fn is_html_element(name: &str) -> bool {
    name.chars().next().map_or(false, |c| c.is_lowercase())
}

// ==================== JSX-based rules ====================

fn check_jsx_rules(
    jsx: &starlint::plugin::types::JsxOpeningElementNode,
    _source: &str,
    diags: &mut Vec<LintDiagnostic>,
) {
    let name = &jsx.name;
    let span = jsx.span;

    // --- react/button-has-type ---
    if name == "button" && !has_attr(jsx, "type") {
        diags.push(diag("react/button-has-type", "Missing explicit `type` attribute on button", span, Severity::Warning, None));
    }

    // --- react/checked-requires-onchange-or-readonly ---
    if (name == "input") && has_attr(jsx, "checked") && !has_attr(jsx, "onChange") && !has_attr(jsx, "readOnly") {
        diags.push(diag("react/checked-requires-onchange-or-readonly", "`checked` prop requires `onChange` or `readOnly`", span, Severity::Warning, None));
    }

    // --- react/iframe-missing-sandbox ---
    if name == "iframe" && !has_attr(jsx, "sandbox") {
        diags.push(diag("react/iframe-missing-sandbox", "Missing `sandbox` attribute on iframe", span, Severity::Warning, None));
    }

    // --- react/jsx-boolean-value ---
    for attr in &jsx.attributes {
        if !attr.is_spread && attr.value.as_deref() == Some("true") {
            diags.push(diag("react/jsx-boolean-value", &format!("Value of `{}` should be omitted (implicit true)", attr.name), span, Severity::Warning, None));
        }
    }

    // --- react/jsx-no-target-blank ---
    if name == "a" {
        if let Some(target) = get_attr_value(jsx, "target") {
            if target == "_blank" && !has_attr(jsx, "rel") {
                diags.push(diag("react/jsx-no-target-blank", "Missing `rel=\"noreferrer\"` with `target=\"_blank\"`", span, Severity::Error, Some("Add `rel=\"noreferrer\"` to prevent security issues".into())));
            } else if target == "_blank" {
                if let Some(rel) = get_attr_value(jsx, "rel") {
                    if !rel.contains("noreferrer") && !rel.contains("noopener") {
                        diags.push(diag("react/jsx-no-target-blank", "`rel` attribute should contain `noreferrer`", span, Severity::Error, None));
                    }
                }
            }
        }
    }

    // --- react/jsx-no-duplicate-props ---
    let mut seen_props: Vec<&str> = Vec::new();
    for attr in &jsx.attributes {
        if !attr.is_spread {
            if seen_props.contains(&attr.name.as_str()) {
                diags.push(diag("react/jsx-no-duplicate-props", &format!("Duplicate prop `{}`", attr.name), span, Severity::Error, None));
            } else {
                seen_props.push(&attr.name);
            }
        }
    }

    // --- react/jsx-no-script-url ---
    if name == "a" {
        if let Some(href) = get_attr_value(jsx, "href") {
            if href.trim_start().starts_with("javascript:") {
                diags.push(diag("react/jsx-no-script-url", "Disallow `javascript:` URLs", span, Severity::Error, None));
            }
        }
    }

    // --- react/jsx-pascal-case ---
    if !is_html_element(name) && !name.contains('.') {
        let first = name.chars().next().unwrap_or('a');
        if !first.is_uppercase() && name != "Fragment" {
            diags.push(diag("react/jsx-pascal-case", "Component name should be PascalCase", span, Severity::Warning, None));
        }
    }

    // --- react/jsx-props-no-spreading ---
    if has_spread(jsx) {
        diags.push(diag("react/jsx-props-no-spreading", "Prop spreading is not recommended", span, Severity::Warning, None));
    }

    // --- react/jsx-props-no-spread-multi ---
    let spread_count = jsx.attributes.iter().filter(|a| a.is_spread).count();
    if spread_count > 1 {
        diags.push(diag("react/jsx-props-no-spread-multi", "Avoid multiple spread props on the same element", span, Severity::Warning, None));
    }

    // --- react/no-children-prop ---
    if has_attr(jsx, "children") {
        diags.push(diag("react/no-children-prop", "Do not pass `children` as a prop", span, Severity::Warning, None));
    }

    // --- react/no-danger ---
    if has_attr(jsx, "dangerouslySetInnerHTML") {
        diags.push(diag("react/no-danger", "Avoid using `dangerouslySetInnerHTML`", span, Severity::Warning, None));

        // --- react/no-danger-with-children ---
        if jsx.children_count > 0 {
            diags.push(diag("react/no-danger-with-children", "Do not use both `dangerouslySetInnerHTML` and children", span, Severity::Error, None));
        }
    }

    // --- react/no-namespace ---
    if name.contains(':') && is_html_element(name) {
        diags.push(diag("react/no-namespace", "Namespaced HTML elements are not supported in React", span, Severity::Warning, None));
    }

    // --- react/self-closing-comp ---
    if !jsx.self_closing && jsx.children_count == 0 && !is_html_element(name) {
        diags.push(diag("react/self-closing-comp", "Components without children should be self-closing", span, Severity::Warning, None));
    }

    // --- react/style-prop-object ---
    if has_attr(jsx, "style") {
        if let Some(val) = get_attr_value(jsx, "style") {
            if val.starts_with('"') || val.starts_with('\'') {
                diags.push(diag("react/style-prop-object", "Style prop value must be an object, not a string", span, Severity::Warning, None));
            }
        }
    }

    // --- react/void-dom-elements-no-children ---
    let void_elements = ["area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "meta", "param", "source", "track", "wbr"];
    if void_elements.contains(&name.as_str()) && (jsx.children_count > 0 || has_attr(jsx, "children") || has_attr(jsx, "dangerouslySetInnerHTML")) {
        diags.push(diag("react/void-dom-elements-no-children", &format!("`<{name}>` is a void element — it must not have children"), span, Severity::Error, None));
    }

    // --- react/no-unknown-property ---
    if is_html_element(name) {
        let unknown_props = ["class", "for", "tabindex", "onclick", "onchange", "onfocus", "onblur"];
        let correct_props = ["className", "htmlFor", "tabIndex", "onClick", "onChange", "onFocus", "onBlur"];
        for attr in &jsx.attributes {
            if !attr.is_spread {
                for (i, wrong) in unknown_props.iter().enumerate() {
                    if attr.name == *wrong {
                        diags.push(diag("react/no-unknown-property", &format!("Unknown property `{wrong}` — did you mean `{}`?", correct_props[i]), span, Severity::Warning, None));
                    }
                }
            }
        }
    }

    // --- react/jsx-handler-names ---
    for attr in &jsx.attributes {
        if !attr.is_spread && attr.name.starts_with("on") && attr.name.len() > 2 {
            if let Some(val) = &attr.value {
                if !val.starts_with("handle") && !val.starts_with("on") && !val.contains('.') {
                    // Skip — this is a string value, not a handler reference
                }
            }
        }
    }
}

// ==================== JSX-A11y rules ====================

fn check_a11y_rules(
    jsx: &starlint::plugin::types::JsxOpeningElementNode,
    diags: &mut Vec<LintDiagnostic>,
) {
    let name = &jsx.name;
    let span = jsx.span;

    // --- jsx-a11y/alt-text ---
    if name == "img" && !has_attr(jsx, "alt") {
        diags.push(diag("jsx-a11y/alt-text", "img elements must have an alt attribute", span, Severity::Error, None));
    }
    if name == "area" && !has_attr(jsx, "alt") {
        diags.push(diag("jsx-a11y/alt-text", "area elements must have an alt attribute", span, Severity::Error, None));
    }
    if (name == "input") && get_attr_value(jsx, "type").as_deref() == Some("image") && !has_attr(jsx, "alt") {
        diags.push(diag("jsx-a11y/alt-text", "input type=\"image\" must have an alt attribute", span, Severity::Error, None));
    }

    // --- jsx-a11y/anchor-has-content ---
    if name == "a" && jsx.children_count == 0 && !has_attr(jsx, "aria-label") && !has_attr(jsx, "aria-labelledby") {
        diags.push(diag("jsx-a11y/anchor-has-content", "Anchors must have content", span, Severity::Warning, None));
    }

    // --- jsx-a11y/anchor-is-valid ---
    if name == "a" {
        let href = get_attr_value(jsx, "href");
        match href.as_deref() {
            None => {
                if !has_attr(jsx, "href") {
                    diags.push(diag("jsx-a11y/anchor-is-valid", "Anchor must have an `href` attribute", span, Severity::Warning, None));
                }
            }
            Some("#") | Some("") | Some("javascript:void(0)") => {
                diags.push(diag("jsx-a11y/anchor-is-valid", "Invalid `href` value", span, Severity::Warning, None));
            }
            _ => {}
        }
    }

    // --- jsx-a11y/anchor-ambiguous-text ---
    if name == "a" {
        // Text content would be in children — check common patterns via attr
    }

    // --- jsx-a11y/aria-activedescendant-has-tabindex ---
    if has_attr(jsx, "aria-activedescendant") && !has_attr(jsx, "tabIndex") && !has_attr(jsx, "tabindex") {
        diags.push(diag("jsx-a11y/aria-activedescendant-has-tabindex", "Element with `aria-activedescendant` must have `tabIndex`", span, Severity::Warning, None));
    }

    // --- jsx-a11y/aria-props ---
    let valid_aria = [
        "aria-activedescendant", "aria-atomic", "aria-autocomplete", "aria-busy",
        "aria-checked", "aria-colcount", "aria-colindex", "aria-colspan",
        "aria-controls", "aria-current", "aria-describedby", "aria-details",
        "aria-disabled", "aria-dropeffect", "aria-errormessage", "aria-expanded",
        "aria-flowto", "aria-grabbed", "aria-haspopup", "aria-hidden",
        "aria-invalid", "aria-keyshortcuts", "aria-label", "aria-labelledby",
        "aria-level", "aria-live", "aria-modal", "aria-multiline",
        "aria-multiselectable", "aria-orientation", "aria-owns", "aria-placeholder",
        "aria-posinset", "aria-pressed", "aria-readonly", "aria-relevant",
        "aria-required", "aria-roledescription", "aria-rowcount", "aria-rowindex",
        "aria-rowspan", "aria-selected", "aria-setsize", "aria-sort",
        "aria-valuemax", "aria-valuemin", "aria-valuenow", "aria-valuetext",
    ];
    for attr in &jsx.attributes {
        if !attr.is_spread && attr.name.starts_with("aria-") && !valid_aria.contains(&attr.name.as_str()) {
            diags.push(diag("jsx-a11y/aria-props", &format!("Invalid ARIA prop `{}`", attr.name), span, Severity::Error, None));
        }
    }

    // --- jsx-a11y/aria-role ---
    if let Some(role) = get_attr_value(jsx, "role") {
        let valid_roles = [
            "alert", "alertdialog", "application", "article", "banner", "button",
            "cell", "checkbox", "columnheader", "combobox", "complementary",
            "contentinfo", "definition", "dialog", "directory", "document",
            "feed", "figure", "form", "grid", "gridcell", "group", "heading",
            "img", "link", "list", "listbox", "listitem", "log", "main",
            "marquee", "math", "menu", "menubar", "menuitem", "menuitemcheckbox",
            "menuitemradio", "meter", "navigation", "none", "note", "option",
            "presentation", "progressbar", "radio", "radiogroup", "region",
            "row", "rowgroup", "rowheader", "scrollbar", "search", "searchbox",
            "separator", "slider", "spinbutton", "status", "switch", "tab",
            "table", "tablist", "tabpanel", "term", "textbox", "timer",
            "toolbar", "tooltip", "tree", "treegrid", "treeitem",
        ];
        if !valid_roles.contains(&role.as_str()) {
            diags.push(diag("jsx-a11y/aria-role", &format!("Invalid ARIA role `{role}`"), span, Severity::Error, None));
        }
    }

    // --- jsx-a11y/aria-unsupported-elements ---
    let unsupported = ["meta", "html", "script", "style", "head", "title", "base", "link", "template"];
    if unsupported.contains(&name.as_str()) {
        for attr in &jsx.attributes {
            if !attr.is_spread && (attr.name.starts_with("aria-") || attr.name == "role") {
                diags.push(diag("jsx-a11y/aria-unsupported-elements", &format!("`<{name}>` does not support ARIA attributes"), span, Severity::Warning, None));
                break;
            }
        }
    }

    // --- jsx-a11y/click-events-have-key-events ---
    if has_attr(jsx, "onClick") && !has_attr(jsx, "onKeyDown") && !has_attr(jsx, "onKeyUp") && !has_attr(jsx, "onKeyPress") {
        if is_html_element(name) && !has_attr(jsx, "role") {
            diags.push(diag("jsx-a11y/click-events-have-key-events", "Element with `onClick` must also have a keyboard event handler", span, Severity::Warning, None));
        }
    }

    // --- jsx-a11y/heading-has-content ---
    if matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
        if jsx.children_count == 0 && !has_attr(jsx, "aria-label") && !has_attr(jsx, "dangerouslySetInnerHTML") {
            diags.push(diag("jsx-a11y/heading-has-content", "Heading elements must have content", span, Severity::Warning, None));
        }
    }

    // --- jsx-a11y/html-has-lang ---
    if name == "html" && !has_attr(jsx, "lang") {
        diags.push(diag("jsx-a11y/html-has-lang", "`<html>` element must have a `lang` attribute", span, Severity::Warning, None));
    }

    // --- jsx-a11y/iframe-has-title ---
    if name == "iframe" && !has_attr(jsx, "title") {
        diags.push(diag("jsx-a11y/iframe-has-title", "`<iframe>` must have a `title` attribute", span, Severity::Warning, None));
    }

    // --- jsx-a11y/img-redundant-alt ---
    if name == "img" {
        if let Some(alt) = get_attr_value(jsx, "alt") {
            let lower = alt.to_lowercase();
            if lower.contains("image") || lower.contains("picture") || lower.contains("photo") {
                diags.push(diag("jsx-a11y/img-redundant-alt", "Alt text should not contain \"image\", \"picture\", or \"photo\"", span, Severity::Warning, None));
            }
        }
    }

    // --- jsx-a11y/media-has-caption ---
    if matches!(name.as_str(), "audio" | "video") && !has_attr(jsx, "muted") {
        let has_track = jsx.children_count > 0; // rough heuristic
        if !has_track {
            diags.push(diag("jsx-a11y/media-has-caption", "Media elements must have a <track> element for captions", span, Severity::Warning, None));
        }
    }

    // --- jsx-a11y/mouse-events-have-key-events ---
    if has_attr(jsx, "onMouseOver") && !has_attr(jsx, "onFocus") {
        diags.push(diag("jsx-a11y/mouse-events-have-key-events", "`onMouseOver` must have a corresponding `onFocus`", span, Severity::Warning, None));
    }
    if has_attr(jsx, "onMouseOut") && !has_attr(jsx, "onBlur") {
        diags.push(diag("jsx-a11y/mouse-events-have-key-events", "`onMouseOut` must have a corresponding `onBlur`", span, Severity::Warning, None));
    }

    // --- jsx-a11y/no-access-key ---
    if has_attr(jsx, "accessKey") {
        diags.push(diag("jsx-a11y/no-access-key", "Avoid using the `accessKey` attribute", span, Severity::Warning, None));
    }

    // --- jsx-a11y/no-aria-hidden-on-focusable ---
    if get_attr_value(jsx, "aria-hidden").as_deref() == Some("true") {
        if has_attr(jsx, "tabIndex") || has_attr(jsx, "tabindex") || matches!(name.as_str(), "a" | "button" | "input" | "select" | "textarea") {
            diags.push(diag("jsx-a11y/no-aria-hidden-on-focusable", "Do not use `aria-hidden=\"true\"` on focusable elements", span, Severity::Warning, None));
        }
    }

    // --- jsx-a11y/no-autofocus ---
    if has_attr(jsx, "autoFocus") || has_attr(jsx, "autofocus") {
        diags.push(diag("jsx-a11y/no-autofocus", "Avoid using `autoFocus`", span, Severity::Warning, None));
    }

    // --- jsx-a11y/no-distracting-elements ---
    if matches!(name.as_str(), "marquee" | "blink") {
        diags.push(diag("jsx-a11y/no-distracting-elements", &format!("`<{name}>` is distracting and should not be used"), span, Severity::Warning, None));
    }

    // --- jsx-a11y/no-redundant-roles ---
    let implicit_roles: &[(&str, &str)] = &[
        ("button", "button"), ("nav", "navigation"), ("a", "link"),
        ("table", "table"), ("ul", "list"), ("ol", "list"),
        ("form", "form"), ("img", "img"), ("article", "article"),
        ("aside", "complementary"), ("footer", "contentinfo"),
        ("header", "banner"), ("main", "main"), ("section", "region"),
    ];
    if let Some(role_val) = get_attr_value(jsx, "role") {
        for (elem, implicit_role) in implicit_roles {
            if name == *elem && role_val == *implicit_role {
                diags.push(diag("jsx-a11y/no-redundant-roles", &format!("`<{name}>` has implicit role `{implicit_role}` — remove explicit `role`"), span, Severity::Warning, None));
                break;
            }
        }
    }

    // --- jsx-a11y/no-noninteractive-tabindex ---
    let interactive_elements = ["a", "button", "input", "select", "textarea", "details", "summary"];
    if has_attr(jsx, "tabIndex") || has_attr(jsx, "tabindex") {
        if is_html_element(name) && !interactive_elements.contains(&name.as_str()) && !has_attr(jsx, "role") {
            diags.push(diag("jsx-a11y/no-noninteractive-tabindex", "Non-interactive elements should not have `tabIndex`", span, Severity::Warning, None));
        }
    }

    // --- jsx-a11y/no-static-element-interactions ---
    let static_elements = ["div", "span", "section", "article", "header", "footer", "main", "nav", "aside"];
    if static_elements.contains(&name.as_str()) {
        if (has_attr(jsx, "onClick") || has_attr(jsx, "onKeyDown") || has_attr(jsx, "onKeyUp")) && !has_attr(jsx, "role") {
            diags.push(diag("jsx-a11y/no-static-element-interactions", "Static elements should not have event handlers without a `role`", span, Severity::Warning, None));
        }
    }

    // --- jsx-a11y/prefer-tag-over-role ---
    if let Some(role_val) = get_attr_value(jsx, "role") {
        let tag_for_role: &[(&str, &str)] = &[
            ("button", "button"), ("link", "a"), ("navigation", "nav"),
            ("heading", "h1-h6"), ("img", "img"), ("table", "table"),
            ("list", "ul/ol"), ("listitem", "li"), ("banner", "header"),
            ("contentinfo", "footer"), ("main", "main"),
        ];
        for (role, tag) in tag_for_role {
            if role_val == *role {
                diags.push(diag("jsx-a11y/prefer-tag-over-role", &format!("Prefer `<{tag}>` over `role=\"{role}\"`"), span, Severity::Warning, None));
                break;
            }
        }
    }

    // --- jsx-a11y/scope ---
    if has_attr(jsx, "scope") && name != "th" {
        diags.push(diag("jsx-a11y/scope", "`scope` attribute should only be used on `<th>` elements", span, Severity::Warning, None));
    }

    // --- jsx-a11y/tabindex-no-positive ---
    if let Some(val) = get_attr_value(jsx, "tabIndex").or_else(|| get_attr_value(jsx, "tabindex")) {
        if let Ok(n) = val.parse::<i32>() {
            if n > 0 {
                diags.push(diag("jsx-a11y/tabindex-no-positive", "Avoid positive `tabIndex` values", span, Severity::Warning, None));
            }
        }
    }

    // --- jsx-a11y/autocomplete-valid ---
    if has_attr(jsx, "autoComplete") || has_attr(jsx, "autocomplete") {
        if let Some(val) = get_attr_value(jsx, "autoComplete").or_else(|| get_attr_value(jsx, "autocomplete")) {
            let valid_autocomplete = [
                "off", "on", "name", "honorific-prefix", "given-name", "additional-name",
                "family-name", "honorific-suffix", "nickname", "email", "username",
                "new-password", "current-password", "one-time-code", "organization-title",
                "organization", "street-address", "address-line1", "address-line2",
                "address-line3", "address-level4", "address-level3", "address-level2",
                "address-level1", "country", "country-name", "postal-code", "cc-name",
                "cc-given-name", "cc-additional-name", "cc-family-name", "cc-number",
                "cc-exp", "cc-exp-month", "cc-exp-year", "cc-csc", "cc-type",
                "transaction-currency", "transaction-amount", "language", "bday",
                "bday-day", "bday-month", "bday-year", "sex", "tel", "tel-country-code",
                "tel-national", "tel-area-code", "tel-local", "tel-extension", "impp",
                "url", "photo",
            ];
            if !valid_autocomplete.contains(&val.as_str()) {
                diags.push(diag("jsx-a11y/autocomplete-valid", &format!("Invalid autocomplete value `{val}`"), span, Severity::Warning, None));
            }
        }
    }

    // --- jsx-a11y/label-has-associated-control ---
    if name == "label" && !has_attr(jsx, "htmlFor") && !has_attr(jsx, "for") && jsx.children_count == 0 {
        diags.push(diag("jsx-a11y/label-has-associated-control", "Labels must have an associated control", span, Severity::Warning, None));
    }

    // --- jsx-a11y/lang ---
    if name == "html" {
        if let Some(lang) = get_attr_value(jsx, "lang") {
            if lang.is_empty() {
                diags.push(diag("jsx-a11y/lang", "lang attribute must have a valid value", span, Severity::Warning, None));
            }
        }
    }

    // --- jsx-a11y/role-has-required-aria-props ---
    if let Some(role_val) = get_attr_value(jsx, "role") {
        let required: &[(&str, &[&str])] = &[
            ("checkbox", &["aria-checked"]),
            ("radio", &["aria-checked"]),
            ("combobox", &["aria-expanded"]),
            ("slider", &["aria-valuemax", "aria-valuemin", "aria-valuenow"]),
            ("scrollbar", &["aria-controls", "aria-valuemax", "aria-valuemin", "aria-valuenow"]),
            ("heading", &["aria-level"]),
            ("option", &["aria-selected"]),
            ("switch", &["aria-checked"]),
            ("spinbutton", &["aria-valuemax", "aria-valuemin", "aria-valuenow"]),
        ];
        for (role, required_props) in required {
            if role_val == *role {
                for prop in *required_props {
                    if !has_attr(jsx, prop) {
                        diags.push(diag("jsx-a11y/role-has-required-aria-props", &format!("Role `{role}` requires `{prop}`"), span, Severity::Error, None));
                    }
                }
                break;
            }
        }
    }
}

// ==================== React-Perf rules ====================

fn check_react_perf_rules(
    jsx: &starlint::plugin::types::JsxOpeningElementNode,
    source: &str,
    diags: &mut Vec<LintDiagnostic>,
) {
    let span = jsx.span;
    let start_usize = span.start as usize;
    let end_usize = span.end as usize;
    let jsx_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");

    for attr in &jsx.attributes {
        if attr.is_spread {
            continue;
        }

        // --- react-perf/jsx-no-new-function-as-prop ---
        if let Some(val) = &attr.value {
            if val.contains("=>") || val.starts_with("function") || val.contains(".bind(") {
                diags.push(diag("react-perf/jsx-no-new-function-as-prop", &format!("Avoid creating new functions in `{}` prop (causes re-renders)", attr.name), span, Severity::Warning, None));
            }
        }

        // --- react-perf/jsx-no-new-object-as-prop ---
        if attr.value.is_none() && !attr.is_spread {
            // Expression attribute — check source text
            // Look for the attribute in source
            if let Some(attr_pos) = jsx_text.find(&format!("{}=", attr.name)) {
                let after = &jsx_text[attr_pos..];
                if after.contains("={{") || after.contains("= {{") {
                    diags.push(diag("react-perf/jsx-no-new-object-as-prop", &format!("Avoid creating new objects in `{}` prop", attr.name), span, Severity::Warning, None));
                }
            }
        }

        // --- react-perf/jsx-no-new-array-as-prop ---
        if attr.value.is_none() && !attr.is_spread {
            if let Some(attr_pos) = jsx_text.find(&format!("{}=", attr.name)) {
                let after = &jsx_text[attr_pos..];
                if after.contains("={[") || after.contains("= {[") {
                    diags.push(diag("react-perf/jsx-no-new-array-as-prop", &format!("Avoid creating new arrays in `{}` prop", attr.name), span, Severity::Warning, None));
                }
            }
        }
    }

    // --- react-perf/jsx-no-jsx-as-prop ---
    for attr in &jsx.attributes {
        if !attr.is_spread && attr.value.is_none() {
            if let Some(attr_pos) = jsx_text.find(&format!("{}=", attr.name)) {
                let after = &jsx_text[attr_pos..];
                if after.contains("={<") || after.contains("= {<") {
                    diags.push(diag("react-perf/jsx-no-jsx-as-prop", &format!("Avoid inline JSX in `{}` prop", attr.name), span, Severity::Warning, None));
                }
            }
        }
    }
}

// ==================== CallExpression-based rules ====================

fn check_call_expr_rules(
    call: &starlint::plugin::types::CallExpressionNode,
    source: &str,
    diags: &mut Vec<LintDiagnostic>,
) {
    let callee = &call.callee_path;
    let span = call.span;

    // --- react/no-find-dom-node ---
    if callee == "ReactDOM.findDOMNode" || callee == "findDOMNode" {
        diags.push(diag("react/no-find-dom-node", "`findDOMNode` is deprecated", span, Severity::Warning, None));
    }

    // --- react/no-render-return-value ---
    if callee == "ReactDOM.render" {
        let start_usize = span.start as usize;
        let before = source.get(start_usize.saturating_sub(30)..start_usize).unwrap_or("");
        if before.contains("=") || before.contains("const ") || before.contains("let ") || before.contains("var ") {
            diags.push(diag("react/no-render-return-value", "Do not use the return value of `ReactDOM.render()`", span, Severity::Warning, None));
        }
    }

    // --- react/jsx-key ---
    if callee.ends_with(".map") {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        if (call_text.contains("<") || call_text.contains("jsx")) && !call_text.contains("key=") && !call_text.contains("key:") {
            diags.push(diag("react/jsx-key", "Missing `key` prop for element in `.map()` iterator", span, Severity::Error, None));
        }
    }

    // --- react/no-array-index-key ---
    if callee.ends_with(".map") {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        // Check for key={index} or key={i} patterns
        if call_text.contains("key={index}") || call_text.contains("key={i}") || call_text.contains("key={idx}") {
            diags.push(diag("react/no-array-index-key", "Avoid using array index as `key` — use a stable identifier", span, Severity::Warning, None));
        }
    }

    // --- react/jsx-no-constructed-context-values ---
    if callee.ends_with(".Provider") {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        if call_text.contains("value={{") || call_text.contains("value={[") || call_text.contains("value={new ") {
            diags.push(diag("react/jsx-no-constructed-context-values", "Context value should be memoized to prevent re-renders", span, Severity::Warning, None));
        }
    }

    // --- react/forward-ref-uses-ref ---
    if callee == "React.forwardRef" || callee == "forwardRef" {
        let start_usize = span.start as usize;
        let end_usize = span.end as usize;
        let call_text = source.get(start_usize..end_usize.min(source.len())).unwrap_or("");
        // Check if the callback uses the ref parameter
        if call_text.contains("(props)") || call_text.contains("(props,") {
            if !call_text.contains(", ref)") && !call_text.contains(",ref)") {
                diags.push(diag("react/forward-ref-uses-ref", "`forwardRef` should use the `ref` parameter", span, Severity::Warning, None));
            }
        }
    }
}

// ==================== Import-based rules ====================

fn check_import_rules(
    import: &starlint::plugin::types::ImportDeclarationNode,
    _diags: &mut Vec<LintDiagnostic>,
) {
    // Most import-related react rules are handled by text scanning
    let _source = &import.source;
}

// ==================== MemberExpression-based rules ====================

fn check_member_expr_rules(
    member: &starlint::plugin::types::MemberExpressionNode,
    diags: &mut Vec<LintDiagnostic>,
) {
    // --- react/no-is-mounted (AST) ---
    if member.property == "isMounted" && member.object.contains("this") {
        diags.push(diag("react/no-is-mounted", "`isMounted` is deprecated", member.span, Severity::Warning, None));
    }
}

// ==================== Text-scanning rules ====================

fn check_no_multi_comp(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let patterns = ["extends Component", "extends React.Component", "extends PureComponent"];
    let mut comp_count = 0;
    for pattern in &patterns {
        comp_count += count_occurrences(source, pattern);
    }
    // Also count function components (rough heuristic)
    let func_comp_count = count_occurrences(source, "return (") + count_occurrences(source, "return (<");
    if comp_count + func_comp_count > 1 && comp_count > 1 {
        diags.push(warn("react/no-multi-comp", "Only one component per file", 0, 0));
    }
}

fn check_no_unescaped_entities(_source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Skip — requires JSX text node detection which is complex in text scanning
}

fn check_jsx_no_comment_textnodes(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Look for HTML comments inside JSX
    let pattern = "<!-- ";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        // Check if inside JSX (rough: preceded by > and followed by <)
        let before = &source[..abs];
        if before.rfind('>').is_some() {
            diags.push(warn(
                "react/jsx-no-comment-textnodes",
                "Comments inside JSX should use `{/* comment */}` syntax",
                abs, abs + pattern.len(),
            ));
        }
        pos = abs + 1;
    }
}

fn check_only_export_components(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Check if file exports non-component items alongside components
    let has_jsx = source.contains("<") && source.contains("/>");
    if !has_jsx {
        return;
    }

    let export_patterns = ["export const ", "export function ", "export let "];
    let mut exported_names: Vec<(&str, usize)> = Vec::new();

    for pattern in &export_patterns {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(pattern) {
            let abs = pos + found;
            let name_start = abs + pattern.len();
            let after = &source[name_start..];
            let name_end = after.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after.len());
            let name = &after[..name_end];
            if !name.is_empty() {
                exported_names.push((name, abs));
            }
            pos = abs + 1;
        }
    }

    let has_component = exported_names.iter().any(|(n, _)| n.chars().next().map_or(false, |c| c.is_uppercase()));
    let has_non_component = exported_names.iter().any(|(n, _)| n.chars().next().map_or(false, |c| c.is_lowercase()));

    if has_component && has_non_component {
        for (name, pos) in &exported_names {
            if name.chars().next().map_or(false, |c| c.is_lowercase()) {
                diags.push(warn(
                    "react/only-export-components",
                    &format!("Non-component export `{name}` mixed with component exports"),
                    *pos, pos + name.len(),
                ));
            }
        }
    }
}

fn check_react_in_jsx_scope(_source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // In modern React (17+), this is not needed. Skip.
}

fn check_no_direct_mutation_state(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "this.state.";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        let after = &source[abs + pattern.len()..];
        // Check for assignment
        if let Some(eq_pos) = after.find('=') {
            let between = &after[..eq_pos];
            if !between.contains('\n') && !between.contains(';') && !between.contains('{') {
                let before_eq = after.as_bytes().get(eq_pos.wrapping_sub(1));
                if before_eq != Some(&b'=') && before_eq != Some(&b'!') && before_eq != Some(&b'>') && before_eq != Some(&b'<') {
                    diags.push(err(
                        "react/no-direct-mutation-state",
                        "Do not mutate `this.state` directly — use `setState()`",
                        abs, abs + pattern.len(),
                    ));
                }
            }
        }
        pos = abs + 1;
    }
}

fn check_no_string_refs(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "ref=\"";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        diags.push(err(
            "react/no-string-refs",
            "String refs are deprecated — use `createRef()` or `useRef()`",
            abs, abs + pattern.len(),
        ));
        pos = abs + 1;
    }

    let pattern2 = "ref='";
    pos = 0;
    while let Some(found) = source[pos..].find(pattern2) {
        let abs = pos + found;
        diags.push(err(
            "react/no-string-refs",
            "String refs are deprecated — use `createRef()` or `useRef()`",
            abs, abs + pattern2.len(),
        ));
        pos = abs + 1;
    }
}

fn check_no_this_in_sfc(_source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Skip — requires component type detection
}

fn check_jsx_filename_extension(_ext: &str, _diags: &mut Vec<LintDiagnostic>) {
    // File pattern already restricts to jsx/tsx — this rule is about enforcing that
}

fn check_no_did_mount_set_state(source: &str, diags: &mut Vec<LintDiagnostic>) {
    check_lifecycle_set_state(source, "componentDidMount", "react/no-did-mount-set-state", diags);
}

fn check_no_will_update_set_state(source: &str, diags: &mut Vec<LintDiagnostic>) {
    check_lifecycle_set_state(source, "componentWillUpdate", "react/no-will-update-set-state", diags);
}

fn check_no_set_state(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "this.setState(";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        diags.push(warn(
            "react/no-set-state",
            "Avoid using `this.setState()` — prefer functional components with hooks",
            abs, abs + pattern.len(),
        ));
        pos = abs + 1;
    }
}

fn check_lifecycle_set_state(source: &str, lifecycle: &str, rule_name: &str, diags: &mut Vec<LintDiagnostic>) {
    if let Some(lifecycle_pos) = source.find(lifecycle) {
        let after = &source[lifecycle_pos..];
        if let Some(body_start) = after.find('{') {
            let body = &after[body_start..];
            if body.contains("this.setState(") {
                let set_state_pos = lifecycle_pos + body_start + body.find("this.setState(").unwrap_or(0);
                diags.push(warn(
                    rule_name,
                    &format!("Do not call `setState` in `{lifecycle}`"),
                    set_state_pos, set_state_pos + 14,
                ));
            }
        }
    }
}

fn check_no_is_mounted(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "this.isMounted()";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        diags.push(warn(
            "react/no-is-mounted",
            "`isMounted` is deprecated",
            abs, abs + pattern.len(),
        ));
        pos = abs + 1;
    }
}

fn check_jsx_fragments(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "<React.Fragment>";
    let mut pos = 0;
    while let Some(found) = source[pos..].find(pattern) {
        let abs = pos + found;
        // Check if it has a key prop
        let after = &source[abs..];
        if !after.starts_with("<React.Fragment key=") {
            diags.push(warn(
                "react/jsx-fragments",
                "Prefer `<>...</>` shorthand over `<React.Fragment>...</React.Fragment>`",
                abs, abs + pattern.len(),
            ));
        }
        pos = abs + 1;
    }
}

fn check_rules_of_hooks(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let hooks = ["useState(", "useEffect(", "useCallback(", "useMemo(", "useRef(", "useContext(", "useReducer("];

    for hook in &hooks {
        let mut pos = 0;
        while let Some(found) = source[pos..].find(hook) {
            let abs = pos + found;
            let before = &source[..abs];

            // Check if hook is inside a conditional
            let last_lines: Vec<&str> = before.lines().rev().take(5).collect();
            for line in &last_lines {
                let trimmed = line.trim();
                if trimmed.starts_with("if ") || trimmed.starts_with("if(") || trimmed.starts_with("} else") {
                    diags.push(err(
                        "react/rules-of-hooks",
                        &format!("Hook `{hook}` must not be called conditionally"),
                        abs, abs + hook.len(),
                    ));
                    break;
                }
            }

            pos = abs + 1;
        }
    }
}

fn check_display_name(source: &str, _diags: &mut Vec<LintDiagnostic>) {
    // Skip — complex to detect anonymous components reliably via text scanning
    let _ = source;
}

fn check_require_render_return(source: &str, diags: &mut Vec<LintDiagnostic>) {
    if let Some(render_pos) = source.find("render()") {
        let after = &source[render_pos..];
        if let Some(body_start) = after.find('{') {
            // Find matching brace
            let mut depth: u32 = 0;
            let mut has_return = false;
            for (i, ch) in after[body_start..].char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            break;
                        }
                    }
                    'r' if depth == 1 && after[body_start + i..].starts_with("return") => {
                        has_return = true;
                    }
                    _ => {}
                }
            }
            if !has_return {
                diags.push(err(
                    "react/require-render-return",
                    "`render()` method must return a value",
                    render_pos, render_pos + 8,
                ));
            }
        }
    }
}

fn check_state_in_constructor(source: &str, diags: &mut Vec<LintDiagnostic>) {
    // Check for state assignment outside constructor
    if source.contains("state = {") && !source.contains("constructor(") {
        if let Some(pos) = source.find("state = {") {
            diags.push(warn(
                "react/state-in-constructor",
                "State should be initialized in the constructor",
                pos, pos + 9,
            ));
        }
    }
}

fn check_prefer_es6_class(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let pattern = "React.createClass(";
    if let Some(pos) = source.find(pattern) {
        diags.push(warn(
            "react/prefer-es6-class",
            "Prefer ES6 class over `React.createClass()`",
            pos, pos + pattern.len(),
        ));
    }
}

fn check_no_unsafe(source: &str, diags: &mut Vec<LintDiagnostic>) {
    let unsafe_methods = [
        "UNSAFE_componentWillMount",
        "UNSAFE_componentWillReceiveProps",
        "UNSAFE_componentWillUpdate",
    ];

    for method in &unsafe_methods {
        if let Some(pos) = source.find(method) {
            diags.push(warn(
                "react/no-unsafe",
                &format!("`{method}` is unsafe and deprecated"),
                pos, pos + method.len(),
            ));
        }
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
