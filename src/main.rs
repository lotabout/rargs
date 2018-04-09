#[macro_use]extern crate structopt;
#[macro_use]extern crate lazy_static;

extern crate regex;
extern crate num_cpus;
extern crate threadpool;

use structopt::StructOpt;
use std::io::{self, BufRead};
use std::process::Command;
use regex::Regex;
use std::collections::HashMap;
use std::convert::From;
use threadpool::ThreadPool;
use std::sync::Arc;

fn main() {
    let mut exit_code = 0;

    let options = Options::from_args();
    let rargs = Arc::new(Rargs::new(&options));

    let stdin = io::stdin();

    let num_worker = if options.worker > 0 {options.worker} else {num_cpus::get()};
    let pool = ThreadPool::new(num_worker);

    let line_ending = if options.read0 {b'\0'} else {b'\n'};
    loop {
        let mut buffer = Vec::with_capacity(1024);
        match stdin.lock().read_until(line_ending, &mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                // remove line-ending
                if buffer.ends_with(&[b'\r', b'\n']) {
                    buffer.pop();
                    buffer.pop();
                } else if buffer.ends_with(&[b'\n']) || buffer.ends_with(&[b'\0']) {
                    buffer.pop();
                }

                // execute command on line
                let rargs = rargs.clone();
                pool.execute(move || {
                    let line = String::from_utf8(buffer).expect("Found invalid UTF8");
                    rargs.execute_for_input(&line);
                });
            }
            Err(_err) => {
                // String not UTF8 or other error, skip.
                exit_code = 1;
                break;
            }
        }
    }

    pool.join();
    std::process::exit(exit_code);
}

lazy_static! {
    static ref CMD_REGEX: Regex = Regex::new(r"\{[[:space:]]*[[:alnum:]._-]*[[:space:]]*\}").unwrap();
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Rargs", about = "Xargs with pattern matching")]
#[structopt(raw(setting = "structopt::clap::AppSettings::TrailingVarArg"))]
struct Options {
    #[structopt(long="read0", short="0")]
    read0: bool,

    #[structopt(long="worker", short="w", default_value = "1")]
    worker: usize,

    #[structopt(long="pattern", short="p")]
    pattern: Option<String>,

    #[structopt(long="delimiter", short="d", conflicts_with = "pattern")]
    delimiter: Option<String>,

    #[structopt()]
    command: Vec<String>,
}

#[derive(Debug)]
struct Rargs {
    pattern: Regex,
    command: String,
    args: Vec<ArgTemplate>,
}

impl Rargs {
    pub fn new(opts: &Options) -> Self {
        let pattern;

        if let Some(pat_string) = opts.pattern.as_ref() {
            pattern = Regex::new(pat_string).unwrap();
        } else if let Some(delimiter) = opts.delimiter.as_ref() {
            let pat_string = format!(r"(.*?){}|(.*?)$", delimiter);
            pattern = Regex::new(&pat_string).unwrap();
        } else {
            pattern = Regex::new(r"(.*?)[[:space:]]+|(.*?)$").unwrap();
        }

        let command = opts.command[0].to_string();
        let args = opts.command[1..].iter().map(|s| ArgTemplate::from(&**s)).collect();

        Rargs{pattern, command, args}
    }

    fn execute_for_input(self: &Self, line: &str) {
        let context = build_regex_context(&self.pattern, line);
        let args: Vec<String> = self.args.iter().map(|arg| arg.apply_context(&context)).collect();

        Command::new(&self.command)
            .args(args)
            .spawn()
            .expect("command failed to start");
    }
}

/// The context parsed from the input line using the pattern given. For Example:
///
/// input: 2018-10-21
/// pattern: "^(?P<year>\d{4})-(\d{2})-(\d{2})$"
///
/// will result in the context:
/// {}/{0} => "2018-10-21"
/// {1}/{year} => "2018"
/// {2} => "10"
/// {3} => "21"
type Context<'a> = HashMap<String, &'a str>;

fn build_regex_context<'a>(pattern: &'a Regex, content: &'a str) -> Context<'a> {
    let mut context = HashMap::new();
    context.insert("".to_string(), content);
    context.insert("0".to_string(), content);

    let group_names = pattern.capture_names()
        .filter_map(|x| x)
        .collect::<Vec<&str>>();

    let mut groups = vec![];

    for caps in pattern.captures_iter(content) {
        // the numbered group
        for mat_wrapper in caps.iter().skip(1) {
            if let Some(mat) = mat_wrapper {
                groups.push(mat.as_str());
            }
        }

        // the named group
        for name in group_names.iter() {
            if let Some(mat) = caps.name(name) {
                context.insert(name.to_string(), mat.as_str());
            }
        }
    }

    let group_num = groups.len();
    for (idx, match_string) in groups.into_iter().enumerate() {
        context.insert((idx+1).to_string(), match_string);
        context.insert((-((group_num-idx) as i32)).to_string(), match_string);
    }

    context
}

#[derive(Debug)]
enum ArgFragment{
    Literal(String),
    Placeholder(String),
}

/// The "compiled" template for arguments. for example:
///
/// "x {abc} z" will be compiled so that later `{abc}` could be replaced by actuals content
#[derive(Debug)]
struct ArgTemplate {
    fragments: Vec<ArgFragment>
}

impl<'a> From<&'a str> for ArgTemplate {
    fn from(arg: &'a str) -> Self {
        let mut fragments = Vec::new();
        let mut last = 0;
        for mat in CMD_REGEX.find_iter(arg) {
            fragments.push(ArgFragment::Literal(arg[last..mat.start()].to_string()));
            fragments.push(ArgFragment::Placeholder(arg[mat.start()+1..mat.end()-1].trim().to_string()));
            last = mat.end()
        }
        fragments.push(ArgFragment::Literal(arg[last..].to_string()));

        ArgTemplate{fragments}
    }
}

impl ArgTemplate {
    fn apply_context(self: &Self, context: &Context) -> String {
        self.fragments.iter()
            .map(|fragment| match *fragment {
                ArgFragment::Literal(ref literal) => literal.as_str(),
                ArgFragment::Placeholder(ref placeholder) => context.get(placeholder).unwrap_or(&""),
            }).collect::<Vec<&str>>().concat()

        // TODO: error handling (lookup fail)
    }
}
