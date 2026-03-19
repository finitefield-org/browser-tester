use bt_runtime::Scheduler;

#[test]
fn scheduler_keeps_timer_order_and_flushes_microtasks() {
    let mut scheduler = Scheduler::default();

    let later = scheduler.queue_timer(10);
    let earlier = scheduler.queue_timer(5);

    scheduler.queue_microtask();
    scheduler.queue_microtask();
    assert_eq!(scheduler.microtask_count(), 2);

    scheduler.flush();
    assert_eq!(scheduler.microtask_count(), 0);

    scheduler.advance_time_to(10);
    let due = scheduler.run_due_timers();
    assert_eq!(
        due.iter().map(|timer| timer.id).collect::<Vec<_>>(),
        vec![earlier, later]
    );
    assert!(scheduler.pending_timers().is_empty());
}
