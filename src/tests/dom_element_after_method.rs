use super::*;

#[test]
fn element_after_inserting_element_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const container = document.createElement('div');
            const p = document.createElement('p');
            container.appendChild(p);
            const span = document.createElement('span');

            p.after(span);
            document.getElementById('result').textContent = container.outerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "<div><p></p><span></span></div>")?;
    Ok(())
}

#[test]
fn element_after_inserting_text_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const container = document.createElement('div');
            const p = document.createElement('p');
            container.appendChild(p);

            p.after('Text');
            document.getElementById('result').textContent = container.outerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "<div><p></p>Text</div>")?;
    Ok(())
}

#[test]
fn element_after_inserting_element_and_text_matches_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const container = document.createElement('div');
            const p = document.createElement('p');
            container.appendChild(p);
            const span = document.createElement('span');

            p.after(span, 'Text');
            document.getElementById('result').textContent = container.outerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "<div><p></p><span></span>Text</div>")?;
    Ok(())
}

#[test]
fn element_after_returns_undefined_and_is_noop_for_detached_target() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const p = document.createElement('p');
            const span = document.createElement('span');

            const returned = p.after(span, 'Text');
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
