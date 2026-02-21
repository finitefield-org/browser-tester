use super::*;

#[test]
fn timer_interval_supports_multiple_additional_parameters() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let id = 0;
          document.getElementById('btn').addEventListener('click', () => {
            let tick = 0;
            id = setInterval((value, suffix) => {
              tick = tick + 1;
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + value + suffix;
              if (tick > 2) {
                clearInterval(id);
              }
            }, 0, 'I', '!');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "I!I!I!")?;
    Ok(())
}

#[test]
fn line_and_block_comments_are_ignored_in_script_parser() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          // top level comment
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'A'; // inline comment
            /* block comment */
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn run_due_timers_runs_only_currently_due_tasks() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert_eq!(h.now_ms(), 0);

    let ran = h.run_due_timers()?;
    assert_eq!(ran, 1);
    assert_eq!(h.now_ms(), 0);
    h.assert_text("#result", "A")?;

    let ran = h.run_due_timers()?;
    assert_eq!(ran, 0);
    h.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn run_due_timers_returns_zero_for_empty_queue() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    assert_eq!(h.run_due_timers()?, 0);
    Ok(())
}

#[test]
fn clear_timer_cancels_specific_pending_timer() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert!(h.clear_timer(1));
    assert!(!h.clear_timer(1));
    assert!(!h.clear_timer(999));

    h.advance_time(0)?;
    h.assert_text("#result", "B")?;
    h.advance_time(10)?;
    h.assert_text("#result", "B")?;
    Ok(())
}

#[test]
fn clear_all_timers_empties_pending_queue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = 'A';
            }, 0);
            setInterval(() => {
              result.textContent = 'B';
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert_eq!(h.pending_timers().len(), 2);
    assert_eq!(h.clear_all_timers(), 2);
    assert!(h.pending_timers().is_empty());
    h.flush()?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn run_next_due_timer_runs_only_one_due_task_without_advancing_clock() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert_eq!(h.now_ms(), 0);

    assert!(h.run_next_due_timer()?);
    assert_eq!(h.now_ms(), 0);
    h.assert_text("#result", "A")?;

    assert!(!h.run_next_due_timer()?);
    assert_eq!(h.now_ms(), 0);
    h.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn run_next_due_timer_returns_false_for_empty_queue() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    assert!(!h.run_next_due_timer()?);
    Ok(())
}

#[test]
fn pending_timers_returns_due_ordered_snapshot() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 10);
            setInterval(() => {}, 5);
            setTimeout(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    let timers = h.pending_timers();
    assert_eq!(
        timers,
        vec![
            PendingTimer {
                id: 3,
                due_at: 0,
                order: 2,
                interval_ms: None,
            },
            PendingTimer {
                id: 2,
                due_at: 5,
                order: 1,
                interval_ms: Some(5),
            },
            PendingTimer {
                id: 1,
                due_at: 10,
                order: 0,
                interval_ms: None,
            },
        ]
    );
    Ok(())
}

#[test]
fn pending_timers_reflects_advance_time_execution() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 5);
            setTimeout(() => {}, 7);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.advance_time(5)?;

    let timers = h.pending_timers();
    assert_eq!(
        timers,
        vec![
            PendingTimer {
                id: 2,
                due_at: 7,
                order: 1,
                interval_ms: None,
            },
            PendingTimer {
                id: 1,
                due_at: 10,
                order: 2,
                interval_ms: Some(5),
            },
        ]
    );
    Ok(())
}

#[test]
fn run_next_timer_executes_single_task_in_due_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'C';
            }, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert_eq!(h.now_ms(), 0);

    assert!(h.run_next_timer()?);
    assert_eq!(h.now_ms(), 0);
    h.assert_text("#result", "B")?;

    assert!(h.run_next_timer()?);
    assert_eq!(h.now_ms(), 10);
    h.assert_text("#result", "BA")?;

    assert!(h.run_next_timer()?);
    assert_eq!(h.now_ms(), 10);
    h.assert_text("#result", "BAC")?;

    assert!(!h.run_next_timer()?);
    assert_eq!(h.now_ms(), 10);
    Ok(())
}

