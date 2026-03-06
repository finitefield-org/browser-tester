use super::*;

#[test]
fn html_anchor_element_global_and_instanceof_work() -> Result<()> {
    let html = r#"
        <a id='link' href='/docs/page'>link</a>
        <div id='box'>box</div>
        <p id='result'></p>
        <script>
          const link = document.getElementById('link');
          const box = document.getElementById('box');
          document.getElementById('result').textContent = [
            typeof HTMLAnchorElement,
            window.HTMLAnchorElement === HTMLAnchorElement,
            link instanceof HTMLAnchorElement,
            link instanceof HTMLElement,
            box instanceof HTMLAnchorElement
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "function:true:true:true:false")?;
    Ok(())
}

#[test]
fn anchor_name_property_reflects_attribute_and_to_string_returns_href() -> Result<()> {
    let html = r#"
        <a id='link' href='/docs/start?x=1#top'>hello</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://example.com/base/index.html';
            const link = document.getElementById('link');
            link.name = 'section-start';
            document.getElementById('result').textContent = [
              link.name,
              link.getAttribute('name'),
              document.getElementById('link').toString()
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "section-start:section-start:https://example.com/docs/start?x=1#top",
    )?;
    Ok(())
}
