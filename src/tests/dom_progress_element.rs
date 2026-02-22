use super::*;

#[test]
fn progress_has_implicit_role_and_value_max_roundtrip_work() -> Result<()> {
    let html = r#"
        <label for='file'>File progress:</label>
        <progress id='file' max='100' value='70'>70%</progress>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const bar = document.getElementById('file');
            const initial =
              bar.role + ':' +
              bar.tagName + ':' +
              bar.value + ':' +
              bar.getAttribute('value') + ':' +
              bar.getAttribute('max') + ':' +
              document.querySelector('label[for="file"]').textContent.trim() + ':' +
              bar.indeterminate;

            bar.setAttribute('value', '85');
            bar.setAttribute('max', '200');
            const assigned =
              bar.value + ':' +
              bar.getAttribute('value') + ':' +
              bar.getAttribute('max') + ':' +
              bar.indeterminate;

            bar.removeAttribute('value');
            const removed =
              (bar.getAttribute('value') === null) + ':' +
              bar.value + ':' +
              bar.indeterminate;

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + removed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "progressbar:PROGRESS:70:70:100:File progress::false|85:85:200:false|true::true",
    )?;
    Ok(())
}

#[test]
fn progress_value_property_reflects_attribute_and_role_override_roundtrips() -> Result<()> {
    let html = r#"
        <progress id='job' max='1'>Loading...</progress>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const job = document.getElementById('job');
            const initial =
              job.role + ':' +
              job.textContent.trim() + ':' +
              (job.getAttribute('value') === null) + ':' +
              job.indeterminate;

            job.value = '0.4';
            const byProp =
              job.value + ':' +
              job.getAttribute('value') + ':' +
              job.indeterminate;

            job.removeAttribute('value');
            const removed =
              (job.getAttribute('value') === null) + ':' +
              job.indeterminate + ':' +
              job.value;

            job.role = 'status';
            const assigned = job.role + ':' + job.getAttribute('role');
            job.removeAttribute('role');
            const restored = job.role + ':' + (job.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + byProp + '|' + removed + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "progressbar:Loading...:true:true|0.4:0.4:false|true:true:|status:status|progressbar:true",
    )?;
    Ok(())
}
