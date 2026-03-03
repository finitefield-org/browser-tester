use super::*;

#[test]
fn element_slot_returns_empty_string_when_unset_and_reflects_attribute_changes() -> Result<()> {
    let html = r#"
        <span id='light'>Light text</span>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const light = document.getElementById('light');
            const initialEmpty = light.slot === '';

            light.setAttribute('slot', 'my-text');
            const assigned = light.slot + ':' + light.getAttribute('slot');

            light.removeAttribute('slot');
            const restoredEmpty = light.slot === '';

            document.getElementById('result').textContent =
              initialEmpty + ':' + assigned + ':' + restoredEmpty;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:my-text:my-text:true")?;
    Ok(())
}

#[test]
fn element_slot_assignment_reflects_attribute_roundtrip() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.slot = 'hero';
            const first = box.slot + ':' + box.getAttribute('slot');

            box.slot = 'footer';
            const second = box.slot + ':' + box.getAttribute('slot');

            box.removeAttribute('slot');
            const restored = box.slot + ':' + (box.getAttribute('slot') === null);

            document.getElementById('result').textContent =
              first + '|' + second + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "hero:hero|footer:footer|:true")?;
    Ok(())
}

#[test]
fn element_slot_mdn_example_like_usage_returns_matching_slot_name() -> Result<()> {
    let html = r#"
        <my-paragraph id='paragraph'>
          <span slot='my-text'>Let's have some different text!</span>
        </my-paragraph>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('paragraph');
            const shadow = host.attachShadow({ mode: 'open' });
            shadow.innerHTML = '<p><slot name="my-text"></slot></p>';

            const slottedSpan = document.querySelector('my-paragraph span');
            const slotName = shadow.querySelector('slot').name;
            document.getElementById('result').textContent =
              slottedSpan.slot + ':' + slotName + ':' + (slottedSpan.slot === slotName);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "my-text:my-text:true")?;
    Ok(())
}