#[test]
fn advance_time_to_runs_due_timers_until_target() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.advance_time_to(7)?;
    assert_eq!(h.now_ms(), 7);
    h.assert_text("#result", "A")?;

    h.advance_time_to(10)?;
    assert_eq!(h.now_ms(), 10);
    h.assert_text("#result", "AB")?;

    h.advance_time_to(10)?;
    assert_eq!(h.now_ms(), 10);
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn advance_time_to_rejects_past_target() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    h.advance_time(3)?;
    let err = h
        .advance_time_to(2)
        .expect_err("advance_time_to with past target should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("advance_time_to requires target >= now_ms"));
            assert!(msg.contains("target=2"));
            assert!(msg.contains("now_ms=3"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn set_timeout_respects_delay_order_and_nested_queueing() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + '1';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + '0';
              setTimeout(() => {
                result.textContent = result.textContent + 'N';
              });
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.flush()?;
    h.assert_text("#result", "0N1")?;
    Ok(())
}

#[test]
fn queue_microtask_runs_after_synchronous_task_body() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            queueMicrotask(() => {
              result.textContent = result.textContent + 'B';
            });
            result.textContent = result.textContent + 'C';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ACB")?;
    Ok(())
}

#[test]
fn promise_then_microtask_runs_before_next_timer() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            Promise.resolve().then(() => {
              result.textContent = result.textContent + 'P';
            });
            setTimeout(() => {
              result.textContent = result.textContent + 'T';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AP")?;
    h.flush()?;
    h.assert_text("#result", "APT")?;
    Ok(())
}

#[test]
fn fake_time_advance_controls_timer_execution() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + '0';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + '1';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + '2';
            }, 20);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    assert_eq!(h.now_ms(), 0);

    h.advance_time(0)?;
    h.assert_text("#result", "0")?;
    assert_eq!(h.now_ms(), 0);

    h.advance_time(9)?;
    h.assert_text("#result", "0")?;
    assert_eq!(h.now_ms(), 9);

    h.advance_time(1)?;
    h.assert_text("#result", "01")?;
    assert_eq!(h.now_ms(), 10);

    h.advance_time(10)?;
    h.assert_text("#result", "012")?;
    assert_eq!(h.now_ms(), 20);
    Ok(())
}

#[test]
fn fake_time_advance_runs_interval_ticks_by_due_time() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'I';
              if (result.textContent === 'III') clearInterval(id);
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;

    h.advance_time(4)?;
    h.assert_text("#result", "")?;

    h.advance_time(1)?;
    h.assert_text("#result", "I")?;

    h.advance_time(10)?;
    h.assert_text("#result", "III")?;

    h.advance_time(100)?;
    h.assert_text("#result", "III")?;
    Ok(())
}

#[test]
fn date_now_uses_fake_clock_for_handlers_and_timers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = Date.now() + ':';
            setTimeout(() => {
              result.textContent = result.textContent + Date.now();
            }, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.advance_time(7)?;
    h.click("#btn")?;
    h.assert_text("#result", "7:")?;

    h.advance_time(9)?;
    h.assert_text("#result", "7:")?;

    h.advance_time(1)?;
    h.assert_text("#result", "7:17")?;
    Ok(())
}

#[test]
fn date_now_with_flush_advances_to_timer_due_time() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = Date.now();
            setTimeout(() => {
              result.textContent = result.textContent + ':' + Date.now();
            }, 25);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0")?;
    h.flush()?;
    h.assert_text("#result", "0:25")?;
    assert_eq!(h.now_ms(), 25);
    Ok(())
}

#[test]
fn performance_now_uses_fake_clock_for_handlers_and_timers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = performance.now() + ':' + window.performance.now();
            setTimeout(() => {
              result.textContent = result.textContent + ':' + performance.now();
            }, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.advance_time(7)?;
    h.click("#btn")?;
    h.assert_text("#result", "7:7")?;

    h.advance_time(9)?;
    h.assert_text("#result", "7:7")?;

    h.advance_time(1)?;
    h.assert_text("#result", "7:7:17")?;
    Ok(())
}

#[test]
fn date_constructor_and_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nowDate = new Date();
            const fromNumber = new Date(1000);
            const parsed = Date.parse('1970-01-01T00:00:02Z');
            const utc = Date.UTC(1970, 0, 1, 0, 0, 3);
            const parsedViaWindow = window.Date.parse('1970-01-01');
            document.getElementById('result').textContent =
              nowDate.getTime() + ':' + fromNumber.getTime() + ':' +
              parsed + ':' + utc + ':' + parsedViaWindow;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.advance_time(42)?;
    h.click("#btn")?;
    h.assert_text("#result", "42:1000:2000:3000:0")?;
    Ok(())
}

#[test]
fn date_instance_methods_and_set_time_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const d = new Date('2024-03-05T01:02:03Z');
            const y = d.getFullYear();
            const m = d.getMonth();
            const day = d.getDate();
            const h = d.getHours();
            const min = d.getMinutes();
            const s = d.getSeconds();
            const iso = d.toISOString();
            const updated = d.setTime(Date.UTC(1970, 0, 2, 3, 4, 5));
            const iso2 = d.toISOString();
            document.getElementById('result').textContent =
              y + ':' + m + ':' + day + ':' + h + ':' + min + ':' + s +
              '|' + iso + '|' + updated + '|' + iso2;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "2024:2:5:1:2:3|2024-03-05T01:02:03.000Z|97445000|1970-01-02T03:04:05.000Z",
    )?;
    Ok(())
}

