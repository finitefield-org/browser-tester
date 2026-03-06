use super::*;

#[test]
fn element_role_returns_only_explicitly_set_role_or_null() -> Result<()> {
    let html = r#"
        <ul id='list'><li>One</li></ul>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('list');
            const initialIsNull = list.role === null;

            list.setAttribute('role', 'treegrid');
            const assigned = list.role + ':' + list.getAttribute('role');

            list.removeAttribute('role');
            const restoredIsNull = list.role === null;

            document.getElementById('result').textContent =
              initialIsNull + ':' + assigned + ':' + restoredIsNull;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:treegrid:treegrid:false")?;
    Ok(())
}

#[test]
fn element_role_assignment_reflects_role_attribute() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.role = 'region';
            const first = box.role + ':' + box.getAttribute('role');

            box.role = 'status';
            const second = box.role + ':' + box.getAttribute('role');

            document.getElementById('result').textContent = first + '|' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "region:region|status:status")?;
    Ok(())
}

#[test]
fn element_role_mdn_images_example_sets_presentation_on_missing_or_empty_alt() -> Result<()> {
    let html = r#"
        <img id='a' alt='logo' />
        <img id='b' alt='' />
        <img id='c' />
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const images = document.querySelectorAll('img');
            images.forEach((image) => {
              if (!image.getAttribute('alt')) {
                image.role = 'presentation';
              }
            });

            document.getElementById('result').textContent = [
              document.getElementById('a').role === null,
              document.getElementById('b').role,
              document.getElementById('c').role
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:presentation:presentation")?;
    Ok(())
}
