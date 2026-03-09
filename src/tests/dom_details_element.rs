use super::*;

#[test]
fn details_summary_click_toggles_open_and_emits_toggle_event_states() -> Result<()> {
    let html = r#"
        <details id='panel'>
          <summary id='toggle'>Details</summary>
          <p>Something small enough to escape casual notice.</p>
        </details>
        <button id='read'>read</button>
        <p id='result'></p>
        <script>
          const panel = document.getElementById('panel');
          let logs = '';
          panel.addEventListener('toggle', (event) => {
            logs = logs + event.oldState + '>' + event.newState + ';';
          });
          document.getElementById('read').addEventListener('click', () => {
            document.getElementById('result').textContent =
              panel.role + ':' + panel.open + ':' + logs;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#read")?;
    h.assert_text("#result", "group:false:")?;

    h.click("#toggle")?;
    h.click("#read")?;
    h.assert_text("#result", "group:true:closed>open;")?;

    h.click("#toggle")?;
    h.click("#read")?;
    h.assert_text("#result", "group:false:closed>open;open>closed;")?;
    Ok(())
}

#[test]
fn details_name_group_keeps_single_open_item_for_initial_and_dynamic_changes() -> Result<()> {
    let html = r#"
        <details id='one' name='requirements' open>
          <summary id='one-summary'>Graduation Requirements</summary>
          <p>Requires 40 credits.</p>
        </details>
        <details id='two' name='requirements' open>
          <summary id='two-summary'>System Requirements</summary>
          <p>Requires a computer.</p>
        </details>
        <div>
          <details id='three' name='requirements'>
            <summary id='three-summary'>Job Requirements</summary>
            <p>Requires HTML, CSS, and JavaScript.</p>
          </details>
        </div>
        <button id='open-three'>open three</button>
        <button id='read'>read</button>
        <p id='result'></p>
        <script>
          let logs = '';
          ['one', 'two', 'three'].forEach((id) => {
            const details = document.getElementById(id);
            details.addEventListener('toggle', (event) => {
              logs = logs + id + ':' + event.oldState + '>' + event.newState + '|';
            });
          });

          document.getElementById('open-three').addEventListener('click', () => {
            document.getElementById('three').open = true;
          });

          document.getElementById('read').addEventListener('click', () => {
            const one = document.getElementById('one');
            const two = document.getElementById('two');
            const three = document.getElementById('three');
            document.getElementById('result').textContent =
              (one.open ? '1' : '0') +
              (two.open ? '1' : '0') +
              (three.open ? '1' : '0') + ':' +
              logs;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#read")?;
    h.assert_text("#result", "100:")?;

    h.click("#two-summary")?;
    h.click("#read")?;
    h.assert_text("#result", "010:two:closed>open|one:open>closed|")?;

    h.click("#open-three")?;
    h.click("#read")?;
    h.assert_text(
        "#result",
        "001:two:closed>open|one:open>closed|three:closed>open|two:open>closed|",
    )?;
    Ok(())
}

#[test]
fn details_toggle_events_stay_local_to_the_details_element_work() -> Result<()> {
    let html = r#"
        <div id='host'>
          <details id='one' name='faq' open>
            <summary id='one-summary'>One</summary>
            <p>First</p>
          </details>
          <details id='two' name='faq'>
            <summary id='two-summary'>Two</summary>
            <p>Second</p>
          </details>
        </div>
        <button id='read'>read</button>
        <p id='result'></p>
        <script>
          const host = document.getElementById('host');
          const log = [];

          host.addEventListener('toggle', (event) => {
            log.push('host:' + event.target.id);
          });

          ['one', 'two'].forEach((id) => {
            const details = document.getElementById(id);
            details.addEventListener('toggle', (event) => {
              log.push([
                id,
                event.oldState,
                event.newState,
                String(event.bubbles),
                String(event.cancelable)
              ].join(':'));
            });
          });

          document.getElementById('read').addEventListener('click', () => {
            document.getElementById('result').textContent = [
              String(document.getElementById('one').open),
              String(document.getElementById('two').open),
              log.join('|')
            ].join(';');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#two-summary")?;
    h.click("#read")?;
    h.assert_text(
        "#result",
        "false;true;two:closed:open:false:false|one:open:closed:false:false",
    )?;
    Ok(())
}

#[test]
fn summary_click_prevent_default_blocks_details_toggle_default_action_work() -> Result<()> {
    let html = r#"
        <details id='panel'>
          <summary id='toggle'>Toggle</summary>
          <p>Body</p>
        </details>
        <button id='arm'>arm</button>
        <button id='read'>read</button>
        <p id='result'></p>
        <script>
          const panel = document.getElementById('panel');
          const summary = document.getElementById('toggle');
          const log = [];
          let block = false;

          panel.addEventListener('toggle', () => {
            log.push('toggle');
          });
          summary.addEventListener('click', (event) => {
            if (!block) return;
            log.push('prevent');
            event.preventDefault();
          });

          document.getElementById('arm').addEventListener('click', () => {
            block = true;
          });
          document.getElementById('read').addEventListener('click', () => {
            document.getElementById('result').textContent =
              String(panel.open) + ':' + log.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#toggle")?;
    h.click("#arm")?;
    h.click("#toggle")?;
    h.click("#read")?;
    h.assert_text("#result", "true:toggle|prevent")?;
    Ok(())
}