#[test]
fn date_instance_method_member_call_supports_constructor_expression_target() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const iso = new Date('2024-03-05T01:02:03Z').toISOString();
            document.getElementById('result').textContent = iso;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2024-03-05T01:02:03.000Z")?;
    Ok(())
}

#[test]
fn date_parse_invalid_input_returns_nan_and_utc_normalizes_overflow() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parsedValue = Date.parse('invalid-date');
            const isInvalid = isNaN(parsedValue);
            const ts = Date.UTC(2020, 12, 1, 25, 61, 61);
            const normalizedDate = new Date(ts);
            const normalized = normalizedDate.toISOString();
            document.getElementById('result').textContent =
              isInvalid + ':' + normalized;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:2021-01-02T02:02:01.000Z")?;
    Ok(())
}

#[test]
fn date_method_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>new Date(1, 2);</script>",
            "new Date supports zero or one argument",
        ),
        (
            "<script>Date.parse();</script>",
            "Date.parse requires exactly one argument",
        ),
        (
            "<script>Date.UTC(1970);</script>",
            "Date.UTC requires between 2 and 7 arguments",
        ),
        (
            "<script>Date.UTC(1970, , 1);</script>",
            "Date.UTC argument cannot be empty",
        ),
        (
            "<script>const d = new Date(); d.getTime(1);</script>",
            "getTime does not take arguments",
        ),
        (
            "<script>const d = new Date(); d.setTime();</script>",
            "setTime requires exactly one argument",
        ),
        (
            "<script>const d = new Date(); d.toISOString(1);</script>",
            "toISOString does not take arguments",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn math_constants_and_symbol_to_string_tag_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.round(Math.E * 1000) + ':' +
              Math.round(Math.LN10 * 1000) + ':' +
              Math.round(Math.LN2 * 1000) + ':' +
              Math.round(Math.LOG10E * 1000) + ':' +
              Math.round(Math.LOG2E * 1000) + ':' +
              Math.round(Math.PI * 1000) + ':' +
              Math.round(Math.SQRT1_2 * 1000) + ':' +
              Math.round(Math.SQRT2 * 1000) + ':' +
              (window.Math.PI === Math.PI) + ':' +
              Math[Symbol.toStringTag];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2718:2303:693:434:1443:3142:707:1414:true:Math")?;
    Ok(())
}

#[test]
fn math_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.abs(-3.5) + ':' +
              Math.acos(1) + ':' +
              Math.acosh(1) + ':' +
              Math.round(Math.asin(1) * 1000) + ':' +
              Math.asinh(0) + ':' +
              Math.round(Math.atan(1) * 1000) + ':' +
              Math.round(Math.atan2(1, 1) * 1000) + ':' +
              Math.round(Math.atanh(0.5) * 1000000) + ':' +
              Math.cbrt(27) + ':' +
              Math.ceil(1.2) + ':' +
              Math.clz32(1) + ':' +
              Math.cos(0) + ':' +
              Math.cosh(0) + ':' +
              Math.round(Math.exp(1) * 1000) + ':' +
              Math.round(Math.expm1(1) * 1000) + ':' +
              Math.floor(1.8) + ':' +
              Math.round(Math.f16round(1.337) * 1000000) + ':' +
              Math.round(Math.fround(1.337) * 1000000) + ':' +
              Math.hypot(3, 4) + ':' +
              Math.imul(2147483647, 2) + ':' +
              Math.round(Math.log(Math.E) * 1000) + ':' +
              Math.log10(1000) + ':' +
              Math.round(Math.log1p(1) * 1000) + ':' +
              Math.log2(8) + ':' +
              Math.max(1, 5, 3) + ':' +
              Math.min(1, 5, 3) + ':' +
              Math.pow(2, 8) + ':' +
              Math.round(1.5) + ':' +
              Math.round(-1.5) + ':' +
              Math.sign(-3) + ':' +
              Math.round(Math.sin(Math.PI / 2) * 1000) + ':' +
              Math.sinh(0) + ':' +
              Math.sqrt(9) + ':' +
              Math.sumPrecise([1, 2, 3]) + ':' +
              Math.tan(0) + ':' +
              Math.tanh(0) + ':' +
              Math.trunc(-1.9) + ':' +
              isNaN(Math.sign(NaN)) + ':' +
              Math.hypot() + ':' +
              isFinite(Math.max()) + ':' +
              isFinite(Math.min());
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "3.5:0:0:1571:0:785:785:549306:3:2:31:1:1:2718:1718:1:1336914:1337000:5:-2:1000:3:693:3:5:1:256:2:-1:-1:1000:0:3:6:0:0:-1:true:0:false:false",
        )?;
    Ok(())
}

#[test]
fn math_sum_precise_requires_array_argument() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Math.sumPrecise(1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("Math.sumPrecise should reject non-array argument");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Math.sumPrecise argument must be an array"))
        }
        other => panic!("unexpected Math.sumPrecise error: {other:?}"),
    }
    Ok(())
}

