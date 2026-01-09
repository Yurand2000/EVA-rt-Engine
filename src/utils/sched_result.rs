/// Error for schedulability test results.
///
/// The error is [`SchedError::NonSchedulable`] when the taskset is not
/// schedulable, [`SchedError::Precondition`] when a schedulability test's
/// precondition is not met, or [`SchedError::Other`] for other errors.
#[derive(Debug)]
pub enum SchedError {
    NonSchedulable(Option<anyhow::Error>),
    Precondition(Option<anyhow::Error>),
    Other(anyhow::Error),
}

impl std::fmt::Display for SchedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SchedError::*;

        match &self {
            NonSchedulable(None) =>
                write!(f, "Analysis error: non schedulable."),
            NonSchedulable(Some(error)) =>
                write!(f, "Analysis error: non schedulable, reason: {}", error),
            Precondition(None) =>
                write!(f, "Analysis precondition error."),
            Precondition(Some(error)) =>
                write!(f, "Analysis precondition error, reason: {}", error),
            Other(error) =>
                write!(f, "Analysis error: {}", error),
        }
    }
}

impl std::error::Error for SchedError {}

/// Type for a schedulability test results.
///
/// Is `Ok( T )` when the taskset is schedulable. \
/// Is `Err(`[`SchedError`]`)` when the taskset is not schedulable.
pub struct SchedResult<T> {
    pub test_name: String,
    pub result: Result<T, SchedError>,
}

impl<T> SchedResult<T> {
    /// Check if the schedulability test passed.
    ///
    /// *WARNING*: by negating this result it does not necessarily mean that the
    /// taskset did satisfy all the precondition of the run schedulability test.
    /// Use [is_not_schedulable](`Self::is_not_schedulable`)/[] or do a patten
    /// matching on the result to check if all the preconditions were satisfied
    /// but the test failed.
    pub fn is_schedulable(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if the schedulability test failed (but all preconditions were satisfied).
    pub fn is_not_schedulable(&self) -> bool {
        match self.result {
            Err(SchedError::NonSchedulable(_)) => true,
            _ => false,
        }
    }

    /// Check if the schedulability test couldn't be run because of an unsatisfied precondition.
    pub fn is_precondition_error(&self) -> bool {
        match self.result {
            Err(SchedError::Precondition(_)) => true,
            _ => false,
        }
    }

    /// Check if there was another error in the execution of the schedulability test.
    pub fn is_other_error(&self) -> bool {
        match self.result {
            Err(SchedError::Other(_)) => true,
            _ => false,
        }
    }

    pub fn map<U, F>(self, op: F) -> SchedResult<U>
        where F: FnOnce(T) -> U
    {
        SchedResult { test_name: self.test_name, result: self.result.map(op) }
    }
}

impl<T> std::fmt::Display for SchedResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SchedError::*;

        match &self.result {
            Ok(_) =>
                write!(f, "Analysis \"{}\": schedulable.", self.test_name),
            Err(NonSchedulable(None)) =>
                write!(f, "Analysis \"{}\" error: non schedulable.", self.test_name),
            Err(NonSchedulable(Some(error))) =>
                write!(f, "Analysis \"{}\" error: non schedulable, reason: {}", self.test_name, error),
            Err(Precondition(None)) =>
                write!(f, "Analysis \"{}\" precondition error.", self.test_name),
            Err(Precondition(Some(error))) =>
                write!(f, "Analysis \"{}\" precondition error, reason: {}", self.test_name, error),
            Err(Other(error)) =>
                write!(f, "Analysis \"{}\" error: {}", self.test_name, error),
        }
    }
}

impl<T> Into<anyhow::Result<bool>> for SchedResult<T> {
    fn into(self) -> anyhow::Result<bool> {
        match self.result {
            Ok(_) => Ok(true),
            Err(SchedError::NonSchedulable(_)) => Ok(false),
            Err(err) => Err(err.into())
        }
    }
}

/// Helper factory for common schedulability test errors.
///
/// Takes the test's name as first parameter.
pub struct SchedResultFactory<'a>(pub &'a str);

impl<'a> SchedResultFactory<'a> {
    pub fn from_err<T>(self, error: SchedError) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(error),
        }
    }

    pub fn is_schedulable(self, is_schedulable: bool) -> SchedResult<()> {
        if is_schedulable {
            self.schedulable(())
        } else {
            self.non_schedulable()
        }
    }

    pub fn schedulable<T>(self, result: T) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Ok(result),
        }
    }

    pub fn non_schedulable<T>(self) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::NonSchedulable(None)),
        }
    }

    pub fn non_schedulable_reason<T>(self, reason: anyhow::Error) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::NonSchedulable(Some(reason))),
        }
    }

    pub fn precondition<T>(self) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::Precondition(None)),
        }
    }

    pub fn precondition_reason<T>(self, reason: anyhow::Error) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::Precondition(Some(reason))),
        }
    }

    pub fn other<T>(self, reason: anyhow::Error) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::Other(reason)),
        }
    }

    pub fn implicit_deadlines<T>(self) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::Precondition(Some(
                anyhow::format_err!("taskset must have implicit deadlines.")
            ))),
        }
    }

    pub fn constrained_deadlines<T>(self) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::Precondition(Some(
                anyhow::format_err!("taskset must have constrained deadlines.")
            ))),
        }
    }

    pub fn rate_monotonic<T>(self) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::Precondition(Some(
                anyhow::format_err!("taskset must be sorted by period.")
            ))),
        }
    }

    pub fn deadline_monotonic<T>(self) -> SchedResult<T> {
        SchedResult {
            test_name: self.0.to_owned(),
            result: Err(SchedError::Precondition(Some(
                anyhow::format_err!("taskset must be sorted by deadline.")
            ))),
        }
    }
}