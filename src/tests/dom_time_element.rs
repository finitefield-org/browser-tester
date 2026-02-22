use super::*;

#[test]
fn time_implicit_role_and_datetime_reflection_work() -> Result<()> {
    let html = r#"
        <p>
          The concert starts at
          <time id='start' datetime='2018-07-07T20:00:00'>20:00</time>
          and lasts for
          <time id='duration' datetime='PT2H30M'>2h 30m</time>.
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const start = document.getElementById('start');
            const duration = document.getElementById('duration');

            const initial =
              start.role + ':' +
              start.dateTime + ':' +
              start.getAttribute('datetime') + ':' +
              start.textContent.trim() + ':' +
              duration.dateTime + ':' +
              duration.textContent.trim();

            start.dateTime = '2018-07-07T21:15:00';
            duration.setAttribute('datetime', 'PT3H');
            const updated =
              start.dateTime + ':' +
              start.getAttribute('datetime') + ':' +
              duration.dateTime + ':' +
              duration.getAttribute('datetime');

            start.removeAttribute('datetime');
            const removed = start.dateTime + ':' + (start.getAttribute('datetime') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + removed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "time:2018-07-07T20:00:00:2018-07-07T20:00:00:20:00:PT2H30M:2h 30m|2018-07-07T21:15:00:2018-07-07T21:15:00:PT3H:PT3H|:true",
    )?;
    Ok(())
}

#[test]
fn time_role_override_and_datetime_roundtrip_work() -> Result<()> {
    let html = r#"
        <p>The event date is <time id='event'>July 7</time>.</p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const eventTime = document.getElementById('event');
            const initial =
              eventTime.role + ':' +
              eventTime.dateTime + ':' +
              (eventTime.getAttribute('datetime') === null);

            eventTime.dateTime = '2018-07-07';
            const assignedDateTime =
              eventTime.dateTime + ':' + eventTime.getAttribute('datetime');

            eventTime.role = 'note';
            const assignedRole = eventTime.role + ':' + eventTime.getAttribute('role');
            eventTime.removeAttribute('role');
            const restoredRole = eventTime.role + ':' + (eventTime.getAttribute('role') === null);

            eventTime.removeAttribute('datetime');
            const removedDateTime =
              eventTime.dateTime + ':' + (eventTime.getAttribute('datetime') === null);

            document.getElementById('result').textContent =
              initial + '|' + assignedDateTime + '|' + assignedRole + '|' + restoredRole + '|' + removedDateTime + '|' + eventTime.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "time::true|2018-07-07:2018-07-07|note:note|time:true|:true|TIME",
    )?;
    Ok(())
}