#[test]
fn math_method_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>Math.abs();</script>",
            "Math.abs requires exactly one argument",
        ),
        (
            "<script>Math.random(1);</script>",
            "Math.random does not take arguments",
        ),
        (
            "<script>Math.atan2(1);</script>",
            "Math.atan2 requires exactly two arguments",
        ),
        (
            "<script>Math.imul(1);</script>",
            "Math.imul requires exactly two arguments",
        ),
        (
            "<script>Math.pow(2);</script>",
            "Math.pow requires exactly two arguments",
        ),
        (
            "<script>Math.sumPrecise();</script>",
            "Math.sumPrecise requires exactly one argument",
        ),
        (
            "<script>Math.max(1, , 2);</script>",
            "Math.max argument cannot be empty",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn math_random_is_deterministic_with_seed() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.random() + ':' + Math.random() + ':' + Math.random();
          });
        </script>
        "#;

    let mut h1 = Harness::from_html(html)?;
    let mut h2 = Harness::from_html(html)?;
    h1.set_random_seed(12345);
    h2.set_random_seed(12345);

    h1.click("#btn")?;
    h2.click("#btn")?;

    let out1 = h1.dump_dom("#result")?;
    let out2 = h2.dump_dom("#result")?;
    assert_eq!(out1, out2);
    Ok(())
}

#[test]
fn math_random_returns_unit_interval() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const r = Math.random();
            if (r >= 0 && r < 1) document.getElementById('result').textContent = 'ok';
            else document.getElementById('result').textContent = 'ng';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_random_seed(42);
    h.click("#btn")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn number_constructor_and_static_properties_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = Number('123');
            const b = Number('12.3');
            const c = Number('');
            const d = Number(null);
            const e = Number('0x11');
            const f = Number('0b11');
            const g = Number('0o11');
            const h = Number('-Infinity') === Number.NEGATIVE_INFINITY;
            const i = Number('foo');
            const j = new Number('5');
            const k = Number.MAX_SAFE_INTEGER === 9007199254740991;
            const l = Number.POSITIVE_INFINITY === Infinity;
            const m = Number.NEGATIVE_INFINITY === -Infinity;
            const n = Number.MIN_VALUE > 0;
            const o = Number.MAX_VALUE > 1e300;
            const p = Number.EPSILON > 0 && Number.EPSILON < 1;
            const q = Number.NaN === Number.NaN;
            const r = window.Number.MAX_SAFE_INTEGER === Number.MAX_SAFE_INTEGER;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' +
              h + ':' + (i === i) + ':' + j + ':' + k + ':' + l + ':' + m + ':' +
              n + ':' + o + ':' + p + ':' + q + ':' + r;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "123:12.3:0:0:17:3:9:true:false:5:true:true:true:true:true:true:false:true",
    )?;
    Ok(())
}

#[test]
fn number_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = Number.isFinite(1 / 3);
            const b = Number.isFinite('1');
            const c = Number.isInteger(3);
            const d = Number.isInteger(3.1);
            const e = Number.isNaN(NaN);
            const f = Number.isNaN('NaN');
            const g = Number.isSafeInteger(9007199254740991);
            const h = Number.isSafeInteger(9007199254740992);
            const i = Number.parseFloat('3.5px');
            const j = Number.parseInt('10', 2);
            const k = window.Number.parseInt('0x10', 16);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' +
              g + ':' + h + ':' + i + ':' + j + ':' + k;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:false:true:false:true:false:true:false:3.5:2:16",
    )?;
    Ok(())
}

#[test]
fn number_instance_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const n = Number('255');
            document.getElementById('result').textContent =
              (12.34).toFixed() + ':' +
              (12.34).toFixed(1) + ':' +
              (12.34).toExponential() + ':' +
              (12.34).toExponential(2) + ':' +
              (12.34).toPrecision() + ':' +
              (12.34).toPrecision(3) + ':' +
              n.toString(16) + ':' +
              n.toString() + ':' +
              (1.5).toString(2) + ':' +
              (1.5).toLocaleString() + ':' +
              (1.5).valueOf();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "12:12.3:1.234e+1:1.23e+1:12.34:12.3:ff:255:1.1:1.5:1.5",
    )?;
    Ok(())
}

