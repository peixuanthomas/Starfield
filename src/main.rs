mod scene;
mod terminal;
#[cfg(feature = "window")]
mod window;

use std::{error::Error, process::ExitCode};
use terminal::Backend;

const HELP: &str = "Starfield — the same starfield in a window or your terminal

Usage: starfield [--window | --terminal[=auto|kitty|ansi]]

  --window         Open the original graphical window (default)
  --terminal       Choose Kitty graphics on recognized terminals, ANSI otherwise
  --terminal=kitty Send the original 1200x800 RGB frame using Kitty graphics
  --terminal=ansi  Use true-color Unicode half blocks (lower resolution)
  -h, --help       Show this help

Terminal controls: Esc, q, or Ctrl-C to quit.
Build without graphical dependencies: cargo build --release --no-default-features
Kitty graphics requires a compatible terminal; ANSI requires Unicode and true color.";

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Window,
    Terminal(Backend),
    Help,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Mode, String> {
    let mut mode = None;
    for arg in args {
        let selected = match arg.as_str() {
            "-h" | "--help" => return Ok(Mode::Help),
            "--window" => Mode::Window,
            "--terminal" | "--terminal=auto" => Mode::Terminal(Backend::Auto),
            "--terminal=kitty" => Mode::Terminal(Backend::Kitty),
            "--terminal=ansi" => Mode::Terminal(Backend::Ansi),
            _ => return Err(format!("Unknown argument: {arg}. Use --help for usage.")),
        };
        if mode.replace(selected).is_some() {
            return Err("Choose only one output mode. Use --help for usage.".into());
        }
    }
    Ok(mode.unwrap_or(Mode::Window))
}

fn run() -> Result<(), Box<dyn Error>> {
    match parse_args(std::env::args().skip(1))? {
        Mode::Help => println!("{HELP}"),
        Mode::Terminal(backend) => terminal::run(backend)?,
        Mode::Window => {
            #[cfg(feature = "window")]
            window::run()?;
            #[cfg(not(feature = "window"))]
            return Err("Window support is disabled in this build. Run with --terminal.".into());
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("starfield: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Mode, String> {
        parse_args(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn output_modes_are_explicit_and_window_remains_default() {
        assert_eq!(parse(&[]), Ok(Mode::Window));
        assert_eq!(parse(&["--window"]), Ok(Mode::Window));
        assert_eq!(parse(&["--terminal"]), Ok(Mode::Terminal(Backend::Auto)));
        assert_eq!(
            parse(&["--terminal=auto"]),
            Ok(Mode::Terminal(Backend::Auto))
        );
        assert_eq!(
            parse(&["--terminal=kitty"]),
            Ok(Mode::Terminal(Backend::Kitty))
        );
        assert_eq!(
            parse(&["--terminal=ansi"]),
            Ok(Mode::Terminal(Backend::Ansi))
        );
        assert_eq!(parse(&["--help"]), Ok(Mode::Help));
        assert!(parse(&["--terminal=invalid"]).is_err());
        assert!(parse(&["--window", "--terminal"]).is_err());
    }
}
