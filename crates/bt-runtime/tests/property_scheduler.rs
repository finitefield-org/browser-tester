use bt_runtime::{ScheduledTimer, Scheduler};
use proptest::prelude::*;

proptest! {
    #[test]
    fn queue_timer_keeps_timers_sorted_by_due_time(times in proptest::collection::vec(-250i64..=250i64, 0..32)) {
        let mut scheduler = Scheduler::default();

        for at_ms in &times {
            scheduler.queue_timer(*at_ms);
        }

        let mut expected: Vec<ScheduledTimer> = times
            .into_iter()
            .enumerate()
            .map(|(index, at_ms)| ScheduledTimer {
                id: (index as u64) + 1,
                at_ms,
            })
            .collect();
        expected.sort_by_key(|timer| (timer.at_ms, timer.id));

        prop_assert_eq!(scheduler.pending_timers(), expected.as_slice());
    }

    #[test]
    fn flush_clears_timers_and_microtasks(times in proptest::collection::vec(-250i64..=250i64, 0..32), microtasks in 0usize..32) {
        let mut scheduler = Scheduler::default();

        for at_ms in &times {
            scheduler.queue_timer(*at_ms);
        }

        for _ in 0..microtasks {
            scheduler.queue_microtask();
        }

        let expected_now_ms = times.into_iter().fold(0i64, |acc, at_ms| acc.max(at_ms));

        scheduler.flush();

        prop_assert!(scheduler.pending_timers().is_empty());
        prop_assert_eq!(scheduler.microtask_count(), 0);
        prop_assert_eq!(scheduler.now_ms(), expected_now_ms.max(0));
    }
}
