use super::*;

#[test]
fn slot_name_attribute_and_fallback_content_work() -> Result<()> {
    let html = r#"
        <div id='component'>
          <slot id='named' name='description'>NEED DESCRIPTION</slot>
          <slot id='unnamed'>DEFAULT CONTENT</slot>
        </div>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const named = document.getElementById('named');
            const unnamed = document.getElementById('unnamed');

            document.getElementById('result').textContent =
              named.name + ':' +
              named.getAttribute('name') + ':' +
              unnamed.name + ':' +
              (unnamed.getAttribute('name') === null) + ':' +
              named.textContent.includes('NEED DESCRIPTION') + ':' +
              unnamed.textContent.trim() + ':' +
              named.role + ':' +
              named.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "description:description::true:true:DEFAULT CONTENT::SLOT",
    )?;
    Ok(())
}

#[test]
fn slot_slotchange_event_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <slot id='watcher' name='item'>Fallback</slot>
        <div id='light' slot='item'>Light node</div>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          const watcher = document.getElementById('watcher');
          let slotChanges = 0;

          watcher.addEventListener('slotchange', () => {
            slotChanges += 1;
          });

          document.getElementById('run').addEventListener('click', () => {
            const initial =
              watcher.role + ':' +
              document.getElementById('light').slot + ':' +
              (watcher.assignedSlot === null) + ':' +
              slotChanges;

            watcher.role = 'status';
            const assigned = watcher.role + ':' + watcher.getAttribute('role');

            watcher.removeAttribute('role');
            const restored = watcher.role + ':' + (watcher.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#watcher", "slotchange")?;
    h.click("#run")?;
    h.assert_text("#result", ":item:true:1|status:status|:true")?;
    Ok(())
}
