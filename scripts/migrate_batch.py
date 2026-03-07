#!/usr/bin/env python3
"""Migrate NativeRule implementations to LintRule.

This script handles mechanical transforms:
1. Import changes (oxc_ast → starlint_ast, NativeRule → LintRule)
2. Trait impl changes (NativeRule → LintRule)
3. Method signature changes (run_on_kinds → run_on_types, etc.)
4. Pattern match changes (AstKind::X → AstNode::X, AstType::X → AstNodeType::X)
5. Test boilerplate (traverse_and_lint → lint_source)

It does NOT handle:
- Child access via NodeId (Expression::X, Statement::X patterns)
- oxc helper functions
- Complex test patterns beyond the standard fn lint() helper
"""

import re
import sys
import os
from pathlib import Path


def should_skip(content: str) -> str | None:
    """Return reason to skip, or None if OK to migrate."""
    if "impl LintRule" in content:
        return "already migrated"
    if "impl NativeRule" not in content:
        return "no NativeRule impl"
    # Skip rules that use Expression:: or Statement:: (medium difficulty)
    if "Expression::" in content or "Statement::" in content:
        return "uses Expression::/Statement::"
    # Skip rules that use ctx.semantic()
    if "ctx.semantic()" in content:
        return "uses ctx.semantic()"
    # Skip rules that use oxc_ast::ast:: sub-types beyond the standard AstKind/AstType
    # Allow: oxc_ast::ast:: imports that are only for AstKind, AstType (handled by script)
    # Allow: use of Argument, PropertyKey etc. that map to simple patterns
    if "oxc_ast::ast::" in content:
        # Check what's actually imported — skip if it uses complex oxc types
        # that have no starlint_ast equivalent
        imports = re.findall(r'use oxc_ast::ast::(\w+)', content)
        # These are handled by the script or have starlint_ast equivalents
        ok_types = {
            'AstKind', 'AstType', 'RegExpFlags', 'UnaryOperator',
            'BinaryOperator', 'LogicalOperator', 'AssignmentOperator',
            'UpdateOperator', 'VariableDeclarationKind',
        }
        problematic = [t for t in imports if t not in ok_types]
        if problematic:
            return f"uses oxc_ast::ast sub-types: {', '.join(problematic[:3])}"
    # Skip rules that use RegExpFlags directly (bitflags need manual conversion)
    if "RegExpFlags::" in content:
        return "uses RegExpFlags bitflags"
    # Skip rules that use AstKind in non-standard ways
    if content.count("AstKind::") > 5:
        return "many AstKind matches (complex)"
    return None


def migrate_imports(content: str) -> str:
    """Replace oxc imports with starlint_ast imports."""
    # Remove oxc_ast imports
    content = re.sub(r'use oxc_ast::AstKind;\n', '', content)
    content = re.sub(r'use oxc_ast::ast_kind::AstType;\n', '', content)
    content = re.sub(r'use oxc_ast::ast::\{[^}]*\};\n', '', content)
    content = re.sub(r'use oxc_ast::ast::[^;]+;\n', '', content)
    # Remove oxc_span imports (we use starlint_plugin_sdk Span)
    content = re.sub(r'use oxc_span::\{[^}]*\};\n', '', content)
    content = re.sub(r'use oxc_span::[^;]+;\n', '', content)

    # Replace NativeRule import with LintRule import
    content = re.sub(
        r'use crate::rule::\{NativeLintContext, NativeRule\};',
        'use crate::lint_rule::{LintContext, LintRule};\n'
        'use starlint_ast::node::AstNode;\n'
        'use starlint_ast::node_type::AstNodeType;\n'
        'use starlint_ast::types::NodeId;',
        content
    )
    # Handle variant with just NativeRule
    content = re.sub(
        r'use crate::rule::NativeRule;',
        'use crate::lint_rule::LintRule;',
        content
    )
    content = re.sub(
        r'use crate::rule::NativeLintContext;',
        'use crate::lint_rule::LintContext;',
        content
    )

    return content


