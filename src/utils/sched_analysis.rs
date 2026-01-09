use crate::prelude::*;
use anyhow::Context as _;

pub trait SchedAnalysis<T, Taskset> {
    /// Name of the schedulability analysis.
    fn analyzer_name(&self) -> &str;

    /// Check if the taskset matches the preconditions necessary to run the analysis.
    fn check_preconditions(&self, taskset: &Taskset) -> Result<(), SchedError>;

    /// Run the schedulability test.
    fn run_test(&self, taskset: Taskset) -> Result<T, SchedError>;

    /// Check if the taskset matches the precondtions and run the schedulability test.
    fn is_schedulable(&self, taskset: Taskset) -> anyhow::Result<T> {
        self.check_preconditions(&taskset)
            .with_context(|| format!("Precondition check error for \"{}\"", self.analyzer_name()))?;

        self.run_test(taskset)
            .with_context(|| format!("Schedulability test error for \"{}\"", self.analyzer_name()))
    }
}