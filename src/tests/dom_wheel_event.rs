use super::*;

#[test]
fn wheel_event_constructor_populates_delta_properties_and_constants() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const e = new WheelEvent('wheel', {
            deltaX: 1.5,
            deltaY: -2,
            deltaZ: 0.25,
            deltaMode: WheelEvent.DOM_DELTA_LINE,
            bubbles: true,
            cancelable: true
          });
          document.getElementById('result').textContent = [
            e.type,
            e.deltaX,
            e.deltaY,
            e.deltaZ,
            e.deltaMode,
            e.bubbles,
            e.cancelable,
            WheelEvent.DOM_DELTA_PIXEL,
            WheelEvent.DOM_DELTA_LINE,
            WheelEvent.DOM_DELTA_PAGE
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "wheel:1.5:-2:0.25:1:true:true:0:1:2")?;
    Ok(())
}

#[test]
fn dispatch_event_with_wheel_event_payload_preserves_delta_fields() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.addEventListener('wheel', (event) => {
            document.getElementById('result').textContent = [
              event.deltaX,
              event.deltaY,
              event.deltaZ,
              event.deltaMode,
              event.isTrusted
            ].join(':');
          });

          document.getElementById('run').addEventListener('click', () => {
            const ev = new WheelEvent('wheel', {
              deltaX: 3,
              deltaY: 4,
              deltaZ: 5,
              deltaMode: WheelEvent.DOM_DELTA_PAGE
            });
            target.dispatchEvent(ev);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:4:5:2:false")?;
    Ok(())
}

#[test]
fn wheel_event_constructor_rejects_non_object_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              new WheelEvent('wheel', 123);
            } catch (error) {
              threw = String(error).includes('options argument must be an object');
            }
            document.getElementById('result').textContent = String(threw);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true")?;
    Ok(())
}
