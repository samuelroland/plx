# PLX CLI

## Introduction
This CLI contains several parts: the live server + some parse utilities.

## Design

```sh
> plx
Usage: plx <COMMAND>

Commands:
  server  Starts the live server
  parse   Parse the given DY file or parse the course.dy inside given folder
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Live server

```sh
> plx server
Started PLX server on port 9120
```

### PLX DY parser

Parse the content of the current folder
```sh
plx parse .
```

Parse the content of the given folder
```sh
plx parse great_plx_course
```

Parse the course
```sh
plx parse course.dy
# OR
plx parse great_plx_course/course.dy
```

Parse the list of skills
```sh
plx parse skills.dy
# OR
plx parse great_plx_course/skills.dy
```

Parse a given exo file
```sh
plx parse intro/greet-me/exo.dy
```

Note: it's not possible to parse more than one file at a time. See full mode if you need a single command for that.

**Full mode**: parse the course + all its skills + all the exos

```sh
plx parse --full great_plx_course
```

## Exit status
- 0 = all good
- 1 = Fatal error during parsing
- 2 = Parse errors detected and displayed

Errors are always displayed on `stderr`. JSON output is only on `stdout`. When there is JSON output, no other content will be present in `stdout`, success messages are passed in `stderr`.
