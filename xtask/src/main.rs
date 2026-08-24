mod bindings;
mod check_docs;
mod fixtures;

#[cfg(test)]
mod workflow_contract;

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::ExitCode,
};

const USAGE: &str = "usage:\n  cargo xtask fixtures [--check]\n  cargo xtask bindings [--check]\n  cargo xtask check-docs";

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Fixtures { check: bool },
    Bindings { check: bool },
    CheckDocs,
}

fn main() -> ExitCode {
    match parse_args(env::args_os().skip(1)).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Fixtures { check } => fixtures::run(check),
        Command::Bindings { check } => bindings::run(check),
        Command::CheckDocs => check_docs::run(),
    }
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let args = args
        .into_iter()
        .map(|value| {
            value
                .into_string()
                .map_err(|_| usage_error("xtask arguments must be valid Unicode"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    match args.as_slice() {
        [command] if command == "fixtures" => Ok(Command::Fixtures { check: false }),
        [command, flag] if command == "fixtures" && flag == "--check" => {
            Ok(Command::Fixtures { check: true })
        }
        [command] if command == "bindings" => Ok(Command::Bindings { check: false }),
        [command, flag] if command == "bindings" && flag == "--check" => {
            Ok(Command::Bindings { check: true })
        }
        [command] if command == "check-docs" => Ok(Command::CheckDocs),
        _ => Err(usage_error("unsupported xtask arguments")),
    }
}

fn usage_error(detail: &str) -> String {
    format!("{detail}\n\n{USAGE}")
}

fn repository_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask manifest directory has no repository parent".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_args};
    use std::ffi::OsString;

    #[test]
    fn command_parser_accepts_only_the_five_documented_forms() {
        assert_eq!(
            parse_args(["fixtures".into()]),
            Ok(Command::Fixtures { check: false })
        );
        assert_eq!(
            parse_args(["fixtures".into(), "--check".into()]),
            Ok(Command::Fixtures { check: true })
        );
        assert_eq!(
            parse_args(["bindings".into()]),
            Ok(Command::Bindings { check: false })
        );
        assert_eq!(
            parse_args(["bindings".into(), "--check".into()]),
            Ok(Command::Bindings { check: true })
        );
        assert_eq!(parse_args(["check-docs".into()]), Ok(Command::CheckDocs));

        for args in [
            Vec::new(),
            vec!["unknown".into()],
            vec!["fixtures".into(), "--repair".into()],
            vec!["fixtures".into(), "--check".into(), "extra".into()],
            vec!["bindings".into(), "--repair".into()],
            vec!["bindings".into(), "--check".into(), "extra".into()],
            vec!["check-docs".into(), "--check".into()],
        ] {
            let error = parse_args(args).expect_err("unsupported arguments must fail");
            assert!(error.contains("cargo xtask fixtures [--check]"));
            assert!(error.contains("cargo xtask bindings [--check]"));
            assert!(error.contains("cargo xtask check-docs"));
        }
    }

    #[cfg(windows)]
    #[test]
    fn non_unicode_arguments_fail_with_the_complete_usage() {
        use std::os::windows::ffi::OsStringExt;

        let error = parse_args([OsString::from_wide(&[0xd800])])
            .expect_err("non-Unicode arguments must fail");
        assert!(error.contains("must be valid Unicode"));
        assert!(error.contains("cargo xtask fixtures [--check]"));
        assert!(error.contains("cargo xtask bindings [--check]"));
        assert!(error.contains("cargo xtask check-docs"));
    }
}
