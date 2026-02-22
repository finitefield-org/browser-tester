use super::*;

#[test]
fn q_has_implicit_generic_role_and_cite_property_reflects_attribute() -> Result<()> {
    let html = r#"
        <p id='line'>
          HAL answered:
          <q id='quote' cite='https://example.com/quotes/hal'>
            I'm sorry, Dave. I'm afraid I can't do that.
          </q>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const quote = document.getElementById('quote');
            const initial =
              quote.role + ':' +
              quote.tagName + ':' +
              quote.cite + ':' +
              quote.getAttribute('cite') + ':' +
              quote.textContent.trim().indexOf("I'm sorry, Dave.") + ':' +
              document.querySelectorAll('p q').length;

            quote.cite = 'https://example.com/quotes/hal-2';
            const assigned = quote.cite + ':' + quote.getAttribute('cite');

            quote.removeAttribute('cite');
            const removed = quote.cite + ':' + (quote.getAttribute('cite') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + removed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic:Q:https://example.com/quotes/hal:https://example.com/quotes/hal:0:1|https://example.com/quotes/hal-2:https://example.com/quotes/hal-2|:true",
    )?;
    Ok(())
}

#[test]
fn q_role_override_roundtrips_and_supports_nested_phrasing_content() -> Result<()> {
    let html = r#"
        <p>
          <q id='short'>
            According to <em>spec guidance</em>, use <code>&lt;q&gt;</code> for short quotations.
          </q>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const short = document.getElementById('short');
            const initial =
              short.role + ':' +
              short.cite + ':' +
              short.querySelectorAll('em').length + ':' +
              short.querySelectorAll('code').length + ':' +
              short.textContent.trim().indexOf('According to');

            short.setAttribute('cite', 'https://example.com/quote-context');
            const citeAssigned = short.cite + ':' + short.getAttribute('cite');

            short.role = 'note';
            const roleAssigned = short.role + ':' + short.getAttribute('role');

            short.removeAttribute('role');
            const roleRestored = short.role + ':' + (short.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + citeAssigned + '|' + roleAssigned + '|' + roleRestored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic::1:1:0|https://example.com/quote-context:https://example.com/quote-context|note:note|generic:true",
    )?;
    Ok(())
}
