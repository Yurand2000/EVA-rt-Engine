use crate::prelude::*;
use crate::algorithms::*;
use itertools::Itertools;

pub struct Analyzer;

impl crate::analysis::Analyzer<RTTask, ()> for Analyzer {
    fn is_schedulable(&self, taskset: &[RTTask], _: &(), short_circuit: bool) -> SchedTestResults {
        let tests: &[&'static str] = &[
            "rm-hyperbolic",
            "dm-classic",
            "rta",
        ];

        let (schedulable, results) =
            if short_circuit {
                let results: Vec<_> =
                    tests.iter()
                    .map(|test_name| Self::run_test(test_name, taskset))
                    .take_while_inclusive(|test| test.result.is_ok())
                    .collect();

                (results.last().unwrap().result.is_ok(), results)
            } else {
                let results: Vec<_> =
                    tests.iter()
                    .map(|test_name| Self::run_test(test_name, taskset))
                    .collect();

                (results.iter().any(|test| test.result.is_ok()), results)
            };

        SchedTestResults { schedulable, results }
    }

    fn run_schedulability_test(&self, taskset: &[RTTask], _: &(), test_name: &str) -> SchedResult<()> {
        let Some((_, test_fn)) = Self::NAME_TO_TEST_MAP.iter()
                                    .find(|(name, _)| *name == test_name)
            else {
                return Err(SchedError::other(
                    test_name,
                    anyhow::format_err!("Unknown Test: {}", test_name)
                ))
            };

        test_fn(taskset)
    }

    fn available_tests(&self) -> impl Iterator<Item = &'static str> {
        Self::NAME_TO_TEST_MAP.iter()
            .map(|&(name, _)| name)
    }
}

impl Analyzer {
    const NAME_TO_TEST_MAP: &[(&'static str, fn(&[RTTask]) -> SchedResult<()>)] = &[
        ("rm-classic", rate_monotonic73::is_schedulable),
        ("rm-simplified", rate_monotonic73::is_schedulable_simple),
        ("rm-hyperbolic", hyperbolic01::is_schedulable),
        ("dm-classic", deadline_monotonic90::is_schedulable),
        ("rta", |t| rta86::is_schedulable(t).map(|_| ())),
    ];

    fn get_test(test_name: &str) -> &fn(&[RTTask]) -> SchedResult<()> {
        Self::NAME_TO_TEST_MAP.iter()
            .find_map(|(name, test_fn)|
                if *name == test_name { Some(test_fn) }
                else { None }
            ).unwrap()
    }

    fn run_test(test_name: &'static str, taskset: &[RTTask]) -> SchedTestResult {
        SchedTestResult {
            test_name: test_name,
            result: Self::get_test(test_name)(taskset)
        }
    }
}