#[test]
fn number_method_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>Number(1, 2);</script>",
            "Number supports zero or one argument",
        ),
        (
            "<script>Number.isFinite();</script>",
            "Number.isFinite requires exactly one argument",
        ),
        (
            "<script>window.Number.parseInt();</script>",
            "Number.parseInt requires one or two arguments",
        ),
        (
            "<script>Number.parseInt('10', );</script>",
            "Number.parseInt radix argument cannot be empty",
        ),
        (
            "<script>(1).toFixed(1, 2);</script>",
            "toFixed supports at most one argument",
        ),
        (
            "<script>(1).toLocaleString(1, 2, 3);</script>",
            "toLocaleString supports at most two arguments",
        ),
        (
            "<script>(1).valueOf(1);</script>",
            "valueOf does not take arguments",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn number_instance_method_runtime_range_errors_are_reported() -> Result<()> {
    let html = r#"
        <button id='fixed'>fixed</button>
        <button id='string'>string</button>
        <script>
          document.getElementById('fixed').addEventListener('click', () => {
            (1).toFixed(101);
          });
          document.getElementById('string').addEventListener('click', () => {
            (1).toString(1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let fixed_err = h
        .click("#fixed")
        .expect_err("toFixed should reject out-of-range fractionDigits");
    match fixed_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("toFixed fractionDigits must be between 0 and 100"))
        }
        other => panic!("unexpected toFixed error: {other:?}"),
    }

    let string_err = h
        .click("#string")
        .expect_err("toString should reject out-of-range radix");
    match string_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("toString radix must be between 2 and 36"))
        }
        other => panic!("unexpected toString error: {other:?}"),
    }

    Ok(())
}

#[test]
fn intl_date_time_and_number_format_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const count = 26254.39;
            const date = new Date('2012-05-24');
            const us =
              new Intl.DateTimeFormat('en-US').format(date) + ' ' +
              new Intl.NumberFormat('en-US').format(count);
            const de =
              new Intl.DateTimeFormat('de-DE').format(date) + ' ' +
              new Intl.NumberFormat('de-DE').format(count);
            document.getElementById('result').textContent = us + '|' + de;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5/24/2012 26,254.39|24.5.2012 26.254,39")?;
    Ok(())
}

#[test]
fn intl_uses_navigator_language_preferences() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const date = new Date('2012-05-24');
            const formattedDate = new Intl.DateTimeFormat(navigator.language).format(date);
            const formattedCount = new Intl.NumberFormat(navigator.languages).format(26254.39);
            document.getElementById('result').textContent = formattedDate + '|' + formattedCount;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5/24/2012|26,254.39")?;
    Ok(())
}

#[test]
fn intl_static_methods_and_to_string_tag_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const canonical = Intl.getCanonicalLocales(['EN-us', 'de-de', 'EN-us']);
            const currencies = Intl.supportedValuesOf('currency');
            document.getElementById('result').textContent =
              canonical.join(',') + '|' + currencies.join(',') + '|' + Intl[Symbol.toStringTag];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "en-US,de-DE|EUR,JPY,USD|Intl")?;
    Ok(())
}

#[test]
fn intl_get_canonical_locales_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const one = Intl.getCanonicalLocales('EN-US');
            const two = Intl.getCanonicalLocales(['EN-US', 'Fr']);
            document.getElementById('result').textContent = one.join(',') + '|' + two.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "en-US|en-US,fr")?;

    let html_error = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Intl.getCanonicalLocales('EN_US');
          });
        </script>
        "#;
    let mut h = Harness::from_html(html_error)?;
    let err = h
        .click("#btn")
        .expect_err("invalid language tag should throw");
    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "RangeError: invalid language tag: \"EN_US\"")
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn intl_supported_values_of_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const calendar = Intl.supportedValuesOf('calendar');
            const collation = Intl.supportedValuesOf('collation');
            const currency = Intl.supportedValuesOf('currency');
            const numberingSystem = Intl.supportedValuesOf('numberingSystem');
            const timeZone = Intl.supportedValuesOf('timeZone');
            const unit = Intl.supportedValuesOf('unit');
            document.getElementById('result').textContent =
              calendar.join(',') + '|' +
              collation.join(',') + '|' +
              currency.join(',') + '|' +
              numberingSystem.join(',') + '|' +
              timeZone.join(',') + '|' +
              unit.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "gregory,islamic-umalqura,japanese|default,emoji,phonebk|EUR,JPY,USD|arab,latn,thai|America/Los_Angeles,America/New_York,Asia/Kolkata,UTC|day,hour,meter,minute,month,second,week,year",
        )?;

    let html_error = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Intl.supportedValuesOf('someInvalidKey');
          });
        </script>
        "#;
    let mut h = Harness::from_html(html_error)?;
    let err = h
        .click("#btn")
        .expect_err("invalid supportedValuesOf key should throw");
    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "RangeError: invalid key: \"someInvalidKey\"")
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn intl_namespace_is_not_a_constructor() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const i = new Intl();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h.click("#btn").expect_err("new Intl should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Intl is not a constructor")),
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn intl_date_time_format_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const date = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));
            const us = new Intl.DateTimeFormat('en-US').format(date);
            const fallback = new Intl.DateTimeFormat(['ban', 'id']).format(date);
            const styled = new Intl.DateTimeFormat('en-GB', {
              dateStyle: 'full',
              timeStyle: 'long',
              timeZone: 'Australia/Sydney',
            }).format(date);
            document.getElementById('result').textContent = us + '|' + fallback + '|' + styled;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "12/20/2020|20/12/2020|Sunday, 20 December 2020 at 14:23:16 GMT+11",
    )?;
    Ok(())
}

