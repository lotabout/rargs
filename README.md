**Rargs** is kind of `xargs` + `awk` with pattern matching support.

## Installation

### Using Cargo

```
cargo install --git https://github.com/lotabout/rargs.git
```

## Example usage

### Batch rename files

Suppose you had several backup files that named under the pattern
`scriptname.sh.bak` and you hope to recover them back to `scriptname.sh`.  We
want to do it in batch, so `xargs` is the first thought, but how do we specify
the name for the batch? I believe there is no easy way.

With `rargs` however, you are able to do:

```sh
ls *.bak | rargs '(.*)\.bak' mv {0} {1}
```

Here `{0}` refers to the whole input string, while `{1}` refers to the first
group captured in the regular expression.

### Batch Download

I had a bunch of URLs and their corresponding target filenames stored as csv:

```
URL1,filename1
URL2,filename2
```

I just hope there would be a simple way to download all of them and sotred
them with the filename(Unfortunatelly the `wget` the URL won't resolve the
name correctly). With `rargs` there is a simple way:

```sh
cat download-list | rargs '(?P<url>.*),(?P<filename>.*)' wget {url} -O {filename}
```

Here `(?P<group_name>...)` will assign name `group_name` for the captured
group. They can later be refered by `{group_name}`.

### awk replacement?

Suppose you have a csv file with lots of columns, and you want only some of
them. Here is some line that taken from `/etc/passwd`, and you can play around
with `rargs`.

```
nobody:*:-2:-2:Unprivileged User:/var/empty:/usr/bin/false
root:*:0:0:System Administrator:/var/root:/bin/sh
daemon:*:1:1:System Services:/var/root:/usr/bin/false
```

For example:

```
$ cat filename | rargs -d : echo -e 'id: "{1}"\t name: "{5}"\t rest: "{6..::}"'
id: "nobody"     name: "Unprivileged User"       rest: "/var/empty:/usr/bin/false"
id: "root"       name: "System Administrator"    rest: "/var/root:/bin/sh"
id: "daemon"     name: "System Services"         rest: "/var/root:/usr/bin/false"
```

`rargs` allow you to specify the delimiter(regex) to split the input and
allows you to refer to the field or field range. Which could make it an awk
replacement for simple usage.

## How does it work?

1. receive the input from stdin and split them into lines.
2. match the input with the regex pattern specified by `-p` or `-d` and
   collect the matched groups(e.g. the ones wrapped with `()`).
    - If we specify the pattern `(.*).bak`, and for the input
      `scriptname.sh.bak` then we capture the groups: `{1}`: `scriptname.sh`.
      And `{0}` will always be the whole input.
    - If we specify the delimiter `[, ]` and given input `1, 2,3`
      the captured groups are `{1}: 1`, `{2}: 2`, `{3}: 3`
3. For each line, expand the command arguments with the captured groups:
    - if we want to execute `mv {0} {1}` the expanded command would be `mv scriptname.sh.bak scriptname.sh`

## Features

### Regexp Captures

`rargs` allows you to use any regular expression to match the input and
captures anything you are interested in. The syntax is the same as normal
regular expression.

- capture group with `(...)`, later you can refer to it as `{1}` (the number
    will increase by `1` for any group captured.)
- named group can be captured by `(?P<name>...)` and later refered by
    `{name}` in the command's arguments.

### Delimiter Captures

For simple usage, you might not want to write the whole regular expression to
captures, all you want is to split the groups by some delimiter. With `rargs`
you could easily achieve it by specifying `-d ...`.

### Field Ranges

We already know we can refer to captures by number(`{1}`) or by
name(`{name}`). There are also cases that you might want to refer to a bunch
of fields all at once. `rargs` will help you to do so.

Suppose we already captured 5 groups: `1, 2, 3, 4, 5`

- `{..}` will grab them all into `1 2 3 4 5` (note that they are separated by
    space which could be overwritten by `-s ...`)
- `{..3}` will result in `1 2 3`
- `{4..}` will result in `4 5`
- `{2..4}` will result in `2 3 4` as you would expect.
- `{3..3}` will result in `3`

you could also speficy the "local" separator(which will not affect the global
setting):
- `{..3:-}` will result in `1-2-3`
- `{..3:/}` will result in `1/2/3`

### Multiple Threading

You could run the commands in multiple thread to speed up:

- `-w <num>` to specify the number of workers you want to run simultaneously
- `-w 0` will default the worke number to the number of your cpu.

## Interested?

All feedback and PRs are welcome!
