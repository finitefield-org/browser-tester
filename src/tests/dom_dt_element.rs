use super::*;

#[test]
fn dt_supports_multiple_terms_for_single_description() -> Result<()> {
    let html = r#"
        <p>Please use the following paint colors for the new house:</p>
        <dl id='paints'>
          <dt>Denim (semigloss finish)</dt>
          <dd>Ceiling</dd>

          <dt>Denim (eggshell finish)</dt>
          <dt>Evening Sky (eggshell finish)</dt>
          <dd>Layered on the walls</dd>
        </dl>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const terms = document.querySelectorAll('#paints > dt');
            const descriptions = document.querySelectorAll('#paints > dd');
            const first = terms[0];
            document.getElementById('result').textContent =
              terms.length + ':' +
              descriptions.length + ':' +
              first.role + ':' +
              first.tagName + '|' +
              terms[0].textContent.trim() + '=' + descriptions[0].textContent.trim() + '|' +
              terms[1].textContent.trim() + '+' + terms[2].textContent.trim() + '=' +
              descriptions[1].textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "3:2::DT|Denim (semigloss finish)=Ceiling|Denim (eggshell finish)+Evening Sky (eggshell finish)=Layered on the walls",
    )?;
    Ok(())
}

#[test]
fn dt_optional_end_tag_and_role_roundtrip_work() -> Result<()> {
    let html = r#"
        <dl id='meta'>
          <dt id='name'>Name
          <dd>Godzilla</dd>
          <dt id='born'>Born
          <dt id='birth-year'>Birth year
          <dd id='year'>1952</dd>
        </dl>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const terms = document.querySelectorAll('#meta > dt');
            const descriptions = document.querySelectorAll('#meta > dd');
            const born = document.getElementById('born');
            const birthYear = document.getElementById('birth-year');

            const pairs =
              terms[0].textContent.trim() + '=' + descriptions[0].textContent.trim() + '|' +
              terms[1].textContent.trim() + '+' + terms[2].textContent.trim() + '=' +
              descriptions[1].textContent.trim();

            const initial = born.role + ':' + birthYear.role;
            born.role = 'listitem';
            const assigned = born.role + ':' + born.getAttribute('role');
            born.removeAttribute('role');
            const restored = born.role + ':' + (born.getAttribute('role') === null);

            document.getElementById('result').textContent =
              terms.length + ':' + descriptions.length + '|' + pairs + '|' +
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "3:2|Name=Godzilla|Born+Birth year=1952|:|listitem:listitem|:true",
    )?;
    Ok(())
}
