use crate::prelude::*;
use super::MPRModel;

use anyhow::Context as _;

pub struct DesignerPeriodConcurrencyNaive<'a, FnA, A>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A,
{
    pub period: Time,
    pub concurrency: u64,
    pub resource_range: (Time, Time, Time),
    pub analysis_gen_fn: FnA,
    pub marker: std::marker::PhantomData<&'a [RTTask]>,
}

impl<'a, FnA, A> SchedDesign<&'a [RTTask], MPRModel> for DesignerPeriodConcurrencyNaive<'a, FnA, A>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A,
{
    fn designer_name(&self) -> &str { "MPR Model designer from period and concurrency" }

    fn check_preconditions(&self, _: &&'a [RTTask]) -> Result<(), SchedError> {
        Err(SchedError::Other(
            anyhow::format_err!("This generic implementor of SchedDesign cannot check for preconditions")
        ))
    }

    fn run_designer(&self, taskset: &'a [RTTask]) -> Result<MPRModel, SchedError> {
        time_range_iterator_w_step(self.resource_range.0, self.resource_range.1, self.resource_range.2)
        .find_map(|resource| {
            let analysis = (self.analysis_gen_fn)(resource, self.period, self.concurrency);

            if analysis.is_schedulable(taskset).is_ok() {
                Some(MPRModel { resource, period: self.period, concurrency: self.concurrency })
            } else {
                None
            }
        })
        .ok_or(SchedError::NonSchedulable(None))
    }

    fn design(&self, taskset: &'a [RTTask]) -> anyhow::Result<MPRModel> {
        self.run_designer(taskset)
            .with_context(|| std::format!("Designer error for \"{}\"", self.designer_name()))
    }
}

pub struct DesignerPeriodNaive<'a, FnA, A>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A + Clone,
{
    pub period: Time,
    pub concurrency_range: (u64, u64),
    pub resource_range: (Time, Time, Time),
    pub analysis_gen_fn: FnA,
    pub marker: std::marker::PhantomData<&'a [RTTask]>,
}

impl<'a, FnA, A> SchedDesign<&'a [RTTask], MPRModel> for DesignerPeriodNaive<'a, FnA, A>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A + Clone,
{
    fn designer_name(&self) -> &str { "MPR Model designer from period" }

    fn check_preconditions(&self, _: &&'a [RTTask]) -> Result<(), SchedError> {
        Err(SchedError::Other(
            anyhow::format_err!("This generic implementor of SchedDesign cannot check for preconditions")
        ))
    }

    fn run_designer(&self, taskset: &'a [RTTask]) -> Result<MPRModel, SchedError> {
        (self.concurrency_range.0 ..= self.concurrency_range.1)
            .find_map(|concurrency| {
                (DesignerPeriodConcurrencyNaive {
                    period: self.period,
                    concurrency: concurrency,
                    resource_range: self.resource_range,
                    analysis_gen_fn: self.analysis_gen_fn.clone(),
                    marker: std::marker::PhantomData,
                })
                .run_designer(taskset).ok()
            })
            .ok_or(SchedError::NonSchedulable(None))
    }

    fn design(&self, taskset: &'a [RTTask]) -> anyhow::Result<MPRModel> {
        self.run_designer(taskset)
            .with_context(|| std::format!("Designer error for \"{}\"", self.designer_name()))
    }
}

pub struct DesignerNaive<'a, FnA, A>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A + Clone,
{
    pub period_range: (Time, Time, Time),
    pub concurrency_range: (u64, u64),
    pub resource_range: (Time, Time, Time),
    pub analysis_gen_fn: FnA,
    pub marker: std::marker::PhantomData<&'a [RTTask]>,
}

impl<'a, FnA, A> SchedDesign<&'a [RTTask], MPRModel> for DesignerNaive<'a, FnA, A>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A + Clone,
{
    fn designer_name(&self) -> &str { "MPR Model designer" }

    fn check_preconditions(&self, _: &&'a [RTTask]) -> Result<(), SchedError> {
        Err(SchedError::Other(
            anyhow::format_err!("This generic implementor of SchedDesign cannot check for preconditions")
        ))
    }

    fn run_designer(&self, taskset: &'a [RTTask]) -> Result<MPRModel, SchedError> {
        time_range_iterator_w_step(self.period_range.0, self.period_range.1, self.period_range.2)
            .find_map(|period| {
                (DesignerPeriodNaive {
                    period,
                    concurrency_range: self.concurrency_range,
                    resource_range: self.resource_range,
                    analysis_gen_fn: self.analysis_gen_fn.clone(),
                    marker: std::marker::PhantomData,
                })
                .run_designer(taskset).ok()
            })
            .ok_or(SchedError::NonSchedulable(None))
    }

    fn design(&self, taskset: &'a [RTTask]) -> anyhow::Result<MPRModel> {
        self.run_designer(taskset)
            .with_context(|| std::format!("Designer error for \"{}\"", self.designer_name()))
    }
}