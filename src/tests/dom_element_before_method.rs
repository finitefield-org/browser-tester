use super::*;

#[test]
fn element_before_inserting_element_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const container = document.createElement('div');
            const p = document.createElement('p');
            container.appendChild(p);
            const span = document.createElement('span');

            p.before(span);
            document.getElementById('result').textContent = container.outerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "<div><span></span><p></p></div>")?;
    Ok(())
}

#[test]
fn element_before_inserting_text_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const container = document.createElement('div');
            const p = document.createElement('p');
            container.appendChild(p);

            p.before('Text');
            document.getElementById('result').textContent = container.outerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "<div>Text<p></p></div>")?;
    Ok(())
}

#[test]
fn element_before_inserting_element_and_text_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const container = document.createElement('div');
            const p = document.createElement('p');
            container.appendChild(p);
            const span = document.createElement('span');

            p.before(span, 'Text');
            document.getElementById('result').textContent = container.outerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "<div><span></span>Text<p></p></div>")?;
    Ok(())
}

#[test]
fn element_before_returns_undefined_and_is_noop_for_detached_target() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const p = document.createElement('p');
            const span = document.createElement('span');

            const returned = p.before(span, 'Text');
            document.getElementById('result').textContent = [
              returned === undefined,
              p.outerHTML,
              span.parentNode === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:<p></p>:true")?;
    Ok(())
}
