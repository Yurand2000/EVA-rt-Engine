# EVA-rt Engine

**Evaluation**, **Verification** and **Analysis Engine** for **Real-Time** applications schedulability.

## 🚀 Quick Start

### Prerequisites

- **Rust**: ≥1.?.?-nightly

### Installation

Standard **Rust** based software:

```bash
> cargo build (--release)
```

### Basic Usage

The analysis software requires an *input file* describing the *taskset* (available formats [here](#taskset-formats)) to analyze and through the command line arguments (or a configuration file, see [advanced usage](#advanced-usage)), it is possible to specify which analysis to run.

*The following examples make use of the `cargo run` command instead of invoking the executable directly. Cargo will build and run the application with the given parameters. Refer to the documentation of `cargo install` to install the software or run the executable from the `target` folder after building.*

```bash
# Let taskset00.txt be a file describing your taskset
> cat taskset00.txt
10 20 20
1 30 30

# Run UniProcessor Rate Monotonic test
> cargo run -- -i taskset00.txt rate-monotonic
Analysis Output: Schedulable

# Some of the analyses may have additional options
> cargo run -- rate-monotonic --help
UniProcessor Rate Monotonic
[...]
Usage: analyzer -i <filename> rate-monotonic [type]
Arguments:
  [type]    Analysis to run
            [default: classic]
            [possible values: classic, simple, hyperbolic]
[...]

> cargo run -- -i taskset00.txt rate-monotonic hyperbolic
Analysis Output: Schedulable
```

```bash
# Help screen (prefer --help to -h as you can get more information)
> cargo run -- --help
Usage: analyzer [OPTIONS] -i <filename> [COMMAND]
Commands:
  rate-monotonic  UniProcessor Rate Monotonic
  [...]
  help            Print this message or the help of the given subcommand(s)
Options:
  -i <filename>   Taskset data file
[...]
```

### Advanced Usage

It is possible to use configuration files to specify which analyses to run, useful for batch modes / scripts and automation. The config file is formatted in *JSON*, and its fields depend on the analysis to run. Refer to the [examples](examples) directory for available fields for each of the given analyses.

```bash
# Example config for UniProcessor Rate Monotonic
> cat examples/up_rate_monotonic/config_hyperbolic.json
{
    "rate-monotonic": {
        "typ": "hyperbolic"
    }
}

# Run the analyzer with the config
> cargo run -- -i taskset.txt -c config_hyperbolic.json
Analysis Output: Schedulable
```

Another interesting command line option is `-q` for *quiet*, which is still useful for scripts and automatic tasks. Basically, the option suppresses the output to *stdout* and *stderr* (unless a argument error occurs), and the analyzer exits with **0** (zero) when the taskset is deemed schedulable, **1** when it is not schedulable, and any other exit code to signal taskset parsing errors or other analysis specific errors (as an example, unmatched preconditions for certain analyses).

```bash
# Run the software in quiet mode
> cargo run -- -q -i taskset.txt -c config_hyperbolic.json

# Echo the exit code
> echo $?
0
```

For other minor command line options, run the program with `-h` or `--help`.

### Taskset Formats

Currently, the software support two taskset formats:
- [**Plain-text**](#plain-text)
- [**JSON**](#json)

#### Plain-Text

Each line in the file represents a task. In case of fixed-priority scheduling, the tasks are assumed to be ordered as given.

Each task is described by three numbers: **Worst Case Execution Time**, **Relative Deadline**, **Relative Period**. All these times are assumed to be in *milliseconds*.

Example Taskset in plain format:

```bash
> cat taskset.txt
10 20 50
30 40 40
2 120 120
```

This taskset is comprised of 3 tasks. The highest priority task (if using *fixed-priority scheduling*) has a **WCET** of 10ms, **Deadline** of 20 ms, **Period** of 50ms, and so on...

#### JSON

🚧 Under Construction 🚧

## 🔬 Available Analyses

### Single Processor Analyses

*References are available in the individual sub-pages*

- [**Rate Monotonic**](src/analyses/up_rate_monotonic/README.md)
- [**Earliest Deadline First**](src/analyses/up_earliest_deadline_first/README.md)
- [**Deadline Monotonic**](src/analyses/up_deadline_monotonic/README.md)
- [**Response Time Analysis**](src/analyses/response_time_analysis/README.md)

### Multi Processor Analyses

- [**Global Deadline Monotonic**](src/analyses/smp_dm/README.md)
- [**Global Earliest Deadline First**](src/analyses/smp_edf/README.md)

## 📄 License

This project is licensed under the GNU General Public License v3 - see the [LICENSE](LICENSE) file for details.

## 👤 Author

This software was developed by:
- **Yuri Andriaccio** [yurand2000@gmail.com](mailto:yurand2000@gmail.com), [GitHub](https://github.com/Yurand2000).

---

**EVA-rt Engine**