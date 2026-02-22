use super::*;

#[test]
fn blockquote_implicit_role_and_cite_reflection_work() -> Result<()> {
    let html = r#"
        <div>
          <blockquote id='quote' cite='https://www.huxley.net/bnw/four.html'>
            <p>
              Words can be like X-rays, if you use them properly—they'll go through anything.
            </p>
          </blockquote>
          <p>—Aldous Huxley, <cite id='book'>Brave New World</cite></p>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const quote = document.getElementById('quote');
            document.getElementById('result').textContent =
              quote.role + ':' +
              quote.cite + ':' +
              quote.getAttribute('cite') + ':' +
              document.querySelectorAll('blockquote').length + ':' +
              document.querySelectorAll('blockquote cite').length + ':' +
              document.getElementById('book').tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "blockquote:https://www.huxley.net/bnw/four.html:https://www.huxley.net/bnw/four.html:1:0:CITE",
    )?;
    Ok(())
}

#[test]
fn blockquote_cite_and_role_assignments_override_and_restore() -> Result<()> {
    let html = r#"
        <blockquote id='rfc' cite='https://datatracker.ietf.org/doc/html/rfc1149'>
          <p>Avian carriers can provide high delay service.</p>
        </blockquote>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const rfc = document.getElementById('rfc');

            const initialCite = rfc.cite;
            rfc.cite = 'https://example.com/quote-source';
            const assignedCite = rfc.cite + ':' + rfc.getAttribute('cite');
            rfc.removeAttribute('cite');
            const restoredCite = rfc.cite + ':' + (rfc.getAttribute('cite') === null);

            const initialRole = rfc.role;
            rfc.role = 'note';
            const assignedRole = rfc.role + ':' + rfc.getAttribute('role');
            rfc.removeAttribute('role');
            const restoredRole = rfc.role + ':' + (rfc.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initialCite + '|' +
              assignedCite + '|' +
              restoredCite + '|' +
              initialRole + '|' +
              assignedRole + '|' +
              restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://datatracker.ietf.org/doc/html/rfc1149|https://example.com/quote-source:https://example.com/quote-source|:true|blockquote|note:note|blockquote:true",
    )?;
    Ok(())
}
