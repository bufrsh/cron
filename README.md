# cron.bufr.sh

A utility server to translate CRON expressions into plain english. It is
intended to use from the comfort of your terminal, and won't work in a web
browser.

#### Usage
An instance is running at `bufr.sh` port 6000. To use it from the terminal, run:

```
nc bufr.sh 6000
```

Then press Enter. Then type in the CRON expression and press Enter again. A
sample session is shown below:

[![asciicast](https://asciinema.org/a/355011.svg)](https://asciinema.org/a/355011)

#### Compilation
You will need Rust toolchain to compile it. A simple `cargo build --release`
will compile it.

#### Contribution
PRs are always welcome!

