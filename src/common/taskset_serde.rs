use crate::prelude::*;

pub mod prelude {
    pub use super::{
        TasksetFileType,
        TasksetParseError,
        parse_taskset,
    };
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
#[derive(clap::ValueEnum)]
pub enum TasksetFileType {
    Auto,
    JSON,
    Plain,
}

#[derive(Debug)]
pub enum TasksetParseError {
    IOError(std::io::Error),
    JSONError(serde_json::Error),
    PlainParseError(String),
}

pub fn parse_taskset(taskset: &str, typ: TasksetFileType) -> Result<Vec<RTTask>, TasksetParseError> {
    use TasksetFileType::*;

    let taskset = std::path::Path::new(taskset);
    let extension =
        if typ == TasksetFileType::Auto {
            if taskset.ends_with(".json") {
                TasksetFileType::JSON
            } else if taskset.ends_with(".txt") {
                TasksetFileType::Plain
            } else {
                TasksetFileType::Plain
            }
        } else { typ };

    let taskset_data = std::fs::read_to_string(taskset)?;

    let taskset_data = match extension {
        Auto => panic!("Unexpected Auto Extension"),
        JSON => serde_json::from_str(&taskset_data)?,
        Plain => plain_deserialize_taskset(&taskset_data)?,
    };

    Ok(taskset_data)
}

fn plain_deserialize_taskset(data: &str) -> Result<Vec<RTTask>, TasksetParseError> {
    data.trim_ascii()
        .lines()
        .map(|line| plain_deserialize_task(line))
        .collect()
}

fn plain_deserialize_task(data: &str) -> Result<RTTask, TasksetParseError> {
    let fields: Vec<&str> = data
        .trim_ascii()
        .split_ascii_whitespace()
        .collect();

    if fields.len() != 3 {
        return Err(TasksetParseError::PlainParseError(format!("RTTask parsing requires three numeric fields (wcet, deadline and period)")));
    }

    Ok(RTTask {
        wcet: Time::millis(fields[0].parse()
            .map_err(|err| TasksetParseError::PlainParseError(format!("Failed to parse field 'wcet': {err}")))?
        ),
        deadline: Time::millis(fields[1].parse()
            .map_err(|err| TasksetParseError::PlainParseError(format!("Failed to parse field 'deadline': {err}")))?
        ),
        period: Time::millis(fields[2].parse()
            .map_err(|err| TasksetParseError::PlainParseError(format!("Failed to parse field 'period': {err}")))?
        ),
    })
}

// =============================================================================

impl std::fmt::Display for TasksetParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Taskset Parse Error, ")?;
        match self {
            TasksetParseError::IOError(error) => write!(f, "IO: {error}")?,
            TasksetParseError::JSONError(error) => write!(f, "JSON: {error}")?,
            TasksetParseError::PlainParseError(error) => write!(f, "Plain: {error}")?,
        };

        Ok(())
    }
}

impl std::error::Error for TasksetParseError {}

impl From<std::io::Error> for TasksetParseError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<serde_json::Error> for TasksetParseError {
    fn from(value: serde_json::Error) -> Self {
        Self::JSONError(value)
    }
}