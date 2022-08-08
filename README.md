# irename
Interactive rename tool

[![demo](https://asciinema.org/a/3q2rKc6Z5WzmfHHuRVleA4AeG.svg)](https://asciinema.org/a/3q2rKc6Z5WzmfHHuRVleA4AeG)

## Usage

### Examples

Rename a bunch of files
```
# run renamer on all .txt files in some/dir
irename some/dir/*.txt
```

Execute in `dry-run` mode
```
# run renamer on all .txt files in some/dir and only print generated commands w/o actual renaming
irename --dry-run some/dir/*.txt
```

Output of `dry-run` mode can be piped
```
# generate rename commands and execute them by yourself
irename --dry-run some/dir/*.txt | parallel -n1
```

**P.S.: The app will exit with an error if there are some conflicting names.
It won't destruct your files as `GNU rename` does**

### Cli help output
```
irename 0.1.0

USAGE:
    irename [OPTIONS] <FILES>...

ARGS:
    <FILES>...

OPTIONS:
        --dry-run          only print shell commands w/o executing them
    -h, --help             Print help information
        --regex <REGEX>    Initial replacement regex
    -V, --version          Print version information
```


### Shortcuts

- `Tab` - switch between `regex` and `replacement` text input areas
- `Enter` - execute renaming
- `Ctrl-c` - exit


## TODO

- [ ] Read input files paths from stdin if no positional args are supplied
- [ ] Files list scrolling with `Ctrl-d/Ctrl-u`
- [ ] Highlight for conflicting names
- [ ] Help side-pane
- [ ] Full-path mode switch for editing the whole path instead of just filename
- [ ] Docs
