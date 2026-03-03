use super::*;

#[test]
fn element_get_animations_returns_own_animations_and_supports_subtree_option() -> Result<()> {
    let html = r#"
        <div id='host'>
          <span id='child'></span>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const child = document.getElementById('child');

            const hostAnimation = host.animate({ opacity: [0, 1] }, { id: 'host' });
            const childAnimation = child.animate({ opacity: [1, 0] }, { id: 'child' });

            const own = host.getAnimations();
            const withSubtree = host.getAnimations({ subtree: true });

            document.getElementById('result').textContent = [
              own.length,
              withSubtree.length,
              own[0] === hostAnimation,
              withSubtree[0] === hostAnimation,
              withSubtree[1] === childAnimation,
              child.getAnimations().length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:2:true:true:true:1")?;
    Ok(())
}

#[test]
fn element_get_animations_non_object_options_default_subtree_to_false() -> Result<()> {
    let html = r#"
        <div id='host'>
          <span id='child'></span>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const child = document.getElementById('child');

            host.animate({ transform: ['scale(0)', 'scale(1)'] }, { id: 'host' });
            child.animate({ transform: ['scale(1)', 'scale(0)'] }, { id: 'child' });

            const boolOption = host.getAnimations(true);
            const stringOption = host.getAnimations('anything');

            document.getElementById('result').textContent = [
              boolOption.length,
              stringOption.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:1")?;
    Ok(())
}

#[test]
fn element_get_animations_rejects_more_than_one_argument() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            host.getAnimations({}, {});
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getAnimations supports zero or one options argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
