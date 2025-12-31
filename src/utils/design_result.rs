
/// Error for interface generation results.
///
/// The error is [`DesignError::Precondition`] when any designer's
/// precondition is not met, or [`DesignError::Other`] for other errors.
#[derive(Debug)]
pub enum DesignError {
    Failure(Option<anyhow::Error>),
    Precondition(Option<anyhow::Error>),
    Other(anyhow::Error),
}

impl std::fmt::Display for DesignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DesignError::*;

        match &self {
            Failure(Some(error)) =>
                write!(f, "Design failed, reason: {}", error),
            Failure(None) =>
                write!(f, "Design failed."),
            Precondition(None) =>
                write!(f, "Design precondition error."),
            Precondition(Some(error)) =>
                write!(f, "Design precondition error, reason: {}", error),
            Other(error) =>
                write!(f, "Design error: {}", error),
        }
    }
}

impl std::error::Error for DesignError {}

/// Type for a interface generation results.
///
/// Is `Ok( T )` when the interface was generated successfully. \
/// Is `Err(`[`DesignError`]`)` when no suitable interface was found.
pub struct DesignResult<T> {
    pub test_name: String,
    pub result: Result<T, DesignError>,
}

impl<T> DesignResult<T> {
    /// Check if the designer successfully terminated.
    ///
    /// *WARNING*: by negating this result it does not necessarily mean that the
    /// designer couldn't find a suitable interface.
    /// Use [is_design_error](`Self::is_design_error`)/[] or do a patten
    /// matching on the result to check if all the preconditions were satisfied
    /// but the designer failed.
    pub fn is_successful(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if the designer failed (but all preconditions were satisfied).
    pub fn is_not_successful(&self) -> bool {
        match self.result {
            Err(DesignError::Failure(_)) => true,
            _ => false,
        }
    }

    /// Check if the designer couldn't be run because of an unsatisfied precondition.
    pub fn is_precondition_error(&self) -> bool {
        match self.result {
            Err(DesignError::Precondition(_)) => true,
            _ => false,
        }
    }

    /// Check if there was another error in the execution of the designer.
    pub fn is_other_error(&self) -> bool {
        match self.result {
            Err(DesignError::Other(_)) => true,
            _ => false,
        }
    }

    pub fn map<U, F>(self, op: F) -> DesignResult<U>
        where F: FnOnce(T) -> U
    {
        DesignResult { test_name: self.test_name, result: self.result.map(op) }
    }
}

impl<T> std::fmt::Display for DesignResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DesignError::*;

        match &self.result {
            Ok(_) =>
                write!(f, "Designer \"{}\" interface successfully generated.", self.test_name),
            Err(Failure(None)) =>
                write!(f, "Designer \"{}\" couldn't find a suitable interface.", self.test_name),
            Err(Failure(Some(error))) =>
                write!(f, "Designer \"{}\" couldn't find a suitable interface, reason: {}", self.test_name, error),
            Err(Precondition(None)) =>
                write!(f, "Designer \"{}\" precondition error.", self.test_name),
            Err(Precondition(Some(error))) =>
                write!(f, "Designer \"{}\" precondition error, reason: {}", self.test_name, error),
            Err(Other(error)) =>
                write!(f, "Designer \"{}\" error: {}", self.test_name, error),
        }
    }
}
/// Helper factory for common schedulability test errors.
///
/// Takes the test's name as first parameter.
pub struct DesignResultFactory<'a>(pub &'a str);

impl<'a> DesignResultFactory<'a> {
    pub fn from_err<T>(self, error: DesignError) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(error),
        }
    }

    pub fn from_option<T>(self, interface: Option<T>) -> DesignResult<T> {
        match interface {
            Some(interface) => self.success(interface),
            None => self.failure(),
        }
    }

    pub fn success<T>(self, interface: T) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Ok(interface),
        }
    }

    pub fn failure<T>(self) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Failure(None)),
        }
    }

    pub fn failure_reason<T>(self, reason: anyhow::Error) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Failure(Some(reason))),
        }
    }

    pub fn precondition<T>(self) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Precondition(None)),
        }
    }

    pub fn precondition_reason<T>(self, reason: anyhow::Error) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Precondition(Some(reason))),
        }
    }

    pub fn other<T>(self, reason: anyhow::Error) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Other(reason)),
        }
    }

    pub fn implicit_deadlines<T>(self) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Precondition(Some(
                anyhow::format_err!("taskset must have implicit deadlines.")
            ))),
        }
    }

    pub fn constrained_deadlines<T>(self) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Precondition(Some(
                anyhow::format_err!("taskset must have constrained deadlines.")
            ))),
        }
    }

    pub fn rate_monotonic<T>(self) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Precondition(Some(
                anyhow::format_err!("taskset must be sorted by period.")
            ))),
        }
    }

    pub fn deadline_monotonic<T>(self) -> DesignResult<T> {
        DesignResult {
            test_name: self.0.to_owned(),
            result: Err(DesignError::Precondition(Some(
                anyhow::format_err!("taskset must be sorted by deadline.")
            ))),
        }
    }
}