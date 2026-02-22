use super::*;

#[test]
fn mark_highlight_usage_and_match_queries_work() -> Result<()> {
    let html = r#"
        <p>Search results for "salamander":</p>
        <hr>
        <p id='first'>
          Several species of <mark class='match' id='m1'>salamander</mark> inhabit the temperate rainforest.
        </p>
        <p id='second'>
          Most <mark class='match' id='m2'>salamander</mark>s are nocturnal.
        </p>
        <blockquote id='quote'>
          During the battle, <mark id='focus'>Rebel spies managed to steal secret plans</mark>.
        </blockquote>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const m1 = document.getElementById('m1');
            const m2 = document.getElementById('m2');
            const focus = document.getElementById('focus');

            document.getElementById('result').textContent =
              m1.role + ':' +
              m2.role + ':' +
              focus.role + ':' +
              document.querySelectorAll('mark.match').length + ':' +
              document.querySelectorAll('#quote mark').length + '|' +
              m1.textContent.trim() + ':' +
              m2.textContent.trim() + ':' +
              focus.textContent.includes('secret plans') + ':' +
              m1.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":::2:1|salamander:salamander:true:MARK")?;
    Ok(())
}

#[test]
fn mark_role_attribute_overrides_and_remove_restores_empty_implicit_role() -> Result<()> {
    let html = r#"
        <p>
          It is a dark time for the Rebellion. The
          <mark id='target' class='match'>Imperial</mark> troops advanced.
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const initial =
              target.role + ':' +
              target.className + ':' +
              target.textContent.trim();

            target.role = 'note';
            const assigned = target.role + ':' + target.getAttribute('role');

            target.removeAttribute('role');
            const restored = target.role + ':' + (target.getAttribute('role') === null);

            target.className = 'hit';
            const classChanged = target.className + ':' + target.getAttribute('class');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + classChanged;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":match:Imperial|note:note|:true|hit:hit")?;
    Ok(())
}
