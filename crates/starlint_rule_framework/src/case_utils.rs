//! Case-checking and case-conversion utilities for lint rules.
//!
//! Provides helpers for detecting and converting between common naming
//! conventions: `PascalCase`, `camelCase`, `kebab-case`, `snake_case`.

/// Check if a string is `PascalCase` (starts with uppercase, no hyphens or
/// interior underscores). `ALL_CAPS` names like `SVG` are allowed.
#[must_use]
pub fn is_pascal_case(s: &str) -> bool {
    let Some(first) = s.chars().next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    if s.contains('-') {
        return false;
    }
    // Allow ALL_CAPS_WITH_UNDERSCORES (e.g., SVG, UNSAFE_Component)
    let is_all_upper = s
        .chars()
        .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit());
    if is_all_upper {
        return true;
    }
    // Must have at least one lowercase letter for PascalCase
    // and no underscores (except leading _)
    let has_lowercase = s.chars().any(|c| c.is_ascii_lowercase());
    let has_invalid_underscore = s.chars().skip(1).any(|c| c == '_');
    has_lowercase && !has_invalid_underscore
}

/// Check if a string is `kebab-case` (all lowercase with hyphens, no leading/trailing hyphens).
#[must_use]
pub fn is_kebab_case(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
        && !s.starts_with('-')
        && !s.ends_with('-')
}

/// Check if a string is `camelCase` (starts with lowercase, no hyphens or underscores).
#[must_use]
pub fn is_camel_case(s: &str) -> bool {
    let first = s.chars().next();
    matches!(first, Some('a'..='z')) && !s.contains('-') && !s.contains('_')
}

/// Convert a string to `PascalCase`.
///
/// Splits on hyphens, underscores, and spaces, then capitalizes the first
/// character of each segment.
#[must_use]
pub fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.collect();
                    format!("{upper}{rest}")
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Convert a string to `camelCase`.
///
/// Like [`to_pascal_case`] but lowercases the first character.
#[must_use]
pub fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(first) => {
            let lower: String = first.to_lowercase().collect();
            let rest: String = chars.collect();
            format!("{lower}{rest}")
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[test]
    fn test_is_pascal_case() {
        assert!(is_pascal_case("MyComponent"), "MyComponent is PascalCase");
        assert!(is_pascal_case("A"), "single uppercase is PascalCase");
        assert!(is_pascal_case("SVG"), "ALL_CAPS is PascalCase");
        assert!(
            is_pascal_case("ALL_CAPS"),
            "ALL_CAPS with underscore is PascalCase"
        );
        assert!(
            !is_pascal_case("myComponent"),
            "camelCase is not PascalCase"
        );
        assert!(
            !is_pascal_case("my-component"),
            "kebab-case is not PascalCase"
        );
        assert!(!is_pascal_case(""), "empty string is not PascalCase");
        assert!(
            !is_pascal_case("My_Component"),
            "underscore in mixed case is not PascalCase"
        );
    }

    #[test]
    fn test_is_kebab_case() {
        assert!(is_kebab_case("my-component"), "kebab-case");
        assert!(is_kebab_case("abc"), "single word lowercase");
        assert!(!is_kebab_case("MyComponent"), "PascalCase");
        assert!(!is_kebab_case("-leading"), "leading hyphen");
        assert!(!is_kebab_case("trailing-"), "trailing hyphen");
    }

    #[test]
    fn test_is_camel_case() {
        assert!(is_camel_case("myComponent"), "camelCase");
        assert!(!is_camel_case("MyComponent"), "PascalCase");
        assert!(!is_camel_case("my-component"), "kebab-case");
        assert!(!is_camel_case("my_component"), "snake_case");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my-component"), "MyComponent");
        assert_eq!(to_pascal_case("my_component"), "MyComponent");
        assert_eq!(to_pascal_case("my component"), "MyComponent");
        assert_eq!(to_pascal_case("already"), "Already");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("my-component"), "myComponent");
        assert_eq!(to_camel_case("my_component"), "myComponent");
    }
}
