# EVA-rt Engine

**Evaluation**, **Verification** and **Analysis Engine** for **Real-Time** applications schedulability.

## 🎯 What is EVA-rt-Engine?

**EVA-rt-Engine** (short as *EVA*) is a software created to perform real-time schedulability analyses.

EVA implements a variety of *state-of-the-art* tests to assert wheter a given taskset is schedulable on a given platform. Additionally, it also implements designers that search for the minimum required resources to schedule the given task on the given platform and scheduling approach.

EVA is distributed under the **GPL3** license as a Rust library that can be easily integrated in other Rust-based projects.

## 🚀 Quick Start

### Prerequisites

- **Rust**: ≥1.85.1

### Installation

```bash
> cargo add --git https://github.com/Yurand2000/EVA-rt-Engine.git
```

### Usage

Most of the useful information is available in the documentation of the crate, but here is a summary on how the analyzers are organized.

At top level there are two important traits that are implemented by the analyzers: `SchedAnalysis` and `SchedDesign`. All the algorithms implemented in the library are available under the module `algorithms`. The sub-modules specify the target preemption model (for now only `full_preemption`), the platform (`uniprocessor` or `global_multiprocessor`) and the algorithm. The `hierarchical` sub-modules contain different modules which allow hierarchical scheduling of tasksets in different settings.

#### Trait: SchedAnalysis

The first trait, `SchedAnalysis` is implemented by schedulability analyzers, which perform schedulability tests on the given taskset and parameters. As an example, in the module `algorithms::full_preemption::uniprocessor::fixed_priority::rta86` there is the struct `Analysis`, which implements `SchedAnalysis`, and performs the classic response time analysis for fixed priority tasksets on uniprocessor machines:
```rust
fn main() {
  let taskset = [ /* taskset specification */ ];

  let analyzer = algorithms::full_preemption::uniprocessor::
                 fixed_priority::rta86::Analysis;

  println!("{:?}", analyzer.is_schedulable(&taskset));
}
```

#### Trait: SchedDesign

The second trait, `SchedDesign` is instead implemented by schedulability designers, i.e. code which generates the best schedulability parameters for a given algorithm/platform and taskset. As an example, in the module
`algorithms::full_preemption::uniprocessor::hierarchical::pr_model03` there is the definition of `PRModel` Periodic Resource Model for hierarchical scheduling, along with `fixed_priority::shin_lee03::DesignerLinear` which implements `SchedDesign` and performs an analysis to generate the best Periodic Resource Model that schedules the input taskset:
```rust
use algorithms::full_preemption::uniprocessor::
    hierarchical::pr_model03::*;

fn main() {
  let taskset = [ /* taskset specification */ ];

  // design the best model for the given taskset.
  let designer = fixed_priority::shin_lee03::DesignerLinear;
  let model: PRModel = designer.design(&taskset).unwrap();

  println!("{:?}", model);

  // assert that the generated model can really schedule the taskset.
  let analysis = fixed_priority::shin_lee03::Analysis { model };
  assert!(analysis.is_schedulable(&taskset));
}
```

#### Examples

A small set of example code is available in the `examples` directory, which allows to test schedulability of tasksets specified in files, running a bunch of standard schedulability tests.

## 🔬 Projects using this Library

- [HCBS-test-suite](https://github.com/Yurand2000/HCBS-Test-Suite) \
  The test suite for the [Hierarchical Constant Bandwith Server](https://github.com/Yurand2000/HCBS-patch) patch for the Linux kernel, which generates tests using the hierarchical Multiprocessor Periodic Resource model.

## 🛠️ Future Work

- [X] Documentation
- [ ] Example GUI interface
- [ ] [📦 crates.io](crate.io) release
- [ ] *More preemption models...*
- [ ] *More platforms...*
- [ ] *More analyses and designers...*

## 📄 License

This project is licensed under the GNU General Public License v3 - see the [LICENSE](LICENSE) file for details.

## 👤 Author

This software was developed by:
- **Yuri Andriaccio** [yurand2000@gmail.com](mailto:yurand2000@gmail.com), [GitHub](https://github.com/Yurand2000).

---

**EVA-rt Engine**