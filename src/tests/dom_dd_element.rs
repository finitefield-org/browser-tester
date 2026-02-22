use super::*;

#[test]
fn dd_works_as_description_details_in_dl_and_keeps_plain_semantics() -> Result<()> {
    let html = r#"
        <p>Cryptids of Cornwall:</p>
        <dl id='cryptids'>
          <dt>Beast of Bodmin</dt>
          <dd id='bodmin-desc'>A large feline inhabiting Bodmin Moor.</dd>
          <dt>Morgawr</dt>
          <dd>A sea serpent.</dd>
          <dt>Owlman</dt>
          <dd>A giant owl-like creature.</dd>
        </dl>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const items = document.querySelectorAll('#cryptids > dd');
            const bodmin = document.getElementById('bodmin-desc');
            document.getElementById('result').textContent =
              items.length + ':' +
              bodmin.role + ':' +
              bodmin.tagName + ':' +
              items[0].textContent.trim() + '|' +
              items[1].textContent.trim() + '|' +
              items[2].textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "3::DD:A large feline inhabiting Bodmin Moor.|A sea serpent.|A giant owl-like creature.",
    )?;
    Ok(())
}

#[test]
fn dd_optional_end_tag_parses_with_following_dt_or_dd_and_role_roundtrip_works() -> Result<()> {
    let html = r#"
        <dl id='cryptids'>
          <dt>Beast of Bodmin
          <dd id='first-desc'>A large feline inhabiting Bodmin Moor.
          <dt>Morgawr
          <dd>A sea serpent.
          <dt>Owlman
          <dd>A giant owl-like creature.
        </dl>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const dts = document.querySelectorAll('#cryptids > dt');
            const dds = document.querySelectorAll('#cryptids > dd');
            const first = document.getElementById('first-desc');

            const counts = dts.length + ':' + dds.length;
            const pairs =
              dts[0].textContent.trim() + '=' + dds[0].textContent.trim() + '|' +
              dts[1].textContent.trim() + '=' + dds[1].textContent.trim() + '|' +
              dts[2].textContent.trim() + '=' + dds[2].textContent.trim();

            const initialRole = first.role;
            first.role = 'note';
            const assigned = first.role + ':' + first.getAttribute('role');
            first.removeAttribute('role');
            const restored = first.role + ':' + (first.getAttribute('role') === null);

            document.getElementById('result').textContent =
              counts + '|' + pairs + '|' + initialRole + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "3:3|Beast of Bodmin=A large feline inhabiting Bodmin Moor.|Morgawr=A sea serpent.|Owlman=A giant owl-like creature.||note:note|:true",
    )?;
    Ok(())
}
