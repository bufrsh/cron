# cron.bufr.sh

A utility web server to translate CRON expressions into plain english. It is
intended to use from the comfort of your terminal, and won't work in a web
browser.

#### Usage
An instance is running at `bufr.sh` port 6000. To use it from the terminal, run:

```
nc bufr.sh 6000
```

Then press Enter. Then type in the CRON expression and press Enter again. A
sample session is shown below:

```
$ nc bufr.sh 6000
1-30/2 1-4/2 1-4/2 1-4/2 mon-fri/2
Run
at every 2nd minute from 1 to 30
past every 2nd hour from 1 to 4
on every 2nd day-of-month from 1 to 4
in every 2nd month from JAN to APR
and
on every 2nd day-of-week from MON to FRI
```

#### Compilation
You will need Rust toolchain to compile it. A simple `cargo build --release`
will compile it.

#### Contribution
PRs are always welcome!