#[test]
fn intl_date_time_format_instance_methods_and_getter_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const dtf = new Intl.DateTimeFormat('en-US', {
              year: 'numeric',
              month: '2-digit',
              day: '2-digit',
              timeZone: 'UTC'
            });
            const d1 = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));
            const d2 = new Date(Date.UTC(2020, 11, 21, 3, 23, 16, 738));
            const fmt = dtf.format;
            const fromGetter = fmt(d1);
            const parts = dtf.formatToParts(d1);
            const range = dtf.formatRange(d1, d2);
            const rangeParts = dtf.formatRangeToParts(d1, d2);
            const partsOk = JSON.stringify(parts).includes('"type":"month"');
            const rangePartsOk =
              JSON.stringify(rangeParts).includes('"source":"startRange"') &&
              JSON.stringify(rangeParts).includes('"source":"endRange"');
            document.getElementById('result').textContent =
              fromGetter + '|' + range + '|' + partsOk + ':' + rangePartsOk;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12/20/2020|12/20/2020 - 12/21/2020|true:true")?;
    Ok(())
}

#[test]
fn intl_date_time_format_supported_locales_and_resolved_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const supportedLocales = Intl.DateTimeFormat.supportedLocalesOf(['ban', 'id', 'en-GB', 'fr']);
            const supported = supportedLocales.join(',');
            const ro = new Intl.DateTimeFormat('ja-JP-u-ca-japanese', {
              numberingSystem: 'arab',
              timeZone: 'America/Los_Angeles',
              dateStyle: 'short',
            }).resolvedOptions();
            const tag = Intl.DateTimeFormat.prototype[Symbol.toStringTag];
            const ar = new Intl.DateTimeFormat('ar-EG').format(new Date(Date.UTC(2012, 11, 20, 3, 0, 0)));
            document.getElementById('result').textContent =
              supported + '|' + ro.locale + ':' + ro.calendar + ':' + ro.numberingSystem + ':' +
              ro.timeZone + ':' + ro.dateStyle + '|' + tag + '|' + ar;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "id,en-GB|ja-JP-u-ca-japanese:japanese:arab:America/Los_Angeles:short|Intl.DateTimeFormat|٢٠/١٢/٢٠١٢",
        )?;
    Ok(())
}

#[test]
fn intl_duration_format_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const duration = {
              hours: 1,
              minutes: 46,
              seconds: 40,
            };
            const fr = new Intl.DurationFormat('fr-FR', { style: 'long' }).format(duration);
            const en = new Intl.DurationFormat('en', { style: 'short' }).format(duration);
            const pt = new Intl.DurationFormat('pt', { style: 'narrow' }).format(duration);
            document.getElementById('result').textContent = fr + '|' + en + '|' + pt;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "1 heure, 46 minutes et 40 secondes|1 hr, 46 min and 40 sec|1 h 46 min 40 s",
    )?;
    Ok(())
}

#[test]
fn intl_duration_format_instance_methods_and_static_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const duration = {
              hours: 1,
              minutes: 2,
              seconds: 3,
            };
            const df = new Intl.DurationFormat('en', { style: 'short' });
            const fmt = df.format;
            const fromGetter = fmt(duration);
            const parts = df.formatToParts(duration);
            const partsOk =
              JSON.stringify(parts).includes('"type":"hour"') &&
              JSON.stringify(parts).includes('"type":"literal"');
            const supported = Intl.DurationFormat.supportedLocalesOf(['fr-FR', 'en', 'pt', 'de']);
            const ro = new Intl.DurationFormat('pt', { style: 'narrow' }).resolvedOptions();
            const tag = Intl.DurationFormat.prototype[Symbol.toStringTag];
            document.getElementById('result').textContent =
              supported.join(',') + '|' + fromGetter + '|' + partsOk + '|' +
              ro.locale + ':' + ro.style + '|' + tag;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "fr-FR,en,pt|1 hr, 2 min and 3 sec|true|pt:narrow|Intl.DurationFormat",
    )?;
    Ok(())
}

#[test]
fn intl_list_format_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const vehicles = ['Motorcycle', 'Bus', 'Car'];
            const formatter = new Intl.ListFormat('en', {
              style: 'long',
              type: 'conjunction',
            });
            const formatter2 = new Intl.ListFormat('de', {
              style: 'short',
              type: 'disjunction',
            });
            const formatter3 = new Intl.ListFormat('en', { style: 'narrow', type: 'unit' });
            document.getElementById('result').textContent =
              formatter.format(vehicles) + '|' +
              formatter2.format(vehicles) + '|' +
              formatter3.format(vehicles);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Motorcycle, Bus, and Car|Motorcycle, Bus oder Car|Motorcycle Bus Car",
    )?;
    Ok(())
}

