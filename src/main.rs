#[macro_use] extern crate structopt;

use structopt::StructOpt;
use std::io::{self, BufRead};
use std::process::Command;

#[derive(StructOpt, Debug)]
#[structopt(name = "Rargs", about = "Xargs with pattern matching")]
#[structopt(raw(setting = "structopt::clap::AppSettings::TrailingVarArg"))]
struct Rargs {
    #[structopt()]
    pattern: String,

    #[structopt()]
    command: Vec<String>,
}

impl Rargs {
    fn execute_for_input(self: &Self, line: &str) {
        Command::new(&self.command[0])
            .args(self.replace_pattern(line))
            .spawn()
            .expect("command failed to start");
    }

    fn replace_pattern(self: &Self, line: &str) -> Vec<String> {
        self.command.iter().skip(1)
            .map(|arg| arg.replace("{}", line))
            .collect()
    }
}

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code);
}

fn real_main() -> i32 {
    let rargs = Rargs::from_args();

    let stdin = io::stdin();
    for wrapped_line in stdin.lock().lines() {
        let line = wrapped_line.unwrap();
        rargs.execute_for_input(&line);
    }

    1
}
