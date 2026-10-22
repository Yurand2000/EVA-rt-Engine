//! ## Multi Supply Function Abstraction - Bini, Buttazzo, Bertogna 2009
//!
//! #### Implements:
//!
//! ---
//! #### References:
//! 1. E. Bini, G. Buttazzo and M. Bertogna, "The Multi Supply Function
//!    Abstraction for Multiprocessors," 2009 15th IEEE International
//!    Conference on Embedded and Real-Time Computing Systems and Applications,
//!    Beijing, China, 2009, pp. 294-302, doi: 10.1109/RTCSA.2009.39.
//! 2. A. Easwaran, M. Anand and I. Lee, "Compositional Analysis Framework
//!    Using EDP Resource Models," 28th IEEE International Real-Time Systems
//!    Symposium (RTSS 2007), Tucson, AZ, USA, 2007, pp. 129-138,
//!    doi: 10.1109/RTSS.2007.36.

use crate::prelude::*;

// Local Scheduling Algorithms
pub mod earliest_deadline_first {
    pub mod msf09;
}

pub mod fixed_priority {
    pub mod msf09;
}

pub mod extra;

/// ## Multi Supply Function Abstraction - Bini, Buttazzo, Bertogna 2009
///
/// Refer to the [module](`self`) level documentation.
#[derive(Debug, Clone)]
pub struct MSFModel<S>
    where S: MSFServer
{
    // Section 2 [1]
    pub processors: Vec<S>,
}

/// Server's characteristic function
pub trait MSFServer {
    fn supply(&self, time: Time) -> Time;
}

impl<S> MSFModel<S>
    where S: MSFServer
{
    // Equation 34 [1]
    pub fn lengths(&self, time: Time) -> Vec<Time> {
        let len = self.processors.len();
        let mut supplies: Vec<_> = self.processors.iter()
            .map(|server| server.supply(time))
            .collect();

        supplies.sort_unstable_by_key(|&supply| std::cmp::Reverse(supply) );

        let mut lengths = vec![Time::zero(); len];

        lengths[0] = time - supplies[0];
        lengths[len - 1] = supplies[len - 1];
        for i in 1 .. (len - 1) {
            lengths[i] = supplies[i] - supplies[i + 1];
        }

        lengths
    }

    // Equation 30 [1]
    pub fn interference(&self, workload: Time, deadline: Time) -> Time {
        let lengths = self.lengths(deadline);

        let p_lp = |l| {
            lengths[0..l].iter().enumerate()
                .map(|(p, &length_p)| p as f64 * length_p)
                .sum()
        };

        lengths[0]
            +
        lengths[1..].iter().enumerate()
            .map(|(l, &length_l)|
                Time::min(
                    length_l,
                    Time::max(
                        Time::zero(),
                        (workload - p_lp(l)) / (l as f64)
                    )
                )
            )
            .sum()
    }
}

/// Linear Upperbound of the server's characteristic function and its inverse.
///
/// Must be true that:
/// *for all t. inverse_supply_linear(supply_linear(t)) = t and
/// for all s. supply_linear(inverse_supply_linear(s)) = s*
pub trait MSFServerLinear: MSFServer {
    fn supply_linear(&self, time: Time) -> Time;

    fn inverse_supply_linear(&self, supply: Time) -> Time;
}

#[derive(Debug, Clone)]
/// Explicit Deadline Periodic server [2]
pub struct ExplicitDeadlinePeriodicServer {
    pub runtime: Time,
    pub deadline: Time,
    pub period: Time,
}

impl MSFServer for ExplicitDeadlinePeriodicServer {
    fn supply(&self, t: Time) -> Time {
        if t < self.deadline - self.runtime {
            return Time::zero();
        }

        let y = f64::floor((t - self.deadline + self.runtime) / self.period);
        let x = self.period + self.deadline - 2.0 * self.runtime;
        y * self.runtime + Time::max(Time::zero(), t - x - y * self.period)
    }
}

impl MSFServerLinear for ExplicitDeadlinePeriodicServer {
    fn supply_linear(&self, t: Time) -> Time {
        self.runtime / self.period * (t - self.deadline + self.runtime)
    }

    fn inverse_supply_linear(&self, s: Time) -> Time {
        (s * self.period / self.runtime) - self.runtime + self.deadline
    }
}
