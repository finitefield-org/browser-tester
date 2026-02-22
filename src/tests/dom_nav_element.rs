use super::*;

#[test]
fn nav_has_implicit_navigation_role_and_link_content_work() -> Result<()> {
    let html = r#"
        <nav id='crumbs' class='crumbs'>
          <ol>
            <li class='crumb'><a id='bikes' href='/bikes'>Bikes</a></li>
            <li class='crumb'><a id='bmx' href='/bmx'>BMX</a></li>
            <li class='crumb'>Jump Bike 3000</li>
          </ol>
        </nav>

        <h1>Jump Bike 3000</h1>
        <p>This BMX bike is a solid step into the pro world.</p>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const crumbs = document.getElementById('crumbs');
            document.getElementById('result').textContent =
              crumbs.role + ':' +
              crumbs.tagName + ':' +
              crumbs.querySelectorAll('a').length + ':' +
              crumbs.querySelectorAll('li').length + ':' +
              crumbs.querySelector('.crumb:last-child').textContent.trim() + ':' +
              document.getElementById('bikes').href + ':' +
              document.links.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/shop/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "navigation:NAV:2:3:Jump Bike 3000:https://app.local/bikes:2",
    )?;
    Ok(())
}

#[test]
fn nav_role_override_and_scoped_header_footer_roles_work() -> Result<()> {
    let html = r#"
        <nav id='primary-nav' aria-labelledby='primary-navigation'>
          <header id='nav-header'>
            <h2 id='primary-navigation'>Primary navigation</h2>
          </header>
          <ul>
            <li><a href='/home'>Home</a></li>
            <li><a href='/about'>About</a></li>
          </ul>
          <footer id='nav-footer'>
            <small>Quick links</small>
          </footer>
        </nav>

        <nav id='footer-nav' aria-labelledby='footer-navigation'>
          <h2 id='footer-navigation'>Footer navigation</h2>
          <a href='/privacy'>Privacy</a>
        </nav>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const primaryNav = document.getElementById('primary-nav');
            const footerNav = document.getElementById('footer-nav');
            const navHeader = document.getElementById('nav-header');
            const navFooter = document.getElementById('nav-footer');

            const initial =
              primaryNav.role + ':' +
              footerNav.role + ':' +
              navHeader.role + ':' +
              navFooter.role + ':' +
              document.getElementById(primaryNav.getAttribute('aria-labelledby')).textContent.trim() + ':' +
              document.getElementById(footerNav.getAttribute('aria-labelledby')).textContent.trim();

            primaryNav.role = 'none';
            const assigned = primaryNav.role + ':' + primaryNav.getAttribute('role');

            primaryNav.removeAttribute('role');
            const restored = primaryNav.role + ':' + (primaryNav.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "navigation:navigation:generic:generic:Primary navigation:Footer navigation|none:none|navigation:true",
    )?;
    Ok(())
}
