use super::*;

#[test]
fn pointer_event_constructor_populates_pointer_properties_and_methods() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const e = new PointerEvent('pointermove', {
            pointerId: 7,
            width: 10.5,
            height: 8,
            pressure: 0.75,
            tangentialPressure: -0.5,
            tiltX: 20,
            tiltY: -10,
            twist: 180,
            pointerType: 'pen',
            isPrimary: true,
            altitudeAngle: 0.7,
            azimuthAngle: 1.2,
            persistentDeviceId: 99,
            bubbles: true,
            cancelable: true
          });
          document.getElementById('result').textContent = [
            e.type,
            e.pointerId,
            e.width,
            e.height,
            e.pressure,
            e.tangentialPressure,
            e.tiltX,
            e.tiltY,
            e.twist,
            e.pointerType,
            e.isPrimary,
            e.altitudeAngle,
            e.azimuthAngle,
            e.persistentDeviceId,
            e.bubbles,
            e.cancelable,
            typeof e.getCoalescedEvents,
            typeof e.getPredictedEvents,
            e.getCoalescedEvents().length,
            e.getPredictedEvents().length
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "pointermove:7:10.5:8:0.75:-0.5:20:-10:180:pen:true:0.7:1.2:99:true:true:function:function:0:0",
    )?;
    Ok(())
}

#[test]
fn dispatch_event_with_pointer_event_payload_preserves_pointer_fields() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.addEventListener('pointermove', (event) => {
            document.getElementById('result').textContent = [
              event.pointerId,
              event.width,
              event.height,
              event.pressure,
              event.tangentialPressure,
              event.tiltX,
              event.tiltY,
              event.twist,
              event.pointerType,
              event.isPrimary,
              event.altitudeAngle,
              event.azimuthAngle,
              event.persistentDeviceId,
              Array.isArray(event.getCoalescedEvents()),
              Array.isArray(event.getPredictedEvents()),
              event.isTrusted
            ].join(':');
          });

          document.getElementById('run').addEventListener('click', () => {
            const ev = new PointerEvent('pointermove', {
              pointerId: 5,
              width: 2.5,
              height: 3.5,
              pressure: 0.9,
              tangentialPressure: 0.2,
              tiltX: 30,
              tiltY: -15,
              twist: 270,
              pointerType: 'touch',
              isPrimary: true,
              altitudeAngle: 1.1,
              azimuthAngle: 0.4,
              persistentDeviceId: 42
            });
            target.dispatchEvent(ev);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "5:2.5:3.5:0.9:0.2:30:-15:270:touch:true:1.1:0.4:42:true:true:false",
    )?;
    Ok(())
}

#[test]
fn pointer_event_constructor_rejects_non_object_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              new PointerEvent('pointermove', 123);
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
