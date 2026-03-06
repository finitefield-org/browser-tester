use browser_tester::Harness;

#[test]
fn issue_134_object_assign_global_is_available() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const out = document.getElementById('out');
        try {
          const target = { a: 1 };
          const src = { b: 2 };
          Object.assign(target, src);
          out.textContent = String(target.a) + ':' + String(target.b);
        } catch (err) {
          out.textContent = 'err:' + String(err && err.message ? err.message : err);
        }
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1:2")?;
    Ok(())
}

#[test]
fn issue_134_object_assign_returns_target_and_ignores_nullish_sources() -> browser_tester::Result<()>
{
    let html = r#"
      <p id='out'></p>
      <script>
        const target = { a: 1, b: 1 };
        const returned = Object.assign(target, null, { b: 4 }, undefined, { c: 5 });
        document.getElementById('out').textContent = [
          String(target.a),
          String(target.b),
          String(target.c),
          String(returned === target),
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|4|5|true")?;
    Ok(())
}

#[test]
fn issue_134_object_assign_copies_symbol_and_string_source_keys() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const sym = Symbol('token');
        const copied = Object.assign({}, { [sym]: 'sym', x: 'x' });
        const fromString = Object.assign({}, 'abc');
        const symbols = Object.getOwnPropertySymbols(copied);
        document.getElementById('out').textContent = [
          String(symbols.length),
          String(copied[sym]),
          String(copied.x),
          Object.keys(fromString).join(','),
          fromString[0] + fromString[1] + fromString[2],
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|sym|x|0,1,2|abc")?;
    Ok(())
}

#[test]
fn issue_134_object_assign_uses_getters_and_setters() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        let getCount = 0;
        let setTotal = 0;
        const source = {
          get amount() {
            getCount += 1;
            return 7;
          }
        };
        const target = {
          set amount(value) {
            setTotal += value;
          }
        };
        Object.assign(target, source);
        document.getElementById('out').textContent = [
          String(getCount),
          String(setTotal),
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|7")?;
    Ok(())
}

#[test]
fn issue_134_object_assign_wraps_primitive_target_and_rejects_null_target()
-> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const wrapped = Object.assign(3, { a: 1 });
        let threwForNull = false;
        try {
          Object.assign(null, { a: 1 });
        } catch (err) {
          threwForNull = String(err && err.message ? err.message : err)
            .includes('Cannot convert undefined or null to object');
        }
        document.getElementById('out').textContent = [
          typeof wrapped,
          String(wrapped.a),
          String(threwForNull),
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "object|1|true")?;
    Ok(())
}

#[test]
fn issue_135_optional_chaining_listener_on_member_path_parses_and_runs()
-> browser_tester::Result<()> {
    let html = r#"
      <button id='btn'>run</button>
      <p id='out'></p>
      <script>
        const actionEls = { close: document.getElementById('btn') };
        actionEls.close?.addEventListener('click', () => {
          document.getElementById('out').textContent = 'ok';
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#btn")?;
    harness.assert_text("#out", "ok")?;
    Ok(())
}

#[test]
fn issue_136_html_button_element_global_supports_instanceof_checks() -> browser_tester::Result<()> {
    let html = r#"
      <button id='btn'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('btn').addEventListener('click', (event) => {
          const checks = [
            typeof HTMLButtonElement,
            String(window.HTMLButtonElement === HTMLButtonElement),
            String(event.target instanceof HTMLButtonElement),
            String(event.target instanceof HTMLElement),
          ];
          document.getElementById('out').textContent = checks.join('|');
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#btn")?;
    harness.assert_text("#out", "function|true|true|true")?;
    Ok(())
}

#[test]
fn issue_137_tofixed_chain_parses_after_escape_normalization_with_unicode()
-> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const quotePair = "\"\"";
        const label = `ABC-001 (${quotePair.length} 件)`;
        const rect = { w: 4.2 };
        const formatted = Math.max(0, rect.w).toFixed(2);
        document.getElementById('out').textContent = `${label}|${formatted}`;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "ABC-001 (2 件)|4.20")?;
    Ok(())
}
