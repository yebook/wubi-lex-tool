mod fixtures;

use std::{env, ffi::OsString, process::ExitCode};

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Fixtures { check: bool },
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
    }
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let args = args
        .into_iter()
        .map(|value| {
            value
                .into_string()
                .map_err(|_| "xtask arguments must be valid Unicode".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;

    match args.as_slice() {
        [command] if command == "fixtures" => Ok(Command::Fixtures { check: false }),
        [command, flag] if command == "fixtures" && flag == "--check" => {
            Ok(Command::Fixtures { check: true })
        }
        _ => Err("usage: cargo xtask fixtures [--check]".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_args};

    #[test]
    fn fixture_command_accepts_only_the_offline_check_flag() {
        assert_eq!(
            parse_args(["fixtures".into()]),
            Ok(Command::Fixtures { check: false })
        );
        assert_eq!(
            parse_args(["fixtures".into(), "--check".into()]),
            Ok(Command::Fixtures { check: true })
        );

        for args in [
            Vec::new(),
            vec!["unknown".into()],
            vec!["fixtures".into(), "--repair".into()],
            vec!["fixtures".into(), "--check".into(), "extra".into()],
        ] {
            assert_eq!(
                parse_args(args).expect_err("unsupported arguments must fail"),
                "usage: cargo xtask fixtures [--check]"
            );
        }
    }
}
