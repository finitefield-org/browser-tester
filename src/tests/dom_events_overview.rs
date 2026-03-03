use super::*;

#[test]
fn custom_event_detail_bubbles_and_cancelable_dispatches() -> Result<()> {
    let html = r#"
        <div id='root'>
          <textarea id='field'></textarea>
        </div>
        <p id='result'></p>
        <script>
          const root = document.getElementById('root');
          const field = document.getElementById('field');

          root.addEventListener('awesome', (event) => {
            const textFn = event.detail.text;
            event.preventDefault();
            document.getElementById('result').textContent = [
              event.type,
              event.target.id,
              event.currentTarget.id,
              typeof textFn,
              textFn(),
              String(event.bubbles),
              String(event.cancelable),
            ].join(':');
          });

          field.addEventListener('input', () => {
            const custom = new CustomEvent('awesome', {
              bubbles: true,
              cancelable: true,
              detail: { text: () => field.value },
            });
            const ok = field.dispatchEvent(custom);
            document.getElementById('result').textContent += ':ret=' + String(ok);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#field", "hello")?;
    h.assert_text(
        "#result",
        "awesome:field:root:function:hello:true:true:ret=false",
    )?;
    Ok(())
}

#[test]
fn mouse_event_constructor_supports_synthetic_click_dispatch() -> Result<()> {
    let html = r#"
        <input id='checkbox' type='checkbox'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const cb = document.getElementById('checkbox');
          const out = document.getElementById('result');
          cb.addEventListener('click', (event) => {
            event.preventDefault();
          });

          document.getElementById('run').addEventListener('click', () => {
            const event = new MouseEvent('click', {
              view: window,
              bubbles: true,
              cancelable: true,
            });
            const cancelled = !cb.dispatchEvent(event);
            out.textContent = [
              String(cancelled),
              String(cb.checked),
              event.type,
              String(event.bubbles),
              String(event.cancelable),
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:click:true:true")?;
    Ok(())
}

#[test]
fn onevent_slot_is_independent_from_add_event_listener_and_keeps_order() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          const out = document.getElementById('result');

          function shared() {
            out.textContent += 'S';
          }
          function tail() {
            out.textContent += 'T';
          }
          function swapped() {
            out.textContent += 'H';
          }

          target.addEventListener('x', shared);
          target.onx = shared;
          target.addEventListener('x', tail);

          document.getElementById('run').addEventListener('click', () => {
            out.textContent = '';
            target.dispatchEvent(new Event('x'));
            const first = out.textContent;

            out.textContent = '';
            target.onx = swapped;
            target.dispatchEvent(new Event('x'));
            const second = out.textContent;

            out.textContent = first + '|' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "SST|SHT")?;
    Ok(())
}
