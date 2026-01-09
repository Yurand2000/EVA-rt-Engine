use crate::prelude::*;
use anyhow::Context as _;

pub trait SchedDesign<Taskset, Model> {
    /// Name of the schedulability analysis.
    fn designer_name(&self) -> &str;

    /// Check if the taskset matches the preconditions necessary to run the designer.
    fn check_preconditions(&self, taskset: &Taskset) -> Result<(), SchedError>;

    /// Run the designer.
    fn run_designer(&self, taskset: Taskset) -> Result<Model, SchedError>;

    /// Check if the taskset matches the precondtions and run the schedulability test.
    fn design(&self, taskset: Taskset) -> anyhow::Result<Model> {
        self.check_preconditions(&taskset)
            .with_context(|| format!("Precondition check error for \"{}\"", self.designer_name()))?;

        self.run_designer(taskset)
            .with_context(|| format!("Designer error for \"{}\"", self.designer_name()))
    }
}