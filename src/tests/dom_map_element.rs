use super::*;

#[test]
fn map_name_usemap_and_role_roundtrip_work() -> Result<()> {
    let html = r#"
        <map id='infographic' name='infographic'>
          <area id='html-area' shape='poly' coords='0,0,30,0,30,30,0,30' href='/docs/html' alt='HTML'>
          <area id='css-area' shape='poly' coords='30,0,60,0,60,30,30,30' href='/docs/css' alt='CSS'>
        </map>
        <img id='diagram' usemap='#infographic' src='/assets/infographic.png' alt='MDN infographic'>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const map = document.getElementById('infographic');
            const diagram = document.getElementById('diagram');

            const initial =
              map.role + ':' +
              map.tagName + ':' +
              map.name + ':' +
              map.getAttribute('name') + ':' +
              map.querySelectorAll('area').length + ':' +
              diagram.getAttribute('usemap');

            map.name = 'primary';
            diagram.setAttribute('usemap', '#primary');
            const renamed =
              map.name + ':' +
              map.getAttribute('name') + ':' +
              diagram.getAttribute('usemap');

            map.role = 'none';
            const assigned = map.role + ':' + map.getAttribute('role');
            map.removeAttribute('role');
            const restored = map.role + ':' + (map.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + renamed + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":MAP:infographic:infographic:2:#infographic|primary:primary:#primary|none:none|:true",
    )?;
    Ok(())
}

#[test]
fn map_allows_flow_content_and_area_navigation_works() -> Result<()> {
    let html = r#"
        <map id='routes' name='routes'>
          <p id='note'>Choose route:</p>
          <area id='go' shape='rect' coords='0,0,10,10' href='/go' alt='Go'>
          <area id='stay' alt='Stay'>
        </map>
        <img id='poster' usemap='#routes' src='/parrots.jpg' alt='Two parrots'>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const map = document.getElementById('routes');
            const go = document.getElementById('go');
            const stay = document.getElementById('stay');

            const before =
              map.querySelector('#note').textContent.trim() + ':' +
              go.href + ':' +
              stay.role + ':' +
              document.links.length;

            go.href = '/next';
            const after =
              go.href + ':' +
              go.getAttribute('href') + ':' +
              map.querySelectorAll('area').length;

            document.getElementById('result').textContent = before + '|' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "Choose route::https://app.local/go::1|https://app.local/next:/next:2",
    )?;

    h.click("#go")?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start/index.html".to_string(),
            to: "https://app.local/next".to_string(),
        }]
    );
    Ok(())
}
