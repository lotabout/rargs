#[macro_use]extern crate structopt;
#[macro_use]extern crate lazy_static;

extern crate regex;
extern crate num_cpus;
extern crate threadpool;

use structopt::StructOpt;
use structopt::clap::AppSettings;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use regex::Regex;
use std::collections::HashMap;
use std::convert::From;
use threadpool::ThreadPool;
use std::sync::Arc;
use std::borrow::Cow;
use std::cmp::max;

fn main() {
    let mut exit_code = 0;

    let options = Options::from_args();
    let rargs = Arc::new(Rargs::new(&options));

    let stdin = io::stdin();

    let num_worker = if options.worker > 0 {options.worker} else {num_cpus::get()};
    let num_threads = if options.threads > 0 {options.threads} else {num_worker};

    let pool = ThreadPool::new(num_threads);

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
    static ref CMD_REGEX: Regex = Regex::new(r"\{[[:space:]]*[^{}]*[[:space:]]*\}").unwrap();

    static ref FIELD_NAMED: Regex = Regex::new(r"^\{[[:space:]]*(?P<name>[[:word:]]*)[[:space:]]*\}$").unwrap();
    static ref FIELD_SINGLE: Regex = Regex::new(r"^\{[[:space:]]*(?P<num>-?\d+)[[:space:]]*\}$").unwrap();
    static ref FIELD_RANGE: Regex = Regex::new(r"^\{(?P<left>-?\d*)?\.\.(?P<right>-?\d*)?(?::(?P<sep>.*))?\}$").unwrap();
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Rargs", about = "Xargs with pattern matching")]
#[structopt(raw(settings = "&[AppSettings::TrailingVarArg]"))]
struct Options {
    #[structopt(long = "read0", short = "0",
                help = "Read input delimited by ASCII NUL(\\0) characters")]
    read0: bool,

    #[structopt(long = "worker", short = "w", default_value = "1",
                help = "Deprecated. Number of threads to be used (same as --threads)")]
    worker: usize,

    #[structopt(long = "threads", short = "j", default_value = "1",
                help = "Number of threads to be used")]
    threads: usize,

    #[structopt(long = "pattern", short = "p", help = "regex pattern that captures the input")]
    pattern: Option<String>,

    #[structopt(long = "separator", short = "s", default_value = " ",
                help = "separator for ranged fields")]
    separator: String,

    #[structopt(long = "delimiter", short = "d", conflicts_with = "pattern",
                help = "regex pattern used as delimiter (conflict with pattern)")]
    delimiter: Option<String>,

    #[structopt(raw(required = "true"), help = "command to execute and its arguments")]
    cmd_and_args: Vec<String>,
}

#[derive(Debug)]
struct Rargs {
    pattern: Regex,
    command: String,
    args: Vec<ArgTemplate>,
    default_sep: String,  // for output range fields
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

        let command = opts.cmd_and_args[0].to_string();
        let args = opts.cmd_and_args[1..].iter().map(|s| ArgTemplate::from(&**s)).collect();
        let default_sep = opts.separator.clone();

        Rargs{pattern, command, args, default_sep}
    }

    fn execute_for_input(&self, line: &str) {
        let context = RegexContext::build(&self.pattern, line, Cow::Borrowed(&self.default_sep));
        let args: Vec<String> = self.args.iter().map(|arg| arg.apply_context(&context)).collect();

        Command::new(&self.command)
            .args(args)
            .stdin(Stdio::null())
            .status()
            .expect("command failed to start");
    }
}

