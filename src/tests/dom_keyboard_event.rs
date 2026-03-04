use super::*;

#[test]
fn keyboard_event_constructor_populates_core_properties_and_constants() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const e = new KeyboardEvent('keydown', {
            key: 'A',
            code: 'KeyA',
            location: KeyboardEvent.DOM_KEY_LOCATION_LEFT,
            ctrlKey: true,
            shiftKey: true,
            repeat: true
          });
          document.getElementById('result').textContent = [
            e.type,
            e.key,
            e.code,
            e.location,
            e.ctrlKey,
            e.metaKey,
            e.shiftKey,
            e.altKey,
            e.repeat,
            e.isComposing,
            e.keyCode,
            e.charCode,
            e.keyIdentifier,
            e.getModifierState('Control'),
            e.getModifierState('Shift'),
            e.getModifierState('Alt'),
            KeyboardEvent.DOM_KEY_LOCATION_STANDARD,
            KeyboardEvent.DOM_KEY_LOCATION_LEFT,
            KeyboardEvent.DOM_KEY_LOCATION_RIGHT,
            KeyboardEvent.DOM_KEY_LOCATION_NUMPAD
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "keydown:A:KeyA:1:true:false:true:false:true:false:65:0:A:true:true:false:0:1:2:3",
    )?;
    Ok(())
}

#[test]
fn dispatch_keyboard_exposes_location_legacy_codes_and_modifier_state() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          window.addEventListener('keydown', (event) => {
            document.getElementById('result').textContent = [
              event.location,
              event.keyCode,
              event.charCode,
              event.getModifierState('Shift'),
              event.getModifierState('Control')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch_keyboard(
        "window",
        "keydown",
        KeyboardEventInit {
            key: "Enter".to_string(),
            code: Some("Enter".to_string()),
            location: 3,
            shift_key: true,
            ..Default::default()
        },
    )?;
    h.assert_text("#result", "3:13:0:true:false")?;
    Ok(())
}

#[test]
fn dispatch_event_with_keyboard_event_payload_preserves_keyboard_fields() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.addEventListener('keydown', (event) => {
            document.getElementById('result').textContent = [
              event.key,
              event.code,
              event.location,
              event.altKey,
              event.isComposing,
              event.getModifierState('Alt'),
              event.keyCode,
              event.charCode,
              event.isTrusted
            ].join(':');
          });

          document.getElementById('run').addEventListener('click', () => {
            const ev = new KeyboardEvent('keydown', {
              key: 'z',
              code: 'KeyZ',
              location: KeyboardEvent.DOM_KEY_LOCATION_RIGHT,
              altKey: true,
              isComposing: true
            });
            target.dispatchEvent(ev);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "z:KeyZ:2:true:true:true:122:0:false")?;
    Ok(())
}

#[test]
fn keyboard_event_constructor_rejects_non_object_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              new KeyboardEvent('keydown', 123);
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
