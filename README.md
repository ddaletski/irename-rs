# irename

[![Crate](https://img.shields.io/crates/v/irename.svg)](https://crates.io/crates/irename)
[![API](https://docs.rs/irename/badge.svg)](https://docs.rs/irename)


Interactive rename tool

[![demo](https://asciinema.org/a/3q2rKc6Z5WzmfHHuRVleA4AeG.svg)](https://asciinema.org/a/3q2rKc6Z5WzmfHHuRVleA4AeG)

## Installation

```shell
cargo install irename
```

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

If no positional args are provided, the files list is read from `stdin`
```
# find files using your favorite tool (e.g. `fd`) and pipe the file list into renamer
fd \.rs | irename
```

**P.S.: The app will exit with an error if there are some conflicting names.
It won't destruct your files as `GNU rename` does**

### Cli help output
```
USAGE:
    irename [OPTIONS] [FILES]...

ARGS:
    <FILES>...    files to rename. If none provided, the files list will be read from stdin

OPTIONS:
        --dry-run              only print shell commands w/o executing them
    -h, --help                 Print help information
        --regex <REGEX>        Initial regex
        --replace <REPLACE>    Initial replacement string
    -V, --version              Print version information
```


### Shortcuts

- `Tab` - switch between `regex` and `replacement` text input areas
- `Enter` - execute renaming
- `Ctrl-c` - exit
- `Ctrl-g` - toggle 'global' flag
- `Ctrl-r` - toggle 'ignore case' flag


## TODO

- [x] Read input files paths from stdin if no positional args are supplied
- [x] Help side-pane
- [x] Match flags
- [ ] Files list scrolling with `Ctrl-d/Ctrl-u`
- [ ] Highlight for conflicting names
- [ ] Full-path mode switch for editing the whole path instead of just filename
- [ ] Docs
