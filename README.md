**Rargs** is kind of `xargs` + `awk` with pattern-matching support.

![Crates.io](https://img.shields.io/crates/v/rargs.svg) [![Build Status](https://travis-ci.org/lotabout/rargs.svg?branch=master)](https://travis-ci.org/lotabout/rargs)

## Installation

### Using Cargo

```
cargo install --git https://github.com/lotabout/rargs.git
```

## Example usage

### Batch rename files

Suppose you have several backup files whose names match the pattern `<scriptname>.sh.bak`, and you want to map each filename back to `<scriptname>.sh`. We want to do it in a batch, so `xargs` is a natural choice, but how do we specify the name for each file? I believe there is no easy way.

With `rargs`, however, you are able to do:

```sh
ls *.bak | rargs -p '(.*)\.bak' mv {0} {1}
```

Here `{0}` refers to the whole input line, while `{1}` refers to the first group captured in the regular expression.

### Batch download

I had a bunch of URLs and their corresponding target filenames stored in a CSV file:

```
URL1,filename1
URL2,filename2
```

I hoped there was a simple way to download and save each URL with its specified filename. With `rargs` there is:

```sh
cat download-list.csv | rargs -p '(?P<url>.*),(?P<filename>.*)' wget {url} -O {filename}
```

Here `(?P<group_name>...)` assigns the name `group_name` to the captured group. This can then be referred to as `{group_name}` in the command.

### AWK replacement?

Suppose you have an xSV file with lots of columns, and you only want to extract and format some of them, e.g.:

```
nobody:*:-2:-2:Unprivileged User:/var/empty:/usr/bin/false
root:*:0:0:System Administrator:/var/root:/bin/sh
daemon:*:1:1:System Services:/var/root:/usr/bin/false
```

Here's an example of how `rargs` can be used to process it:

```
$ cat /etc/passwd | rargs -d: echo -e 'id: "{1}"\t name: "{5}"\t rest: "{6..::}"'
id: "nobody"     name: "Unprivileged User"       rest: "/var/empty:/usr/bin/false"
id: "root"       name: "System Administrator"    rest: "/var/root:/bin/sh"
id: "daemon"     name: "System Services"         rest: "/var/root:/usr/bin/false"
```

`rargs` allow you to specify the delimiter (regex) to split the input on, and allows you to refer to the corresponding fields or field ranges. This allows it to be used as an AWK replacement for some simple but common cases.

## How does it work?

1. receive the input on stdin and split it into lines
2. split (`-d`) or extract (`-p`) the input into named or numbered groups, with `{0}` matching the whole line
3. map the named and numbered groups into a command passed as the remaining arguments, and execute the command

## Features

### Regexp captures

`rargs` allows you to use any regular expression to match the input, and captures anything you are interested in. The syntax is the standard, mostly Perl-compatible [Rust regex syntax](https://docs.rs/regex/0.2.10/regex/#syntax) used by tools such as [ripgrep](https://github.com/BurntSushi/ripgrep).
- positional (numbered) groups are captured with parentheses, e.g. `'(\w+):(\d+)'`, and the corresponding groups are referred to by `{1}`, `{2}` etc. in the command
- named groups are captured with `(?P<name>...)` and referred to by `{name}` in the command

### Delimiter captures

For simple usage, you might not want to write the whole regular expression to extract parts of the line. All you want is to split the groups by some delimiter. With `rargs` you can achieve this by using the `-d` (delimiter) option.

### Field ranges

We already know how to refer to captures by number (`{1}`) or by name (`{name}`). There are also cases where you might want to substitute multiple fields at the same time. `rargs` also supports this with field-range expressions.

Suppose we have already captured 5 groups representing the strings `1`, `2`, `3`, `4` and `5`

- `{..}` gathers them all into `1 2 3 4 5` (note that they are separated by a space; this can be overridden by the `-s` option)
- `{..3}` results in `1 2 3`
- `{4..}` results in `4 5`
- `{2..4}` results in `2 3 4`
- `{3..3}` results in `3`

You can also specify a "local" separator (which will not affect the global setting):

- `{..3:-}` results in `1-2-3`
- `{..3:/}` results in `1/2/3`

### Multiple threading

You can run commands in multiple threads to improve performance:

- `-w <num>` specifies the number of workers you want to run simultaneously
- `-w 0` defaults the number of workers to the number of CPUs on your system

## Interested?

All feedback and PRs are welcome!