trait Context<'a> {
    fn get_by_name(&'a self, group_name: &str) -> Option<Cow<'a, str>>;
    fn get_by_range(&'a self, range: &Range, sep: Option<&str>) -> Option<Cow<'a, str>>;
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
struct RegexContext<'a> {
    map: HashMap<String, Cow<'a, str>>,
    groups: Vec<Cow<'a, str>>,
    default_sep: Cow<'a, str>,
}

impl<'a> RegexContext<'a> {
    fn build(pattern: &'a Regex, content: &'a str, default_sep: Cow<'a, str>) -> Self {
        let mut map = HashMap::new();
        map.insert("".to_string(), Cow::Borrowed(content));
        map.insert("0".to_string(), Cow::Borrowed(content));

        let group_names = pattern.capture_names()
            .filter_map(|x| x)
            .collect::<Vec<&str>>();

        let mut groups = vec![];

        for caps in pattern.captures_iter(content) {
            // the numbered group
            for mat_wrapper in caps.iter().skip(1) {
                if let Some(mat) = mat_wrapper {
                    groups.push(Cow::Borrowed(mat.as_str()));
                }
            }

            // the named group
            for name in group_names.iter() {
                if let Some(mat) = caps.name(name) {
                    map.insert(name.to_string(), Cow::Borrowed(mat.as_str()));
                }
            }
        }

        RegexContext{map, groups, default_sep}
    }

    fn translate_neg_index(&self, idx: i32) -> usize {
        let len = self.groups.len() as i32;
        let idx = if idx < 0 {idx + len + 1} else {idx};
        max(0, idx) as usize
    }
}

impl<'a> Context<'a> for RegexContext<'a> {
    fn get_by_name(&'a self, group_name: &str) -> Option<Cow<'a, str>> {
        self.map.get(group_name).map(|c| c.clone())
    }

    fn get_by_range(&'a self, range: &Range, sep: Option<&str>) -> Option<Cow<'a, str>> {
        match *range {
            Single(num) => {
                let num = self.translate_neg_index(num);

                if num == 0 {
                    return self.map.get("").map(|c| c.clone());
                } else if num > self.groups.len() {
                    return None;
                }

                let x = Some(self.groups[num -1].clone());
                return x;
            }

            Both(left, right) => {
                let left = self.translate_neg_index(left);
                let right = self.translate_neg_index(right);

                if left == 0 {
                    return self.get_by_range(&LeftInf(right as i32), sep)
                } else if right > self.groups.len() {
                    return self.get_by_range(&RightInf(left as i32), sep)
                } else if left == right {
                    return self.get_by_range(&Single(left as i32), sep)
                }

                Some(Cow::Owned(self.groups[(left-1)..right].join(sep.unwrap_or(&self.default_sep))))
            }

            LeftInf(right) => {
                let right = self.translate_neg_index(right);
                if right > self.groups.len() {
                    return self.get_by_range(&Inf(), sep)
                }

                Some(Cow::Owned(self.groups[..right].join(sep.unwrap_or(&self.default_sep))))
            }

            RightInf(left) => {
                let left = self.translate_neg_index(left);
                if left == 0 {
                    return self.get_by_range(&Inf(), sep)
                }

                Some(Cow::Owned(self.groups[(left-1)..].join(sep.unwrap_or(&self.default_sep))))
            }

            Inf() => {
                Some(Cow::Owned(self.groups.join(sep.unwrap_or(&self.default_sep))))
            }
        }
    }
}

#[derive(Debug)]
enum Range {
    Single(i32),
    Both(i32, i32),
    LeftInf(i32),
    RightInf(i32),
    Inf(),
}

use Range::*;

#[derive(Debug)]
enum ArgFragment{
    Literal(String),
    NamedGroup(String),
    RangeGroup(Range, Option<String>),
}

use ArgFragment::*;

impl ArgFragment {
    fn parse(field_string: &str) -> Self {
        let opt_caps = FIELD_SINGLE.captures(field_string);
        if let Some(caps) = opt_caps {
            return RangeGroup(Single(caps.name("num")
                                     .expect("something is wrong in matching FIELD_SINGLE")
                                     .as_str()
                                     .parse()
                                     .expect("field is not a number")),
                              None);
        }

        let opt_caps = FIELD_NAMED.captures(field_string);
        if let Some(caps) = opt_caps {
            return NamedGroup(caps.name("name")
                              .expect("something is wrong in matching FIELD_NAMED")
                              .as_str()
                              .to_string());
        }


        let opt_caps = FIELD_RANGE.captures(field_string);
        if let Some(caps) = opt_caps {
            let opt_left = caps.name("left").map(|s| s.as_str().parse().unwrap_or(1));
            let opt_right = caps.name("right").map(|s| s.as_str().parse().unwrap_or(-1));
            let opt_sep = caps.name("sep").map(|s| s.as_str().to_string());

            if opt_left.is_none() && opt_right.is_none() {
                return RangeGroup(Inf(), opt_sep);
            } else if opt_left.is_none() {
                return RangeGroup(LeftInf(opt_right.unwrap()), opt_sep);
            } else if opt_right.is_none() {
                return RangeGroup(RightInf(opt_left.unwrap()), opt_sep);
            } else {
                return RangeGroup(Both(opt_left.unwrap(), opt_right.unwrap()), opt_sep);
            }
        }

        return Literal(field_string.to_string());
    }
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
            fragments.push(Literal(arg[last..mat.start()].to_string()));
            fragments.push(ArgFragment::parse(mat.as_str()));
            last = mat.end()
        }
        fragments.push(ArgFragment::Literal(arg[last..].to_string()));

        ArgTemplate{fragments}
    }
}

impl<'a> ArgTemplate {
    fn apply_context<T: Context<'a>>(&self, context: &'a T) -> String {
        self.fragments.iter()
            .map(|fragment| match *fragment {
                Literal(ref literal) => Cow::Borrowed(literal.as_str()),
                NamedGroup(ref name) => context.get_by_name(name).unwrap_or(Cow::Borrowed("")),
                RangeGroup(ref range, ref opt_sep) => {
                    context.get_by_range(range, opt_sep.as_ref().map(|s| &**s))
                        .unwrap_or(Cow::Borrowed(""))
                }
            }).collect::<Vec<Cow<str>>>().concat()
    }
}
