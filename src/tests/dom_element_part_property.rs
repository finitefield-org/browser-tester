use super::*;

#[test]
fn element_part_assignment_updates_part_attribute_and_tokens() -> Result<()> {
    let html = r#"
        <div id='tabs'>
          <button id='t1' part='tab active'>One</button>
          <button id='t2' part='tab'>Two</button>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const tabs = [];
            const children = document.getElementById('tabs').children;

            for (const elem of children) {
              if (elem.getAttribute('part')) {
                tabs.push(elem);
              }
            }

            tabs.forEach((tab) => {
              tab.part = 'tab';
            });
            tabs[1].part = 'tab active';

            document.getElementById('result').textContent = [
              tabs[0].getAttribute('part'),
              tabs[1].getAttribute('part'),
              tabs[0].part.length,
              tabs[1].part.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "tab:tab active:1:2")?;
    Ok(())
}

#[test]
fn element_part_returns_empty_list_when_attribute_missing_or_empty() -> Result<()> {
    let html = r#"
        <div id='a'></div>
        <div id='b' part=''></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const a = document.getElementById('a');
            const b = document.getElementById('b');
            document.getElementById('result').textContent =
              a.part.length + ':' + b.part.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0:0")?;
    Ok(())
}

#[test]
fn element_part_assignment_via_object_member_path_forwards_to_attribute() -> Result<()> {
    let html = r#"
        <div id='chip'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const holder = { el: document.getElementById('chip') };
            holder.el.part = null;
            const value = holder.el.getAttribute('part');
            document.getElementById('result').textContent =
              value + ':' + holder.el.part.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "null:1")?;
    Ok(())
}
