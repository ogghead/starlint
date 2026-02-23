#!/usr/bin/env bash
set -euo pipefail

# Sync version from Cargo.toml to all npm package.json files.
# Usage: ./scripts/version-sync.sh

VERSION=$(grep '^version' crates/starlint_cli/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
echo "Syncing version: $VERSION"

PACKAGES=(
  "npm/starlint/package.json"
  "npm/cli-linux-x64-gnu/package.json"
  "npm/cli-linux-arm64-gnu/package.json"
  "npm/cli-darwin-x64/package.json"
  "npm/cli-darwin-arm64/package.json"
  "npm/cli-win32-x64-msvc/package.json"
  "editors/vscode/package.json"
)

for pkg in "${PACKAGES[@]}"; do
  node -e "
    const fs = require('fs');
    const pkg = JSON.parse(fs.readFileSync('$pkg', 'utf8'));
    pkg.version = '$VERSION';
    fs.writeFileSync('$pkg', JSON.stringify(pkg, null, 2) + '\n');
  "
  echo "  Updated $pkg"
done

# Update optionalDependencies versions in root package
node -e "
  const fs = require('fs');
  const pkg = JSON.parse(fs.readFileSync('npm/starlint/package.json', 'utf8'));
  for (const dep in pkg.optionalDependencies) {
    pkg.optionalDependencies[dep] = '$VERSION';
  }
  fs.writeFileSync('npm/starlint/package.json', JSON.stringify(pkg, null, 2) + '\n');
"
echo "  Updated optionalDependencies in npm/starlint/package.json"

echo "Done. All packages set to $VERSION"
