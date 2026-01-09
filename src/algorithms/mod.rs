//! Root Module containing all the implemented analyses

pub mod full_preemption {
    pub mod uniprocessor {
        pub mod earliest_deadline_first {
            pub mod edf73;
        }

        pub mod fixed_priority {
            pub mod rate_monotonic73;
            pub mod rta86;
            pub mod deadline_monotonic90;
            pub mod hyperbolic01;
        }

        pub mod hierarchical {
            pub mod pr_model03;
        }
    }

    pub mod global_multiprocessor {
        pub mod earliest_deadline_first {
            pub mod gbf03;
            pub mod baker03;
            pub mod bcl05;
            pub mod baruah07;
            pub mod bcl09;
        }

        pub mod fixed_priority {
            pub mod deadline_monotonic_bcl05;
            pub mod rta_lc09;
            pub mod bcl09;
        }

        pub mod generic_work_conserving {
            pub mod bcl09;
        }

        pub mod hierarchical {
            pub mod mpr_model09;
        }
    }
}