# EVA-rt Engine

**Evaluation**, **Verification** and **Analysis Engine** for **Real-Time** applications schedulability.

## 🚀 Quick Start

### Prerequisites

- **Rust**: ≥1.85.1

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

# Run UniProcessor Fixed Priority Test
> cargo run -- -i taskset00.txt -a up-fp -n 1
Rate Monotonic - Liu & Layland 1973 (Simplified): Pass
```

```bash
# Help screen (prefer --help to -h as you can get more information)
> cargo run -- --help
Usage: eva-engine-cli [OPTIONS] -i <TASKSET FILE> <-a <ALGORITHM>|-n <n. CPUs>|--test <TEST NAME>|-c <CONFIG FILE>>

Options:
  -q          Quiet mode / Exit code as analysis result
  -h, --help  Print help (see a summary with '-h')

Scheduling Algorithm Specification:
  -a <ALGORITHM>      Scheduling Algorithm
                      [possible values:
                        up-edf,
                        up-fp,
                        global-edf,
                        global-fp]
  -n <n. CPUs>        Number of processors
  --test <TEST NAME>  Specific Test
  -c <CONFIG FILE>    Config file

Taskset Specification:
  -i <TASKSET FILE>     Taskset data file
[...]
```

### Advanced Usage

It is possible to use configuration files to specify which analyses to run, useful for batch modes / scripts and automation. The config file is formatted in *JSON*, and its fields depend on the analysis to run. Refer to the [examples](examples) directory for available fields for each of the given analyses.

```bash
# Example config for UniProcessor Rate Monotonic
> cat examples/up_fixed_priority/rate_monotonic/config_hyperbolic.json
{
    "algorithm": "UpFP",
    "num_processors": 1,
    "specific_test": "rm-hyperbolic"
}

# Run the analyzer with the config
> cargo run -- -i taskset.txt -c config_hyperbolic.json
Rate Monotonic - Bini, Buttazzo, Buttazzo 2001: Pass
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

### Uni-Processor Analyses

*References are available in the individual sub-pages*

- Fixed Priority
  - [**Rate Monotonic**](src/analyses/up_fixed_priority/rate_monotonic/README.md)
  - [**Deadline Monotonic**](src/analyses/up_fixed_priority/deadline_monotonic/README.md)
- [**Earliest Deadline First**](src/analyses/up_earliest_deadline_first/README.md)

### Multi-Processor Analyses

- Global Fixed Priority
  - [**Deadline Monotonic**](src/analyses/smp_fixed_priority/deadline_monotonic/README.md)
- [**Global Earliest Deadline First**](src/analyses/smp_earliest_deadline_first/README.md)

## 📄 License

This project is licensed under the GNU General Public License v3 - see the [LICENSE](LICENSE) file for details.

## 👤 Author

This software was developed by:
- **Yuri Andriaccio** [yurand2000@gmail.com](mailto:yurand2000@gmail.com), [GitHub](https://github.com/Yurand2000).

---

**EVA-rt Engine**