def migrate_trait_impl(content: str, struct_name: str) -> str:
    """Change NativeRule impl to LintRule impl."""
    content = content.replace(
        f"impl NativeRule for {struct_name}",
        f"impl LintRule for {struct_name}"
    )
    return content


def migrate_run_method(content: str) -> str:
    """Change run method signature and body patterns."""
    # Change run signature
    content = re.sub(
        r'fn run\(&self, kind: &AstKind<\'_>, ctx: &mut NativeLintContext<\'_>\)',
        'fn run(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<\'_>)',
        content
    )
    # Change leave signature
    content = re.sub(
        r'fn leave\(&self, kind: &AstKind<\'_>, ctx: &mut NativeLintContext<\'_>\)',
        'fn leave(&self, _node_id: NodeId, node: &AstNode, ctx: &mut LintContext<\'_>)',
        content
    )
    # Change run_once signature
    content = re.sub(
        r'fn run_once\(&self, ctx: &mut NativeLintContext<\'_>\)',
        'fn run_once(&self, ctx: &mut LintContext<\'_>)',
        content
    )
    # Change should_run_on_file
    content = re.sub(
        r'fn should_run_on_file\(&self, source_text: &str, file_path: &Path\) -> bool',
        'fn should_run_on_file(&self, source_text: &str, file_path: &Path) -> bool',
        content
    )
    # Change run_on_kinds → run_on_types
    content = re.sub(
        r'fn run_on_kinds\(&self\) -> Option<&\'static \[AstType\]>',
        "fn run_on_types(&self) -> Option<&'static [AstNodeType]>",
        content
    )
    # Change leave_on_kinds → leave_on_types
    content = re.sub(
        r'fn leave_on_kinds\(&self\) -> Option<&\'static \[AstType\]>',
        "fn leave_on_types(&self) -> Option<&'static [AstNodeType]>",
        content
    )
    # Replace AstKind:: with AstNode:: in pattern matches
    content = content.replace("AstKind::", "AstNode::")
    # Replace AstType:: with AstNodeType::
    content = content.replace("AstType::", "AstNodeType::")
    # Replace 'kind' with 'node' in let-else patterns
    content = re.sub(r'let AstNode::(\w+)\((\w+)\) = kind\b', r'let AstNode::\1(\2) = node', content)
    # Replace match kind { with match node {
    content = re.sub(r'match kind \{', 'match node {', content)
    # Replace if let AstNode::X(y) = kind {
    content = re.sub(r'if let (AstNode::\w+\(\w+\)) = kind\b', r'if let \1 = node', content)
    # Replace ctx.source_text() that was previously available via NativeLintContext
    # (LintContext also has source_text(), so no change needed)

    # --- Field name mappings (oxc → starlint_ast) ---

    # Import source: import.source.value.as_str() → import.source.as_str()
    content = re.sub(r'\.source\.value\.as_str\(\)', '.source.as_str()', content)
    # import.source.value.to_string() → import.source.to_string() (or clone)
    content = re.sub(r'\.source\.value\.to_string\(\)', '.source.clone()', content)
    # import.source.value (standalone, e.g. in format strings or comparisons)
    content = re.sub(r'\.source\.value\b', '.source', content)
    # import.source.span → import.source_span
    content = re.sub(r'\.source\.span\b', '.source_span', content)

    # Function async: .r#async → .is_async
    content = re.sub(r'\.r#async\b', '.is_async', content)

    # NumericLiteral: .raw_str() → .raw.as_str()
    content = re.sub(r'\.raw_str\(\)', '.raw.as_str()', content)

    # RegExpLiteral: oxc uses it.regex.pattern and it.regex.flags (bitflags struct)
    # starlint_ast has .pattern (String) and .flags (String) directly
    # it.regex.pattern.text → it.pattern
    content = re.sub(r'\.regex\.pattern\.text\.as_str\(\)', '.pattern.as_str()', content)
    content = re.sub(r'\.regex\.pattern\.text', '.pattern', content)
    content = re.sub(r'\.regex\.pattern\.as_str\(\)', '.pattern.as_str()', content)
    content = re.sub(r'\.regex\.pattern', '.pattern', content)
    # it.regex.flags → it.flags (oxc uses bitflags, starlint_ast uses String)
    # NOTE: Rules using flags.contains(RegExpFlags::X) need manual fixup
    content = re.sub(r'\.regex\.flags', '.flags', content)

    return content


