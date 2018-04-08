#[macro_use] extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Rargs", about = "Xargs with pattern matching")]
#[structopt(raw(setting = "structopt::clap::AppSettings::TrailingVarArg"))]
struct Rargs {
    #[structopt()]
    pattern: String,

    #[structopt()]
    command: Vec<String>,
}

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code);
}

fn real_main() -> i32 {
    let opts = Rargs::from_args();
    println!("{:?}", opts);
    1
}
