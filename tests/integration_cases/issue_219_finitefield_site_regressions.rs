use std::thread;

use browser_tester::Harness;

#[test]
fn issue_219_margin_markup_page_loads_and_runs_bulk_flow_on_small_test_thread()
-> browser_tester::Result<()> {
    // Fixed snapshot of the retail margin/markup calculator page that overflowed
    // the default Rust test thread stack in v1.4.14.
    let html = include_str!("../fixtures/issue-219-retail-margin-markup-live.html");

    let handle = thread::Builder::new()
        .name("issue-219-small-stack".to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(|| -> Result<(), String> {
            let mut harness = Harness::from_html(html).map_err(|error| error.to_string())?;
            harness
                .click("#margin-markup-calculator-open-button")
                .map_err(|error| error.to_string())?;
            harness
                .click("#margin-markup-calculator-settings summary")
                .map_err(|error| error.to_string())?;
            harness
                .type_text("#field-cost", "1200")
                .map_err(|error| error.to_string())?;
            harness
                .type_text("#field-extra-cost", "230")
                .map_err(|error| error.to_string())?;
            harness
                .type_text("#field-target-margin", "40")
                .map_err(|error| error.to_string())?;
            harness
                .click("[data-mode-tab='bulk']")
                .map_err(|error| error.to_string())?;
            harness
                .click("#bulk-insert-sample")
                .map_err(|error| error.to_string())?;
            harness
                .assert_text("#bulk-summary", "全 3 行中 3 行を計算しました。")
                .map_err(|error| error.to_string())?;
            Ok(())
        })
        .expect("small-stack regression thread should spawn");

    handle
        .join()
        .expect("small-stack regression thread should not panic")
        .map_err(browser_tester::Error::ScriptRuntime)?;
    Ok(())
}
