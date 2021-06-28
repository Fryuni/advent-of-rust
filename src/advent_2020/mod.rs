mod day15;
mod day16;
mod day17;
mod day18;

use crate::advent_adapters::StatefulAdvent;
use crate::advents::{AdventYear, SkippedAdvent};

pub fn get_advent_year() -> AdventYear {
    AdventYear::new(
        2020,
        vec![
            Box::new(SkippedAdvent::new(1)),
            Box::new(SkippedAdvent::new(2)),
            Box::new(SkippedAdvent::new(3)),
            Box::new(SkippedAdvent::new(4)),
            Box::new(SkippedAdvent::new(5)),
            Box::new(SkippedAdvent::new(6)),
            Box::new(SkippedAdvent::new(7)),
            Box::new(SkippedAdvent::new(8)),
            Box::new(SkippedAdvent::new(9)),
            Box::new(SkippedAdvent::new(10)),
            Box::new(SkippedAdvent::new(11)),
            Box::new(SkippedAdvent::new(12)),
            Box::new(SkippedAdvent::new(13)),
            Box::new(SkippedAdvent::new(14)),
            Box::new(day15::AdventDay15),
            Box::new(day16::AdventDay16),
            Box::new(StatefulAdvent::<day17::AdventDay17>::new(17)),
            Box::new(StatefulAdvent::<day18::AdventDay18>::new(18)),
        ],
    )
}