#[test]
fn intl_list_format_methods_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const list = ['Motorcycle', 'Bus', 'Car'];
            const long = new Intl.ListFormat('en-GB', { style: 'long', type: 'conjunction' }).format(list);
            const short = new Intl.ListFormat('en-GB', { style: 'short', type: 'disjunction' }).format(list);
            const narrow = new Intl.ListFormat('en-GB', { style: 'narrow', type: 'unit' }).format(list);
            const parts = new Intl.ListFormat('en-GB', {
              style: 'long',
              type: 'conjunction',
            }).formatToParts(list);
            const partsOk =
              JSON.stringify(parts).includes('"type":"element"') &&
              JSON.stringify(parts).includes('"value":" and "');
            const supportedLocales = Intl.ListFormat.supportedLocalesOf(['de', 'en-GB', 'fr']);
            const supported = supportedLocales.join(',');
            const ro = new Intl.ListFormat('en-GB', {
              style: 'short',
              type: 'disjunction'
            }).resolvedOptions();
            const roTypeOk = JSON.stringify(ro).includes('"type":"disjunction"');
            const tag = Intl.ListFormat.prototype[Symbol.toStringTag];
            const defaultList = new Intl.ListFormat('en');
            const ctor = defaultList.constructor === Intl.ListFormat;
            const format = defaultList.format;
            const fromGetter = format(list);
            document.getElementById('result').textContent =
              long + '|' + short + '|' + narrow + '|' + partsOk + '|' +
              supported + '|' + ro.locale + ':' + ro.style + ':' + roTypeOk + '|' +
              tag + '|' + ctor + '|' + fromGetter;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "Motorcycle, Bus and Car|Motorcycle, Bus or Car|Motorcycle Bus Car|true|de,en-GB|en-GB:short:true|Intl.ListFormat|true|Motorcycle, Bus, and Car",
        )?;
    Ok(())
}

#[test]
fn intl_locale_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const korean = new Intl.Locale('ko', {
              script: 'Kore',
              region: 'KR',
              hourCycle: 'h23',
              calendar: 'gregory',
            });
            const japanese = new Intl.Locale('ja-Jpan-JP-u-ca-japanese-hc-h12');
            document.getElementById('result').textContent =
              korean.baseName + '|' + japanese.baseName + '|' +
              korean.hourCycle + '|' + japanese.hourCycle;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ko-Kore-KR|ja-Jpan-JP|h23|h12")?;
    Ok(())
}

#[test]
fn intl_locale_properties_and_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const locale = new Intl.Locale('en-Latn-US-u-ca-gregory-kf-upper-co-phonebk-kn-nu-latn');
            const us = new Intl.Locale('en-US', { hourCycle: 'h12' });
            const textInfo = new Intl.Locale('he-IL').getTextInfo();
            const weekInfo = us.getWeekInfo();
            const weekend = weekInfo.weekend;
            const maximize = new Intl.Locale('zh').maximize();
            const minimize = maximize.minimize();
            const props =
              locale.language + ':' + locale.script + ':' + locale.region + ':' +
              locale.calendar + ':' + locale.caseFirst + ':' + locale.collation + ':' +
              locale.numberingSystem + ':' + locale.numeric + ':' +
              locale.hourCycle + ':' + locale.variants.length;
            const calendars = us.getCalendars();
            const collations = us.getCollations();
            const hourCycles = us.getHourCycles();
            const numberingSystems = us.getNumberingSystems();
            const timeZones = us.getTimeZones();
            const tag = Intl.Locale.prototype[Symbol.toStringTag];
            const ctor = us.constructor === Intl.Locale;
            const full = us.toString();
            document.getElementById('result').textContent =
              props + '|' + calendars.join(',') + '|' + collations.join(',') + '|' + hourCycles.join(',') + '|' +
              numberingSystems.join(',') + '|' + textInfo.direction + '|' + timeZones.join(',') + '|' +
              weekInfo.firstDay + ':' + weekInfo.minimalDays + ':' + weekend.join('/') + '|' +
              maximize.baseName + ':' + minimize.baseName + '|' + tag + '|' + ctor + '|' + full;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "en:Latn:US:gregory:upper:phonebk:latn:true:undefined:0|gregory|default,emoji|h12,h23|latn|rtl|America/New_York,America/Los_Angeles|7:1:6/7|zh-Hans-CN:zh|Intl.Locale|true|en-US-u-hc-h12",
        )?;
    Ok(())
}

