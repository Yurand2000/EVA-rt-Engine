//! ## Multiprocessor Periodic Resource Model - Shin, Easwaran, Lee 2009
//!
//! ---
//! #### References:
//! 1. I. Shin, A. Easwaran, and I. Lee, “Hierarchical Scheduling Framework for
//!    Virtual Clustering of Multiprocessors,” in 2008 Euromicro Conference on
//!    Real-Time Systems, July 2008, pp. 181–190. doi: 10.1109/ECRTS.2008.28.
//! 2. A. Easwaran, I. Shin, and I. Lee, “Optimal virtual cluster-based
//!    multiprocessor scheduling,” Real-Time Syst, vol. 43, no. 1, pp. 25–59,
//!    Sept. 2009, doi: 10.1007/s11241-009-9073-x.

use crate::prelude::*;

// Section 3.2 [1]
#[derive(Debug, Clone)]
pub struct MPRModel {
    pub resource: Time,
    pub period: Time,
    pub concurrency: u64,
}

// Design Specification for MPRModel search
#[derive(Debug, Clone)]
pub struct MPRModelSpecification {
    pub period: Time,
    pub concurrency: u64,
}

impl MPRModel {
    pub fn is_feasible(&self) -> bool {
        self.resource <= self.concurrency as f64 * self.period
    }

    /// resource / period
    pub fn utilization(&self) -> f64 {
        self.resource / self.period
    }

    /// Get the total supply the model provides in the given time interval.
    pub fn get_supply(&self, interval: Time) -> Time {
        // Definition 1 [2]
        // Supply Bound Function for a MPRModel
        let m = self.concurrency as f64;
        let a = Time::floor(self.resource / m);
        let b = self.resource - m * a;
        let t1 = interval - (self.period - Time::ceil(self.resource / m));
        let x = t1 - self.period * f64::floor(t1 / self.period);
        let y = self.period - a;

        if t1 < Time::zero() {
            Time::zero()
        } else {
            f64::floor(t1 / self.period) * self.resource
                +
            Time::max(Time::zero(), m*x - m*self.period + self.resource)
                -
            if x >= Time::one() && x < y {
                Time::zero()
            } else {
                Time::nanos(m) - b
            }
        }
    }

    /// Get the total supply the model provides in the given time interval (linear version).
    ///
    /// Note: for_all time. linear_supply(model, time) <= supply(model, time)
    pub fn get_supply_linear(&self, interval: Time) -> Time {
        let (resource, period, concurrency) = (self.resource, self.period, self.concurrency);

        // Equation 2 [2]
        resource / period * (interval - 2.0 * (period - resource / concurrency as f64) + Time::nanos(2.0))
    }

    /// Get the resource of the model which provides the given (linear) supply in the given time interval.
    pub fn resource_from_linear_supply(lsbf: Time, interval: Time, period: Time, concurrency: u64) -> Time {
        // Note that this only works for positive values of the linear supply bound.
        // There is only one positive solution for a positive bound, but two
        // solutions or zero for a negative one.
        debug_assert!(lsbf >= Time::zero());

        let cpus = concurrency as f64;
        let negb = 2.0 * period - interval + Time::nanos(2.0);
        let bsqr = negb * negb;

        // Extracted Theta from Equation 2 [2]
        cpus * (negb + Time2::sqrt(bsqr + 8.0 * period * lsbf / cpus) ) / 4.0
    }
}

impl MPRModelSpecification {
    pub fn into_model(self, resource: Time) -> MPRModel {
        MPRModel {
            resource,
            period: self.period,
            concurrency: self.concurrency
        }
    }

    /// Get the resource of the model which provides the given (linear) supply in the given time interval.
    pub fn resource_from_linear_supply(&self, lsbf: Time, interval: Time) -> Time {
        MPRModel::resource_from_linear_supply(lsbf, interval, self.period, self.concurrency)
    }

    /// Get the model which provides the given (linear) supply in the given time interval.
    pub fn model_from_linear_supply(&self, lsbf: Time, interval: Time) -> MPRModel {
        MPRModel {
            resource: self.resource_from_linear_supply(lsbf, interval),
            period: self.period,
            concurrency: self.concurrency,
        }
    }
}

// -----------------------------------------------------------------------------
// Convert each MPRModel to a set of periodic tasks (with implicit deadline)
// that represent the high-level requirements for the scheduled taskset. This
// set of server tasks can be scheduled with uniprocessor algorithms, as they
// are meant to be pinned to invididual CPUs.
impl MPRModel {
    // Section 5.2, Definition 1 [1]
    pub fn to_periodic_tasks(&self) -> Vec<RTTask> {
        #[inline(always)]
        fn psi(model: &MPRModel) -> Time {
            model.resource - model.concurrency as f64 *
                (model.resource / model.concurrency as f64).floor()
        }

        let k = psi(&self).as_nanos();

        (0..self.concurrency)
            .map(|i| {
                let wcet =
                    if i <= k as u64 {
                        (self.resource / self.concurrency as f64).floor() + Time::one()
                    } else if i == k as u64 + 1 {
                        (self.resource / self.concurrency as f64).floor()
                            + psi(&self) - k * (psi(&self) / k).floor()
                    } else {
                        (self.resource / self.concurrency as f64).floor()
                    };

                RTTask {
                    wcet: wcet,
                    deadline: self.period,
                    period: self.period,
                }
            })
            .collect()
    }

    pub fn to_periodic_tasks_simple(&self) -> (RTTask, u64) {
        let task =
            RTTask {
                wcet: (self.resource / self.concurrency as f64).floor() + Time::one(),
                deadline: self.period,
                period: self.period,
            };

        (task, self.concurrency)
    }
}