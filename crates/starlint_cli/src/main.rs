//! Binary entry point for starlint.

fn main() -> miette::Result<()> {
    starlint_cli::run()
}