#[test]
fn intl_plural_rules_locales_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const enCardinalRules = new Intl.PluralRules('en-US');
            const arCardinalRules = new Intl.PluralRules('ar-EG');
            const enOrdinalRules = new Intl.PluralRules('en-US', { type: 'ordinal' });
            document.getElementById('result').textContent =
              enCardinalRules.select(0) + ':' + enCardinalRules.select(1) + ':' +
              enCardinalRules.select(2) + ':' + enCardinalRules.select(3) + '|' +
              arCardinalRules.select(0) + ':' + arCardinalRules.select(1) + ':' +
              arCardinalRules.select(2) + ':' + arCardinalRules.select(6) + ':' +
              arCardinalRules.select(18) + '|' +
              enOrdinalRules.select(0) + ':' + enOrdinalRules.select(1) + ':' +
              enOrdinalRules.select(2) + ':' + enOrdinalRules.select(3) + ':' +
              enOrdinalRules.select(4) + ':' + enOrdinalRules.select(21);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "other:one:other:other|zero:one:two:few:many|other:one:two:few:other:one",
    )?;
    Ok(())
}

#[test]
fn intl_plural_rules_methods_and_static_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const enOrdinalRules = new Intl.PluralRules('en-US', { type: 'ordinal' });
            const supported = Intl.PluralRules.supportedLocalesOf(['ar-EG', 'en-US', 'de']);
            const ro = enOrdinalRules.resolvedOptions();
            const categories = ro.pluralCategories;
            const categoriesText = categories.join(',');
            const range =
              enOrdinalRules.selectRange(1, 1) + ':' +
              enOrdinalRules.selectRange(1, 2) + ':' +
              new Intl.PluralRules('ar-EG').selectRange(0, 0);
            const suffixes = { one: 'st', two: 'nd', few: 'rd', other: 'th' };
            const formatOrdinals = (n) => {
              const rule = enOrdinalRules.select(n);
              return n + suffixes[rule];
            };
            const tag = Intl.PluralRules.prototype[Symbol.toStringTag];
            const ctor = enOrdinalRules.constructor === Intl.PluralRules;
            document.getElementById('result').textContent =
              supported.join(',') + '|' +
              ro.locale + ':' + ro['type'] + ':' + categoriesText + '|' +
              range + '|' +
              formatOrdinals(0) + ',' + formatOrdinals(1) + ',' + formatOrdinals(2) + ',' +
              formatOrdinals(3) + ',' + formatOrdinals(4) + ',' + formatOrdinals(11) + ',' +
              formatOrdinals(21) + ',' + formatOrdinals(42) + ',' + formatOrdinals(103) + '|' +
              tag + '|' + ctor;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "ar-EG,en-US|en-US:ordinal:one,two,few,other|one:other:zero|0th,1st,2nd,3rd,4th,11th,21st,42nd,103rd|Intl.PluralRules|true",
        )?;
    Ok(())
}

#[test]
fn intl_relative_time_format_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const rtf1 = new Intl.RelativeTimeFormat('en', { style: 'short' });
            const qtrs = rtf1.format(3, 'quarter');
            const ago = rtf1.format(-1, 'day');
            const rtf2 = new Intl.RelativeTimeFormat('es', { numeric: 'auto' });
            const auto = rtf2.format(2, 'day');
            document.getElementById('result').textContent = qtrs + '|' + ago + '|' + auto;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "in 3 qtrs.|1 day ago|pasado mañana")?;
    Ok(())
}

#[test]
fn intl_relative_time_format_methods_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const rtf = new Intl.RelativeTimeFormat('en', {
              localeMatcher: 'best fit',
              numeric: 'auto',
              style: 'long',
            });
            const text = rtf.format(-1, 'day');
            const partsAuto = rtf.formatToParts(-1, 'day');
            const parts = rtf.formatToParts(100, 'day');
            const partsOk =
              JSON.stringify(partsAuto).includes('"value":"yesterday"') &&
              JSON.stringify(parts).includes('"type":"integer"') &&
              JSON.stringify(parts).includes('"unit":"day"') &&
              JSON.stringify(parts).includes('"value":"in "') &&
              JSON.stringify(parts).includes('"value":"100"') &&
              JSON.stringify(parts).includes('"value":" days"');
            const supportedLocales = Intl.RelativeTimeFormat.supportedLocalesOf(['es', 'en', 'de']);
            const supported = supportedLocales.join(',');
            const ro = rtf.resolvedOptions();
            const tag = Intl.RelativeTimeFormat.prototype[Symbol.toStringTag];
            const ctor = rtf.constructor === Intl.RelativeTimeFormat;
            document.getElementById('result').textContent =
              text + '|' + partsOk + '|' + supported + '|' +
              ro.locale + ':' + ro.style + ':' + ro.numeric + ':' + ro.localeMatcher + '|' +
              tag + '|' + ctor;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "yesterday|true|es,en|en:long:auto:best fit|Intl.RelativeTimeFormat|true",
    )?;
    Ok(())
}
