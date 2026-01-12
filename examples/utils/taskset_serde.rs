use eva_rt_engine::prelude::*;

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Eq)]
#[derive(clap::ValueEnum)]
pub enum TasksetPlainUnit {
    Millis,
    Micros,
    Nanos
}

pub fn parse_taskset<P: AsRef<std::path::Path>>(
    taskset_file: P,
    unit: TasksetPlainUnit,
) -> anyhow::Result<Vec<RTTask>> {
    let taskset_data = std::fs::read_to_string(taskset_file)?;
    Ok(plain_deserialize_taskset(&taskset_data, unit)?)
}

fn plain_deserialize_taskset(data: &str, unit: TasksetPlainUnit) -> anyhow::Result<Vec<RTTask>> {
    data.trim_ascii()
        .lines()
        .map(|line| plain_deserialize_task(line, unit))
        .collect()
}

fn plain_deserialize_task(data: &str, unit: TasksetPlainUnit) -> anyhow::Result<RTTask> {
    let fields: Vec<&str> = data
        .trim_ascii()
        .split_ascii_whitespace()
        .collect();

    let multiplier =
        match unit {
            TasksetPlainUnit::Millis => Time::MILLI_TO_NANO,
            TasksetPlainUnit::Micros => Time::MICRO_TO_NANO,
            TasksetPlainUnit::Nanos => 1.0,
        };

    if fields.len() != 3 {
        return Err(
            anyhow::format_err!("RTTask parsing requires three numeric fields (wcet, deadline and period)")
        );
    }

    Ok(RTTask {
        wcet: Time::nanos(fields[0].parse::<f64>()
            .map_err(|err|
                anyhow::format_err!("Failed to parse field 'wcet': {err}")
            )? * multiplier
        ),
        deadline: Time::nanos(fields[1].parse::<f64>()
            .map_err(|err|
                anyhow::format_err!("Failed to parse field 'deadline': {err}")
            )? * multiplier
        ),
        period: Time::nanos(fields[2].parse::<f64>()
            .map_err(|err|
                anyhow::format_err!("Failed to parse field 'period': {err}")
            )? * multiplier
        ),
    })
}