use bt_runtime::Scheduler;

#[test]
fn scheduler_keeps_timer_order_and_flushes_microtasks() {
    let mut scheduler = Scheduler::default();

    let later = scheduler.queue_timer(10);
    let earlier = scheduler.queue_timer(5);
    assert_eq!(later, 1);
    assert_eq!(earlier, 2);

    scheduler.queue_microtask();
    scheduler.queue_microtask();
    assert_eq!(scheduler.microtask_count(), 2);

    scheduler.advance_time_to(7);
    assert_eq!(scheduler.now_ms(), 7);
    assert_eq!(
        scheduler.pending_timers(),
        &[bt_runtime::ScheduledTimer {
            id: later,
            at_ms: 10
        }]
    );

    scheduler.advance_time(3);
    assert_eq!(scheduler.now_ms(), 10);
    assert!(scheduler.pending_timers().is_empty());

    scheduler.queue_timer(12);
    scheduler.queue_microtask();
    scheduler.flush();
    assert_eq!(scheduler.microtask_count(), 0);
    assert_eq!(scheduler.now_ms(), 12);
    assert!(scheduler.pending_timers().is_empty());
}