def migrate_test_module(content: str, struct_name: str, rule_constructor: str) -> str:
    """Replace the test module's lint helper and imports."""
    # Find the #[cfg(test)] mod tests block
    test_start = content.find("#[cfg(test)]")
    if test_start == -1:
        return content

    before_tests = content[:test_start]
    test_section = content[test_start:]

    # Remove old test imports
    test_section = re.sub(r'\s*use std::path::Path;\n', '\n', test_section)
    test_section = re.sub(r'\s*use oxc_allocator::Allocator;\n', '', test_section)
    test_section = re.sub(r'\s*use crate::parser::parse_file;\n', '', test_section)
    test_section = re.sub(r'\s*use crate::traversal::traverse_and_lint;\n', '', test_section)
    test_section = re.sub(r'\s*use crate::traversal::\{[^}]*\};\n', '', test_section)
    test_section = re.sub(r'\s*use crate::rule::NativeRule;\n', '', test_section)
    test_section = re.sub(r'\s*use crate::rule::\{NativeRule[^}]*\};\n', '', test_section)

    # Add lint_source import after super::*
    if "use crate::lint_rule::lint_source;" not in test_section:
        test_section = test_section.replace(
            "use super::*;",
            "use super::*;\n    use crate::lint_rule::lint_source;"
        )

    # Replace the standard lint helper function pattern
    # Pattern: fn lint(source: &str) -> Vec<...> { allocator... traverse_and_lint... }
    standard_lint_fn = re.compile(
        r'fn lint\(source: &str\) -> Vec<[^>]+> \{\n'
        r'(?:.*?\n)*?'  # any lines
        r'\s*\}',
        re.MULTILINE
    )

    # Try to find and replace the lint function
    # We need to be careful - find the function that contains traverse_and_lint or Allocator
    lines = test_section.split('\n')
    new_lines = []
    i = 0
    replaced_lint = False

    while i < len(lines):
        line = lines[i]
        # Detect start of lint helper function
        if (re.match(r'\s+fn lint\(source: &str\)', line) and not replaced_lint):
            # Find the end of this function by tracking braces
            brace_count = 0
            fn_lines = []
            j = i
            while j < len(lines):
                fn_lines.append(lines[j])
                brace_count += lines[j].count('{') - lines[j].count('}')
                if brace_count <= 0 and '{' in ''.join(fn_lines):
                    break
                j += 1

            fn_body = '\n'.join(fn_lines)
            if 'traverse_and_lint' in fn_body or 'Allocator' in fn_body or 'parse_file' in fn_body:
                # Replace with lint_source version
                indent = re.match(r'^(\s*)', line).group(1)
                new_lines.append(f"{indent}fn lint(source: &str) -> Vec<Diagnostic> {{")
                new_lines.append(f"{indent}    let rules: Vec<Box<dyn LintRule>> = vec![Box::new({rule_constructor})];")
                new_lines.append(f'{indent}    lint_source(source, "test.js", &rules)')
                new_lines.append(f"{indent}}}")
                replaced_lint = True
                i = j + 1
                continue

        # Also handle lint_with_path pattern
        if re.match(r'\s+fn lint_with_path\(source: &str, path: &str\)', line):
            brace_count = 0
            fn_lines = []
            j = i
            while j < len(lines):
                fn_lines.append(lines[j])
                brace_count += lines[j].count('{') - lines[j].count('}')
                if brace_count <= 0 and '{' in ''.join(fn_lines):
                    break
                j += 1
            fn_body = '\n'.join(fn_lines)
            if 'traverse_and_lint' in fn_body or 'Allocator' in fn_body:
                indent = re.match(r'^(\s*)', line).group(1)
                new_lines.append(f"{indent}fn lint_with_path(source: &str, path: &str) -> Vec<Diagnostic> {{")
                new_lines.append(f"{indent}    let rules: Vec<Box<dyn LintRule>> = vec![Box::new({rule_constructor})];")
                new_lines.append(f"{indent}    lint_source(source, path, &rules)")
                new_lines.append(f"{indent}}}")
                i = j + 1
                continue

        new_lines.append(line)
        i += 1

    test_section = '\n'.join(new_lines)

    # Replace any remaining NativeRule references in tests
    test_section = test_section.replace("Vec<Box<dyn NativeRule>>", "Vec<Box<dyn LintRule>>")

    # Replace any remaining traverse_and_lint calls that weren't in a helper
    # Pattern: let diags = traverse_and_lint(...);
    # These are inline in test functions
    test_section = re.sub(
        r'let (\w+) = traverse_and_lint\(&parsed\.program, &rules, source, [^)]+\);',
        r'let \1 = lint_source(source, "test.js", &rules);',
        test_section
    )

    return before_tests + test_section


