use super::*;

#[test]
fn dfn_implicit_term_role_and_definition_link_metadata_work() -> Result<()> {
    let html = r#"
        <p>
          The <strong>HTML Definition element
          (<dfn id='definition-dfn' title='HTML Definition element'>&lt;dfn&gt;</dfn>)</strong>
          is used to indicate the term being defined.
        </p>
        <p>
          We use <code><a id='def-link' href='#definition-dfn'>&lt;dfn&gt;</a></code>.
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const dfn = document.getElementById('definition-dfn');
            const link = document.getElementById('def-link');
            document.getElementById('result').textContent =
              dfn.role + ':' +
              dfn.title + ':' +
              dfn.textContent + ':' +
              link.getAttribute('href') + ':' +
              link.hash + ':' +
              dfn.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "term:HTML Definition element:<dfn>:#definition-dfn:#definition-dfn:DFN",
    )?;
    Ok(())
}

#[test]
fn dfn_role_override_restore_and_abbr_definition_usage_work() -> Result<()> {
    let html = r#"
        <p>
          The <dfn id='hst'><abbr id='hst-abbr' title='Hubble Space Telescope'>HST</abbr></dfn>
          is among the most productive scientific instruments ever constructed.
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const hst = document.getElementById('hst');
            const abbr = document.getElementById('hst-abbr');
            const initial = hst.role + ':' + hst.textContent + ':' + abbr.title;

            hst.role = 'note';
            const assigned = hst.role + ':' + hst.getAttribute('role');

            hst.removeAttribute('role');
            const restored = hst.role + ':' + (hst.getAttribute('role') === null);

            hst.title = 'Hubble Space Telescope';
            const titleRoundtrip = hst.title + ':' + hst.getAttribute('title');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + titleRoundtrip;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "term:HST:Hubble Space Telescope|note:note|term:true|Hubble Space Telescope:Hubble Space Telescope",
    )?;
    Ok(())
}
