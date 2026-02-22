use super::*;

#[test]
fn span_has_no_implicit_role_and_groups_inline_content() -> Result<()> {
    let html = r#"
        <p id='recipe'>
          Add the <span id='basil' class='ingredient' lang='it'>basil</span>,
          <span class='ingredient'>pine nuts</span> and
          <span class='ingredient'>garlic</span>.
        </p>
        <ul>
          <li>
            <span id='portfolio-wrap'>
              <a id='portfolio' href='portfolio.html' target='_blank'>See my portfolio</a>
            </span>
          </li>
        </ul>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const basil = document.getElementById('basil');
            const wrap = document.getElementById('portfolio-wrap');
            const portfolio = document.getElementById('portfolio');

            document.getElementById('result').textContent =
              basil.role + ':' +
              basil.tagName + ':' +
              basil.className + ':' +
              basil.getAttribute('lang') + ':' +
              document.querySelectorAll('span.ingredient').length + ':' +
              wrap.role + ':' +
              portfolio.href + ':' +
              wrap.textContent.replace(/\s+/g, ' ').trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/docs/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":SPAN:ingredient:it:3::https://app.local/docs/portfolio.html:See my portfolio",
    )?;
    Ok(())
}

#[test]
fn span_role_override_and_global_attrs_roundtrip_work() -> Result<()> {
    let html = r#"
        <p>
          <span id='label' title='Flavor note'>Extra virgin olive oil</span>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const label = document.getElementById('label');
            const initial =
              label.role + ':' +
              label.getAttribute('title') + ':' +
              label.textContent.trim();

            label.role = 'note';
            label.className = 'small-text';
            label.lang = 'en';
            const assigned =
              label.role + ':' +
              label.getAttribute('role') + ':' +
              label.className + ':' +
              label.getAttribute('lang');

            label.removeAttribute('role');
            const restored = label.role + ':' + (label.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":Flavor note:Extra virgin olive oil|note:note:small-text:en|:true",
    )?;
    Ok(())
}
