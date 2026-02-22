use super::*;

#[test]
fn dl_supports_term_description_groups_and_div_wrappers() -> Result<()> {
    let html = r#"
        <p>Monster metadata:</p>
        <dl id='monster-meta'>
          <div class='group'>
            <dt>Name</dt>
            <dd>Godzilla</dd>
          </div>
          <div class='group'>
            <dt>Born</dt>
            <dd>1952</dd>
          </div>
          <div class='group'>
            <dt>Color</dt>
            <dd>Green</dd>
          </div>
        </dl>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('monster-meta');
            const groups = list.querySelectorAll('div.group');
            const terms = list.querySelectorAll('dt');
            const descriptions = list.querySelectorAll('dd');

            document.getElementById('result').textContent =
              list.tagName + ':' +
              list.role + ':' +
              groups.length + ':' +
              terms.length + ':' +
              descriptions.length + '|' +
              terms[0].textContent.trim() + '=' + descriptions[0].textContent.trim() + '|' +
              terms[1].textContent.trim() + '=' + descriptions[1].textContent.trim() + '|' +
              terms[2].textContent.trim() + '=' + descriptions[2].textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "DL::3:3:3|Name=Godzilla|Born=1952|Color=Green")?;
    Ok(())
}

#[test]
fn dl_role_override_and_compact_attribute_roundtrip_work() -> Result<()> {
    let html = r#"
        <dl id='cryptids' compact>
          <dt>Owlman</dt>
          <dd id='desc'>A giant owl-like creature.</dd>
        </dl>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('cryptids');
            const initial = list.hasAttribute('compact') + ':' + list.role;

            list.role = 'list';
            list.setAttribute('compact', 'compact');
            const assigned =
              list.role + ':' + list.getAttribute('role') + ':' + list.getAttribute('compact');

            list.removeAttribute('role');
            list.removeAttribute('compact');
            const restored =
              list.role + ':' +
              (list.getAttribute('role') === null) + ':' +
              (list.getAttribute('compact') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' +
              document.getElementById('desc').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:|list:list:compact|:true:true|A giant owl-like creature.",
    )?;
    Ok(())
}