def find_struct_and_constructor(content: str) -> tuple[str, str]:
    """Find the struct name and its test constructor."""
    # Find struct name from impl NativeRule for X
    match = re.search(r'impl NativeRule for (\w+)', content)
    if not match:
        return "", ""
    struct_name = match.group(1)

    # Check if struct has new() or default constructor
    # Look for ::new() in test lint helper
    if f"{struct_name}::new()" in content:
        constructor = f"{struct_name}::new()"
    elif f"{struct_name}::default()" in content:
        constructor = f"{struct_name}::default()"
    elif re.search(rf'{struct_name}\s*\{{', content):
        # Has field initialization - check tests for the pattern
        # Default to struct literal
        constructor = struct_name
    else:
        constructor = struct_name

    return struct_name, constructor


def migrate_file(filepath: str, dry_run: bool = False) -> tuple[bool, str]:
    """Migrate a single file. Returns (success, reason)."""
    with open(filepath, 'r') as f:
        content = f.read()

    skip_reason = should_skip(content)
    if skip_reason:
        return False, f"skipped: {skip_reason}"

    struct_name, constructor = find_struct_and_constructor(content)
    if not struct_name:
        return False, "skipped: couldn't find struct name"

    original = content

    # Apply transforms
    content = migrate_imports(content)
    content = migrate_trait_impl(content, struct_name)
    content = migrate_run_method(content)
    content = migrate_test_module(content, struct_name, constructor)

    # Clean up double blank lines
    content = re.sub(r'\n{3,}', '\n\n', content)

    if content == original:
        return False, "no changes needed"

    if not dry_run:
        with open(filepath, 'w') as f:
            f.write(content)

    return True, "migrated"


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Migrate NativeRule to LintRule")
    parser.add_argument("--dry-run", action="store_true", help="Don't write files")
    parser.add_argument("--file", type=str, help="Migrate a single file")
    parser.add_argument("--all", action="store_true", help="Migrate all eligible files")
    parser.add_argument("--dir", type=str, default="crates/starlint_core/src/rules",
                       help="Directory to search for rule files")
    args = parser.parse_args()

    if args.file:
        success, reason = migrate_file(args.file, args.dry_run)
        print(f"{'OK' if success else 'SKIP'}: {args.file} ({reason})")
        return

    if args.all:
        rules_dir = Path(args.dir)
        migrated = []
        skipped = {}

        for filepath in sorted(rules_dir.rglob("*.rs")):
            if filepath.name == "mod.rs":
                continue
            success, reason = migrate_file(str(filepath), args.dry_run)
            if success:
                migrated.append(str(filepath))
            else:
                skipped.setdefault(reason, []).append(str(filepath))

        print(f"\nMigrated: {len(migrated)} files")
        for f in migrated:
            print(f"  + {f}")

        print(f"\nSkipped: {sum(len(v) for v in skipped.values())} files")
        for reason, files in sorted(skipped.items()):
            print(f"  {reason}: {len(files)}")

        return

    parser.print_help()


if __name__ == "__main__":
    main()
