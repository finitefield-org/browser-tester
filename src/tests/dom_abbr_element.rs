use super::*;

#[test]
fn abbr_title_property_reflects_title_attribute_and_does_not_mutate_document_title() -> Result<()> {
    let html = r#"
        <abbr id='api' title='Application Programming Interface'>API</abbr>
        <abbr id='html'>HTML</abbr>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.title = 'Original';
          document.getElementById('run').addEventListener('click', () => {
            const api = document.getElementById('api');
            const html = document.getElementById('html');
            const before = api.title + ':' + api.getAttribute('title');

            html.title = 'HyperText Markup Language';
            api.title = 'API Expanded';

            document.getElementById('result').textContent =
              before + '|' +
              html.title + ':' + html.getAttribute('title') + '|' +
              api.title + ':' + api.getAttribute('title') + '|' +
              document.title;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "Application Programming Interface:Application Programming Interface|HyperText Markup Language:HyperText Markup Language|API Expanded:API Expanded|Original",
    )?;
    Ok(())
}

#[test]
fn abbr_works_as_phrasing_content_and_can_be_used_with_dfn() -> Result<()> {
    let html = r#"
        <p>
          JavaScript Object Notation (<abbr id='json'>JSON</abbr>) is a lightweight format.
        </p>
        <p>
          <dfn id='html-def'><abbr title='HyperText Markup Language'>HTML</abbr></dfn> is
          a markup language.
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const abbrs = document.querySelectorAll('abbr');
            const first = abbrs[0].textContent;
            const second = abbrs[1].textContent;
            const dfnText = document.getElementById('html-def').textContent;
            const jsonHasTitle = document.getElementById('json').getAttribute('title') === null
              ? 'none'
              : 'has';
            document.getElementById('result').textContent =
              abbrs.length + ':' + first + ':' + second + ':' + dfnText + ':' + jsonHasTitle;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:JSON:HTML:HTML:none")?;
    Ok(())
}
