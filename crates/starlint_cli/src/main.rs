//! Binary entry point for starlint.

fn main() -> miette::Result<()> {
    let status = starlint_cli::run()?;
    if status == starlint_cli::ExitStatus::LintErrors {
        #[allow(clippy::exit)]
        std::process::exit(1);
    }
    Ok(())
}
