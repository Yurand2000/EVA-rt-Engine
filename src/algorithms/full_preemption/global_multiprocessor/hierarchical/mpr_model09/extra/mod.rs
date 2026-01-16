use crate::prelude::*;
use super::MPRModel;

use anyhow::Context as _;

pub struct DesignerPeriodConcurrencyNaive<'a, FnA, A, FnR>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A,
        FnR: Fn(Time, u64) -> Result<Box<dyn Iterator<Item = Time>>, SchedError>,
{
    pub period: Time,
    pub concurrency: u64,
    pub resource_iter_fn: FnR,
    pub analysis_gen_fn: FnA,
    pub marker: std::marker::PhantomData<&'a [RTTask]>,
}

impl<'a, FnA, A, FnR> SchedDesign<&'a [RTTask], MPRModel> for DesignerPeriodConcurrencyNaive<'a, FnA, A, FnR>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A,
        FnR: Fn(Time, u64) -> Result<Box<dyn Iterator<Item = Time>>, SchedError>,
{
    fn designer_name(&self) -> &str { "MPR Model designer from period and concurrency" }

    fn check_preconditions(&self, _: &&'a [RTTask]) -> Result<(), SchedError> {
        Err(SchedError::Other(
            anyhow::format_err!("This generic implementor of SchedDesign cannot check for preconditions")
        ))
    }

    fn run_designer(&self, taskset: &'a [RTTask]) -> Result<MPRModel, SchedError> {
        (self.resource_iter_fn)(self.period, self.concurrency)?
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

pub struct DesignerPeriodNaive<'a, FnA, A, FnR, FnC>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A + Clone,
        FnR: Fn(Time, u64) -> Result<Box<dyn Iterator<Item = Time>>, SchedError> + Clone,
        FnC: Fn(Time) -> Result<Box<dyn Iterator<Item = u64>>, SchedError>,
{
    pub period: Time,
    pub concurrency_iter_fn: FnC,
    pub resource_iter_fn: FnR,
    pub analysis_gen_fn: FnA,
    pub marker: std::marker::PhantomData<&'a [RTTask]>,
}

impl<'a, FnA, A, FnR, FnC> SchedDesign<&'a [RTTask], MPRModel> for DesignerPeriodNaive<'a, FnA, A, FnR, FnC>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A + Clone,
        FnR: Fn(Time, u64) -> Result<Box<dyn Iterator<Item = Time>>, SchedError> + Clone,
        FnC: Fn(Time) -> Result<Box<dyn Iterator<Item = u64>>, SchedError>,
{
    fn designer_name(&self) -> &str { "MPR Model designer from period" }

    fn check_preconditions(&self, _: &&'a [RTTask]) -> Result<(), SchedError> {
        Err(SchedError::Other(
            anyhow::format_err!("This generic implementor of SchedDesign cannot check for preconditions")
        ))
    }

    fn run_designer(&self, taskset: &'a [RTTask]) -> Result<MPRModel, SchedError> {
        (self.concurrency_iter_fn)(self.period)?
        .find_map(|concurrency| {
            (DesignerPeriodConcurrencyNaive {
                period: self.period,
                concurrency: concurrency,
                resource_iter_fn: self.resource_iter_fn.clone(),
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

pub struct DesignerNaive<'a, FnA, A, FnR, FnC, FnP>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A + Clone,
        FnR: Fn(Time, u64) -> Result<Box<dyn Iterator<Item = Time>>, SchedError> + Clone,
        FnC: Fn(Time) -> Result<Box<dyn Iterator<Item = u64>>, SchedError> + Clone,
        FnP: Fn() -> Result<Box<dyn Iterator<Item = Time>>, SchedError>,
{
    pub period_iter_fn: FnP,
    pub concurrency_iter_fn: FnC,
    pub resource_iter_fn: FnR,
    pub analysis_gen_fn: FnA,
    pub marker: std::marker::PhantomData<&'a [RTTask]>,
}

impl<'a, FnA, A, FnR, FnC, FnP> SchedDesign<&'a [RTTask], MPRModel> for DesignerNaive<'a, FnA, A, FnR, FnC, FnP>
    where
        A: SchedAnalysis<(), &'a [RTTask]>,
        FnA: Fn(Time, Time, u64) -> A + Clone,
        FnR: Fn(Time, u64) -> Result<Box<dyn Iterator<Item = Time>>, SchedError> + Clone,
        FnC: Fn(Time) -> Result<Box<dyn Iterator<Item = u64>>, SchedError> + Clone,
        FnP: Fn() -> Result<Box<dyn Iterator<Item = Time>>, SchedError>,
{
    fn designer_name(&self) -> &str { "MPR Model designer" }

    fn check_preconditions(&self, _: &&'a [RTTask]) -> Result<(), SchedError> {
        Err(SchedError::Other(
            anyhow::format_err!("This generic implementor of SchedDesign cannot check for preconditions")
        ))
    }

    fn run_designer(&self, taskset: &'a [RTTask]) -> Result<MPRModel, SchedError> {
        (self.period_iter_fn)()?
        .flat_map(|period| {
            (DesignerPeriodNaive {
                period,
                concurrency_iter_fn: self.concurrency_iter_fn.clone(),
                resource_iter_fn: self.resource_iter_fn.clone(),
                analysis_gen_fn: self.analysis_gen_fn.clone(),
                marker: std::marker::PhantomData,
            })
            .run_designer(taskset).ok()
        })
        .min_by_key(|model| model.resource)
        .ok_or(SchedError::NonSchedulable(None))
    }

    fn design(&self, taskset: &'a [RTTask]) -> anyhow::Result<MPRModel> {
        self.run_designer(taskset)
            .with_context(|| std::format!("Designer error for \"{}\"", self.designer_name()))
    }
}