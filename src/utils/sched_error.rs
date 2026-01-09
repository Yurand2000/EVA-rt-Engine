#[derive(Debug)]
pub enum SchedError {
    NonSchedulable(Option<anyhow::Error>),
    Precondition(Option<anyhow::Error>),
    Other(anyhow::Error),
}

impl std::fmt::Display for SchedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::NonSchedulable(None) =>
                write!(f, "Non schedulable."),
            Self::NonSchedulable(Some(error)) =>
                write!(f, "Non schedulable, reason: {}", error),
            Self::Precondition(None) =>
                write!(f, "Precondition error."),
            Self::Precondition(Some(error)) =>
                write!(f, "Precondition error: {}", error),
            Self::Other(error) =>
                write!(f, "Other error: {}", error),
        }
    }
}

impl std::error::Error for SchedError { }

impl SchedError {
    pub fn result_from_schedulable(is_schedulable: bool) -> Result<(), Self> {
        if is_schedulable {
            Ok(())
        } else {
            Err(Self::NonSchedulable(None))
        }
    }

    pub fn implicit_deadlines() -> Self {
        Self::Precondition(Some(
            anyhow::format_err!("taskset must have implicit deadlines.")
        ))
    }

    pub fn constrained_deadlines() -> Self {
        Self::Precondition(Some(
            anyhow::format_err!("taskset must have constrained deadlines.")
        ))
    }

    pub fn rate_monotonic() -> Self {
        Self::Precondition(Some(
            anyhow::format_err!("taskset must be sorted by period.")
        ))
    }

    pub fn deadline_monotonic() -> Self {
        Self::Precondition(Some(
            anyhow::format_err!("taskset must be sorted by deadline.")
        ))
    }
}