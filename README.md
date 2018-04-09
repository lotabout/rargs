**R**egex **args** arms xargs with pattern matching.

```
rargs is still in the planning phase, suggestions are welcome
```

## Use Cases

Do you miss regex while using `xargs`?

### Batch rename files

Suppose you had several backup files that named under the pattern
`scriptname.sh.bak` and you hope to recover them back to `scriptname.sh`.

We want to do it in batch, so `xargs` is the first thought, but how do we
specify the name for the batch? I believe there is no easy way.

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
name correctly).

With `rargs` there is a simple way:

```sh
cat download-list | rargs '(?P<url>.*),(?P<filename>.*)' wget {url} -O {filename}
```

Here `(?P<group_name>...)` will assign name `group_name` for the captured
group. They can later be refered by `{group_name}`.

## Roadmap

- [X] Able to run the commands in batch. (do not support `-n` in `xargs`)
- [x] Able to capture groups and use the in commands
- [x] Able to capture named groups
- [x] Able to specify the delimiter to split the input. (e.g. `-0` in `xargs`)

### Maybe later

- [ ] Enhance the capturing syntax. e.g. awk like?
- [ ] run the commands in parallel.
- [ ] parallel like placeholders?
