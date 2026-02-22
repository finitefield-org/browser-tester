use super::*;

#[test]
fn query_selector_all_index_supports_expression() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const index = 1;
            const next = items[index + 1].textContent;
            document.getElementById('result').textContent = items[index].textContent + ':' + next;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B:C")?;
    Ok(())
}

#[test]
fn query_selector_all_list_index_after_reuse_works() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const picked = items[2];
            document.getElementById('result').textContent = picked.textContent + ':' + items.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "C:3")?;
    Ok(())
}

#[test]
fn get_elements_by_class_name_works() -> Result<()> {
    let html = r#"
        <ul>
          <li id='x' class='item target'>A</li>
          <li id='y' class='item'>B</li>
          <li id='z' class='target'>C</li>
          <li id='w' class='item target'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.getElementsByClassName('item target');
            document.getElementById('result').textContent = items.length + ':' + items[0].id + ':' + items[1].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:x:w")?;
    Ok(())
}

#[test]
fn get_elements_by_tag_name_works() -> Result<()> {
    let html = r#"
        <ul>
          <li id='a'>A</li>
          <li id='b'>B</li>
        </ul>
        <section id='s'>
          <li id='c'>C</li>
        </section>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.getElementsByTagName('li');
            document.getElementById('result').textContent = items.length + ':' + items[2].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:c")?;
    Ok(())
}

#[test]
fn get_elements_by_name_works() -> Result<()> {
    let html = r#"
        <input id='a' name='target' value='one'>
        <input id='b' name='other' value='other'>
        <input id='c' name='target' value='two'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fields = document.getElementsByName('target');
            document.getElementById('result').textContent = fields.length + ':' + fields[1].value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:two")?;
    Ok(())
}

#[test]
fn class_list_add_remove_multiple_arguments_work() -> Result<()> {
    let html = r#"
        <div id='box' class='base'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.classList.add('alpha', 'beta', 'gamma');
            box.classList.remove('base', 'gamma');
            document.getElementById('result').textContent = box.className;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "alpha beta")?;
    Ok(())
}

#[test]
fn member_chain_dom_targets_support_class_list_listener_and_open_property() -> Result<()> {
    let html = r#"
        <div id='dialog' class='panel hidden'></div>
        <button id='open-tool'>open</button>
        <details id='settings' open></details>
        <p id='result'></p>
        <script>
          const el = {
            dialog: document.getElementById('dialog'),
            openToolBtn: document.getElementById('open-tool'),
            settingsDetails: document.getElementById('settings'),
          };

          el.dialog.classList.remove('hidden');
          el.openToolBtn.addEventListener('click', () => {
            document.getElementById('result').textContent =
              el.dialog.className + ':' + (el.settingsDetails.open ? 'open' : 'closed');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#open-tool")?;
    h.assert_text("#result", "panel:open")?;
    Ok(())
}

#[test]
fn document_core_properties_and_collections_work() -> Result<()> {
    let html = r#"
        <html id='doc'>
          <head id='head'>
            <title>Initial</title>
          </head>
          <body id='body'>
            <form id='f'><input id='name'></form>
            <img id='logo' src='logo.png'>
            <a id='link' href='/x'>x</a>
            <button id='btn'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('btn').addEventListener('click', () => {
                const kids = document.children;
                const first = document.firstElementChild;
                const last = document.lastElementChild;
                const activeBeforeNode = document.activeElement;
                const activeBefore = activeBeforeNode ? activeBeforeNode.id : 'none';
                document.getElementById('name').focus();
                const activeAfterNode = document.activeElement;
                const activeAfter = activeAfterNode ? activeAfterNode.id : 'none';

                document.getElementById('result').textContent =
                  document.title + ':' +
                  document.characterSet + ':' +
                  document.compatMode + ':' +
                  document.contentType + ':' +
                  document.readyState + ':' +
                  document.referrer + ':' +
                  document.URL + ':' +
                  document.documentURI + ':' +
                  document.location + ':' +
                  document.location.href + ':' +
                  document.visibilityState + ':' +
                  document.hidden + ':' +
                  document.body.id + ':' +
                  document.head.id + ':' +
                  document.documentElement.id + ':' +
                  document.childElementCount + ':' +
                  kids.length + ':' +
                  first.id + ':' +
                  last.id + ':' +
                  document.forms.length + ':' +
                  document.images.length + ':' +
                  document.links.length + ':' +
                  document.scripts.length + ':' +
                  activeBefore + ':' +
                  activeAfter + ':' +
                  (document.defaultView ? 'yes' : 'no');
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Initial:UTF-8:CSS1Compat:text/html:complete::about:blank:about:blank:about:blank:about:blank:visible:false:body:head:doc:1:1:doc:doc:1:1:1:1:none:name:yes",
    )?;
    Ok(())
}

#[test]
fn document_title_assignment_and_body_chain_target_work() -> Result<()> {
    let html = r#"
        <html id='doc'>
          <head id='head'></head>
          <body id='body'>
            <button id='btn'>run</button>
            <p id='result'></p>
            <script>
              document.body.classList.add('ready');
              document.body.addEventListener('click', () => {});

              document.getElementById('btn').addEventListener('click', () => {
                document.title = 'Updated';
                const first = document.firstElementChild;
                const last = document.lastElementChild;
                document.getElementById('result').textContent =
                  document.title + ':' +
                  document.head.id + ':' +
                  document.documentElement.id + ':' +
                  document.body.className + ':' +
                  first.id + ':' +
                  last.id;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Updated:head:doc:ready:doc:doc")?;
    Ok(())
}

#[test]
fn location_properties_and_setters_work_from_location_document_and_window() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://developer.mozilla.org:8080/en-US/search?q=URL#search-results-close-container';
            document.location.protocol = 'http:';
            window.location.hostname = 'example.com';
            location.port = '9090';
            location.pathname = 'docs';
            location.search = 'k=v';
            location.hash = 'anchor';

            document.getElementById('result').textContent =
              location.href + '|' +
              location.protocol + '|' +
              location.host + '|' +
              location.hostname + '|' +
              location.port + '|' +
              location.pathname + '|' +
              location.search + '|' +
              location.hash + '|' +
              location.origin + '|' +
              document.location.toString() + '|' +
              window.location.toString() + '|' +
              location.ancestorOrigins.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://example.com:9090/docs?k=v#anchor|http:|example.com:9090|example.com|9090|/docs|?k=v|#anchor|http://example.com:9090|http://example.com:9090/docs?k=v#anchor|http://example.com:9090/docs?k=v#anchor|0",
    )?;
    Ok(())
}

#[test]
fn location_assign_replace_reload_and_navigation_logs_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.assign('https://app.local/a?x=1#h');
            location.replace('/b');
            location.reload();
            document.getElementById('result').textContent = location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "https://app.local/b")?;
    assert_eq!(h.location_reload_count(), 1);

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "about:blank".to_string(),
                to: "https://app.local/a?x=1#h".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Replace,
                from: "https://app.local/a?x=1#h".to_string(),
                to: "https://app.local/b".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Reload,
                from: "https://app.local/b".to_string(),
                to: "https://app.local/b".to_string(),
            },
        ]
    );
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}

#[test]
fn location_mock_pages_load_on_navigation_and_reload() -> Result<()> {
    let html = r#"
        <button id='go'>go</button>
        <script>
          document.getElementById('go').addEventListener('click', () => {
            location.assign('https://app.local/next');
          });
        </script>
        "#;

    let first_mock = r#"
        <button id='reload'>reload</button>
        <p id='marker'>first</p>
        <script>
          document.getElementById('reload').addEventListener('click', () => {
            location.reload();
          });
        </script>
        "#;
    let second_mock = "<p id='marker'>second</p>";

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/next", first_mock);
    h.click("#go")?;
    h.assert_text("#marker", "first")?;

    h.set_location_mock_page("https://app.local/next", second_mock);
    h.click("#reload")?;
    h.assert_text("#marker", "second")?;
    assert_eq!(h.location_reload_count(), 1);

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "about:blank".to_string(),
                to: "https://app.local/next".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Reload,
                from: "https://app.local/next".to_string(),
                to: "https://app.local/next".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn hash_only_location_navigation_does_not_trigger_mock_page_swap() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'>alive</p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://app.local/path';
            location.hash = 'frag';
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/path#frag", "<p id='result'>swapped</p>");
    h.click("#run")?;
    h.assert_text("#result", "alive:https://app.local/path#frag")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "about:blank".to_string(),
                to: "https://app.local/path".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "https://app.local/path".to_string(),
                to: "https://app.local/path#frag".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn anchor_properties_and_to_string_work() -> Result<()> {
    let html = r#"
        <a id='link' href='/docs/page?x=1#intro'>hello</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://example.com/base/index.html?doc=1#docfrag';
            document.getElementById('link').download = 'report.txt';
            document.getElementById('link').hreflang = 'ja';
            document.getElementById('link').ping = 'https://p1.test https://p2.test';
            document.getElementById('link').referrerPolicy = 'no-referrer';
            document.getElementById('link').rel = 'noopener noreferrer';
            document.getElementById('link').target = '_blank';
            document.getElementById('link').type = 'text/plain';
            document.getElementById('link').attributionSrc = 'https://attr.test/src';
            document.getElementById('link').interestForElement = 'panel';
            document.getElementById('link').charset = 'utf-8';
            document.getElementById('link').coords = '0,0,10,10';
            document.getElementById('link').rev = 'prev';
            document.getElementById('link').shape = 'rect';

            document.getElementById('result').textContent =
              document.getElementById('link').href + '|' +
              document.getElementById('link').protocol + '|' +
              document.getElementById('link').host + '|' +
              document.getElementById('link').hostname + '|' +
              document.getElementById('link').port + '|' +
              document.getElementById('link').pathname + '|' +
              document.getElementById('link').search + '|' +
              document.getElementById('link').hash + '|' +
              document.getElementById('link').origin + '|' +
              document.getElementById('link').download + '|' +
              document.getElementById('link').hreflang + '|' +
              document.getElementById('link').ping + '|' +
              document.getElementById('link').referrerPolicy + '|' +
              document.getElementById('link').rel + '|' +
              document.getElementById('link').relList.length + '|' +
              document.getElementById('link').target + '|' +
              document.getElementById('link').type + '|' +
              document.getElementById('link').attributionSrc + '|' +
              document.getElementById('link').interestForElement + '|' +
              document.getElementById('link').charset + '|' +
              document.getElementById('link').coords + '|' +
              document.getElementById('link').rev + '|' +
              document.getElementById('link').shape + '|' +
              document.getElementById('link').toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/docs/page?x=1#intro|https:|example.com|example.com||/docs/page|?x=1|#intro|https://example.com|report.txt|ja|https://p1.test https://p2.test|no-referrer|noopener noreferrer|2|_blank|text/plain|https://attr.test/src|panel|utf-8|0,0,10,10|prev|rect|https://example.com/docs/page?x=1#intro",
    )?;
    Ok(())
}

#[test]
fn anchor_username_password_and_url_part_setters_work() -> Result<()> {
    let html = r#"
        <a id='cred' href='https://u:p@example.com:8443/p?q=1#h'>cred</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initial =
              document.getElementById('cred').username + ':' +
              document.getElementById('cred').password + ':' +
              document.getElementById('cred').host + ':' +
              document.getElementById('cred').origin;

            document.getElementById('cred').username = 'alice';
            document.getElementById('cred').password = 'secret';
            document.getElementById('cred').protocol = 'http:';
            document.getElementById('cred').hostname = 'api.example.test';
            document.getElementById('cred').port = '9090';
            document.getElementById('cred').pathname = 'docs';
            document.getElementById('cred').search = 'k=v';
            document.getElementById('cred').hash = 'frag';

            document.getElementById('result').textContent =
              initial + '|' +
              document.getElementById('cred').href + '|' +
              document.getElementById('cred').username + '|' +
              document.getElementById('cred').password + '|' +
              document.getElementById('cred').protocol + '|' +
              document.getElementById('cred').host + '|' +
              document.getElementById('cred').pathname + '|' +
              document.getElementById('cred').search + '|' +
              document.getElementById('cred').hash + '|' +
              document.getElementById('cred').origin + '|' +
              document.getElementById('cred').toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "u:p:example.com:8443:https://example.com:8443|http://alice:secret@api.example.test:9090/docs?k=v#frag|alice|secret|http:|api.example.test:9090|/docs|?k=v|#frag|http://api.example.test:9090|http://alice:secret@api.example.test:9090/docs?k=v#frag",
    )?;
    Ok(())
}

#[test]
fn anchor_text_alias_and_read_only_properties_work() -> Result<()> {
    let html = r#"
        <a id='link' href='https://example.com/start' rel='noopener'>old</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('link').text = 'Updated text';

            let originReadOnly = 'no';
            try {
              document.getElementById('link').origin = 'https://evil.example';
            } catch (e) {
              originReadOnly = 'yes';
            }

            let relListReadOnly = 'no';
            try {
              document.getElementById('link').relList = 'x';
            } catch (e) {
              relListReadOnly = 'yes';
            }

            document.getElementById('result').textContent =
              document.getElementById('link').textContent + ':' +
              document.getElementById('link').text + ':' +
              originReadOnly + ':' +
              relListReadOnly + ':' +
              document.getElementById('link').origin + ':' +
              document.getElementById('link').relList.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "Updated text:Updated text:yes:yes:https://example.com:1",
    )?;
    Ok(())
}

#[test]
fn anchor_click_follows_href_for_relative_and_non_http_urls() -> Result<()> {
    let html = r#"
        <a id='web' href='/docs/page?x=1#intro'>web</a>
        <a id='mail' href='mailto:m.bluth@example.com'>mail</a>
        <a id='phone' href='tel:+123456789'>phone</a>
        "#;

    let mut h = Harness::from_html_with_url("https://example.com/base/index.html", html)?;
    h.click("#web")?;
    h.click("#mail")?;
    h.click("#phone")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "https://example.com/base/index.html".to_string(),
                to: "https://example.com/docs/page?x=1#intro".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "https://example.com/docs/page?x=1#intro".to_string(),
                to: "mailto:m.bluth@example.com".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "mailto:m.bluth@example.com".to_string(),
                to: "tel:+123456789".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn anchor_click_navigation_is_skipped_without_href_download_or_target_blank() -> Result<()> {
    let html = r#"
        <a id='nohref'>nohref</a>
        <a id='blank' href='/blank' target='_blank'>blank</a>
        <a id='download' href='/report.csv' download='report.csv'>download</a>
        <a id='self' href='/self'>self</a>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#nohref")?;
    h.click("#blank")?;
    h.click("#download")?;
    h.click("#self")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "https://app.local/self".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn press_enter_activates_anchor_with_href_and_respects_keydown_prevent_default() -> Result<()> {
    let html = r#"
        <a id='go' href='/go'>Go</a>
        <a id='blocked' href='/blocked'>Blocked</a>
        <a id='plain'>Plain</a>
        <p id='result'></p>
        <script>
          let clicks = 0;
          document.getElementById('go').addEventListener('click', () => {
            clicks = clicks + 1;
            document.getElementById('result').textContent = 'go:' + clicks;
          });
          document.getElementById('blocked').addEventListener('keydown', (event) => {
            event.preventDefault();
          });
          document.getElementById('blocked').addEventListener('click', () => {
            document.getElementById('result').textContent = 'blocked-click';
          });
          document.getElementById('plain').addEventListener('click', () => {
            document.getElementById('result').textContent = 'plain-click';
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.press_enter("#go")?;
    h.assert_text("#result", "go:1")?;
    h.press_enter("#blocked")?;
    h.assert_text("#result", "go:1")?;
    h.press_enter("#plain")?;
    h.assert_text("#result", "go:1")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "https://app.local/go".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn history_properties_push_state_and_replace_state_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initialLen = history.length;
            const initialState = history.state === null ? 'null' : 'non-null';
            history.pushState({ step: 1 }, '', 'https://app.local/one');
            const pushed = history.length + ':' + history.state.step + ':' + location.href;
            history.replaceState({ step: 2 }, '', 'https://app.local/two');
            const replaced = history.length + ':' + history.state.step + ':' + location.href;
            document.getElementById('result').textContent =
              initialLen + ':' + initialState + '|' + pushed + '|' + replaced + '|' + window.history.scrollRestoration;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1:null|2:1:https://app.local/one|2:2:https://app.local/two|auto",
    )?;
    Ok(())
}

#[test]
fn history_back_forward_and_go_dispatch_popstate_with_state() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          window.addEventListener('popstate', (event) => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              '[' + (event.state === null ? 'null' : event.state) + '@' + location.href + ']';
          });

          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = '';
            history.pushState('A', '', 'https://app.local/a');
            history.pushState('B', '', 'https://app.local/b');
            history.back();
            history.forward();
            history.go(-2);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "[A@https://app.local/a][B@https://app.local/b][null@about:blank]",
    )?;
    Ok(())
}

#[test]
fn history_out_of_bounds_navigation_is_noop() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            history.pushState('A', '', 'https://app.local/a');
            history.go(10);
            history.forward();
            history.go(-10);
            document.getElementById('result').textContent =
              history.length + ':' + history.state + ':' + location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:A:https://app.local/a")?;
    Ok(())
}

#[test]
fn history_go_reload_works_with_location_mock_page() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            history.go();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("about:blank", "<p id='marker'>reloaded</p>");
    h.click("#run")?;
    h.assert_text("#marker", "reloaded")?;
    assert_eq!(h.location_reload_count(), 1);
    Ok(())
}

#[test]
fn history_scroll_restoration_setter_and_window_history_access_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = window.history.scrollRestoration;
            history.scrollRestoration = 'manual';
            document.getElementById('result').textContent =
              before + ':' + history.scrollRestoration + ':' + window.history.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "auto:manual:1")?;
    Ok(())
}

#[test]
fn history_read_only_and_invalid_scroll_restoration_are_rejected() {
    let readonly_err = Harness::from_html(
        r#"
        <script>
          window.history.length = 2;
        </script>
        "#,
    )
    .expect_err("history.length should be read-only");
    match readonly_err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "history.length is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }

    let invalid_mode_err = Harness::from_html(
        r#"
        <script>
          history.scrollRestoration = 'smooth';
        </script>
        "#,
    )
    .expect_err("invalid scrollRestoration value should fail");
    match invalid_mode_err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "history.scrollRestoration must be 'auto' or 'manual'")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn document_has_focus_reports_active_element_state() -> Result<()> {
    let html = r#"
        <input id='name'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const before = document.hasFocus();
            document.getElementById('name').focus();
            const during = document.hasFocus();
            document.getElementById('name').blur();
            const after = document.hasFocus();
            document.getElementById('result').textContent = before + ':' + during + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true:false")?;
    Ok(())
}

#[test]
fn document_body_chain_supports_query_selector_and_query_selector_all() -> Result<()> {
    let html = r#"
        <body>
          <div id='a' class='item'></div>
          <div id='b' class='item'></div>
          <button id='btn'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('btn').addEventListener('click', () => {
              const picked = document.body.querySelector('.item');
              const total = document.body.querySelectorAll('.item').length;
              picked.classList.remove('item');
              document.getElementById('result').textContent =
                picked.id + ':' + total + ':' + document.body.querySelectorAll('.item').length;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:2:1")?;
    Ok(())
}

#[test]
fn class_list_for_each_supports_single_arg_and_index() -> Result<()> {
    let html = r#"
        <div id='box' class='red green blue'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let joined = '';
            let indexes = '';
            document.getElementById('box').classList.forEach((name, index) => {
              joined = joined + name;
              indexes = indexes + index;
            });
            document.getElementById('result').textContent = joined + ':' + indexes;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "redgreenblue:012")?;
    Ok(())
}

#[test]
fn element_click_method_from_script_works() -> Result<()> {
    let html = r#"
        <button id='trigger'>click me</button>
        <input id='agree' type='checkbox'>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('agree').click();
            document.getElementById('result').textContent =
              (document.getElementById('agree').checked ? 'checked' : 'unchecked');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "checked")?;
    h.click("#trigger")?;
    h.assert_text("#result", "unchecked")?;
    Ok(())
}

#[test]
fn anchor_download_blob_click_inside_handler_keeps_dom_state_for_following_statements() -> Result<()>
{
    let html = r#"
        <html><body>
          <button id='run'>run</button>
          <div id='result'></div>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              const blob = new Blob(['abc'], { type: 'text/plain' });
              const url = URL.createObjectURL(blob);
              const a = document.createElement('a');
              a.href = url;
              a.download = 'test.csv';
              document.body.appendChild(a);

              document.getElementById('result').textContent = 'before';
              a.click();
              document.getElementById('result').textContent += '|after';

              a.remove();
              URL.revokeObjectURL(url);
            });
          </script>
        </body></html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "before|after")?;
    assert!(h.take_location_navigations().is_empty());
    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("test.csv".to_string()),
            mime_type: Some("text/plain".to_string()),
            bytes: b"abc".to_vec(),
        }]
    );
    Ok(())
}

#[test]
fn element_scroll_into_view_method_from_script_works() -> Result<()> {
    let html = r#"
        <button id='trigger'>scroll target</button>
        <section id='target'></section>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('target').scrollIntoView();
            document.getElementById('result').textContent = 'done';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "done")?;
    Ok(())
}

#[test]
fn element_scroll_into_view_accepts_optional_argument() -> Result<()> {
    let html = r#"
        <button id='trigger'>target</button>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('trigger').scrollIntoView({ behavior: 'smooth', block: 'start' });
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn add_event_listener_accepts_async_arrow_callback() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', async () => {
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn form_submit_method_bypasses_submit_event_and_validation() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <form id='f' method='dialog'>
            <input id='name' required>
          </form>
        </dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          const form = document.getElementById('f');
          let marker = 'none';
          document.getElementById('f').addEventListener('submit', (event) => {
            marker = 'submitted';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            dialog.showModal();
            form.submit();
            document.getElementById('result').textContent = marker + ':' + dialog.open;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "none:false")?;
    Ok(())
}

#[test]
fn harness_submit_runs_validation_and_submit_event() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required>
        </form>
        <p id='result'>none</p>
        <script>
          document.getElementById('f').addEventListener('submit', () => {
            document.getElementById('result').textContent = 'submitted';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.submit("#f")?;
    h.assert_text("#result", "none")?;
    h.type_text("#name", "ok")?;
    h.submit("#f")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn form_request_submit_runs_validation_and_submit_event() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required>
        </form>
        <button id='empty'>empty</button>
        <button id='filled'>filled</button>
        <p id='result'>none</p>
        <script>
          const form = document.getElementById('f');
          const name = document.getElementById('name');
          let marker = 'none';
          form.addEventListener('submit', (event) => {
            event.preventDefault();
            marker = 'submitted';
          });
          document.getElementById('empty').addEventListener('click', () => {
            form.requestSubmit();
            document.getElementById('result').textContent = marker;
          });
          document.getElementById('filled').addEventListener('click', () => {
            name.value = 'ok';
            form.requestSubmit();
            document.getElementById('result').textContent = marker;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#empty")?;
    h.assert_text("#result", "none")?;
    h.click("#filled")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn form_request_submit_accepts_submitter_argument() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required value='ok'>
          <button id='submitter' type='submit'>send</button>
        </form>
        <button id='trigger'>run</button>
        <p id='result'>none</p>
        <script>
          const form = document.getElementById('f');
          const submitter = document.getElementById('submitter');
          form.addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = 'submitted';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            form.requestSubmit(submitter);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn form_request_submit_accepts_image_submitter_argument() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required value='ok'>
          <input id='submitter' type='image' alt='send' src='/send.png'>
        </form>
        <button id='trigger'>run</button>
        <p id='result'>none</p>
        <script>
          const form = document.getElementById('f');
          const submitter = document.getElementById('submitter');
          form.addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = 'submitted';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            form.requestSubmit(submitter);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn form_request_submit_rejects_non_submitter_argument() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required value='ok'>
          <input id='plain' type='text' value='x'>
        </form>
        <button id='trigger'>run</button>
        <script>
          const form = document.getElementById('f');
          const plain = document.getElementById('plain');
          document.getElementById('trigger').addEventListener('click', () => {
            form.requestSubmit(plain);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#trigger") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("requestSubmit submitter must be a submit control"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_request_submit_rejects_submitter_from_another_form() -> Result<()> {
    let html = r#"
        <form id='a'>
          <input id='name' required value='ok'>
          <button id='a-submit' type='submit'>a</button>
        </form>
        <form id='b'>
          <button id='b-submit' type='submit'>b</button>
        </form>
        <button id='trigger'>run</button>
        <script>
          const a = document.getElementById('a');
          const bSubmit = document.getElementById('b-submit');
          document.getElementById('trigger').addEventListener('click', () => {
            a.requestSubmit(bSubmit);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#trigger") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("requestSubmit submitter must belong to the target form"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_reset_method_dispatches_reset_and_restores_defaults() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='default'>
          <input id='agree' type='checkbox' checked>
        </form>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          let marker = '';
          document.getElementById('f').addEventListener('reset', () => {
            marker = marker + 'reset';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('name').value = 'changed';
            document.getElementById('agree').checked = false;
            document.getElementById('f').reset();
            document.getElementById('result').textContent =
              marker + ':' +
              document.getElementById('name').value + ':' +
              (document.getElementById('agree').checked ? 'on' : 'off');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "reset:default:on")?;
    Ok(())
}

#[test]
fn dialog_show_modal_close_and_toggle_events_work() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <button id='close' type='button'>Close</button>
          <form method='dialog' id='form'>
            <p>
              <label for='fav-animal'>Favorite animal:</label>
              <select id='fav-animal' name='favAnimal' required>
                <option></option>
                <option>Brine shrimp</option>
                <option>Red panda</option>
                <option>Spider monkey</option>
              </select>
            </p>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='open'>Open dialog</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let logs = '';

          dialog.addEventListener('beforetoggle', (event) => {
            logs = logs + 'before:' + event.oldState + '>' + event.newState + '|';
          });
          dialog.addEventListener('toggle', (event) => {
            logs = logs + 'toggle:' + event.newState + '|';
          });
          dialog.addEventListener('close', () => {
            logs = logs + 'close:' + dialog.returnValue + '|';
          });

          document.getElementById('open').addEventListener('click', () => {
            dialog.showModal();
            dialog.close('Red panda');
            document.getElementById('result').textContent = logs + 'open=' + dialog.open;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#open")?;
    h.assert_text(
        "#result",
        "before:closed>open|toggle:open|before:open>closed|toggle:closed|close:Red panda|open=false",
    )?;
    Ok(())
}

#[test]
fn dialog_request_close_fires_cancel_and_can_be_prevented() -> Result<()> {
    let html = r#"
        <dialog id='dialog' open></dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let marker = '';
          dialog.addEventListener('cancel', (event) => {
            marker = marker + 'cancel:' + dialog.returnValue;
            dialog.returnValue = '';
            event.preventDefault();
          });
          dialog.addEventListener('close', () => {
            marker = marker + '|close';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            dialog.returnValue = 'seed';
            dialog.requestClose('next');
            document.getElementById('result').textContent =
              marker + '|open=' + dialog.open + '|value=' + dialog.returnValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "cancel:next|open=true|value=")?;
    Ok(())
}

#[test]
fn dialog_form_method_dialog_closes_and_keeps_submit_return_value() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <form id='form' method='dialog'>
            <select id='fav-animal' required>
              <option></option>
              <option>Brine shrimp</option>
              <option>Red panda</option>
              <option>Spider monkey</option>
            </select>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          const form = document.getElementById('form');
          const select = document.getElementById('fav-animal');

          form.addEventListener('submit', () => {
            dialog.returnValue = select.value;
          });
          dialog.addEventListener('close', () => {
            document.getElementById('result').textContent =
              dialog.returnValue + ':' + dialog.open;
          });

          document.getElementById('trigger').addEventListener('click', () => {
            dialog.show();
            select.value = 'Spider monkey';
            document.getElementById('submit').click();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "Spider monkey:false")?;
    Ok(())
}

#[test]
fn dialog_form_submit_is_blocked_when_required_control_is_empty() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <form id='form' method='dialog'>
            <select id='fav-animal' required>
              <option></option>
              <option>Brine shrimp</option>
            </select>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let marker = 'none';
          document.getElementById('form').addEventListener('submit', () => {
            marker = 'submitted';
          });
          dialog.addEventListener('close', () => {
            marker = marker + '|closed';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            dialog.showModal();
            document.getElementById('submit').click();
            document.getElementById('result').textContent = marker + ':' + dialog.open;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "none:true")?;
    Ok(())
}

#[test]
fn dialog_closed_by_property_reflects_closedby_attribute() -> Result<()> {
    let html = r#"
        <dialog id='dialog' closedby='none'></dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          document.getElementById('trigger').addEventListener('click', () => {
            const before = dialog.closedBy;
            dialog.closedBy = 'any';
            document.getElementById('result').textContent =
              before + ':' + dialog.closedBy + ':' + dialog.getAttribute('closedby');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "none:any:any")?;
    Ok(())
}

#[test]
fn element_matches_method_works() -> Result<()> {
    let html = r#"
        <div id='container'>
          <button id='target' class='item primary'></button>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const direct = document.getElementById('target').matches('#target.item');
            const byTag = document.getElementById('target').matches('button');
            const bySelectorMismatch = document.getElementById('target').matches('.secondary');
            document.getElementById('result').textContent =
              direct + ':' + byTag + ':' + bySelectorMismatch;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn element_closest_method_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <section id='scope'>
            <div id='container'>
              <button id='btn'>run</button>
            </div>
          </section>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const scoped = document.getElementById('btn').closest('section');
            const selfMatch = document.getElementById('btn').closest('#btn');
            document.getElementById('result').textContent =
              scoped.id + ':' + selfMatch.id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "scope:btn")?;
    Ok(())
}

#[test]
fn element_closest_method_returns_null_when_not_found() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const matched = document.getElementById('btn').closest('section');
            document.getElementById('result').textContent = matched ? 'found' : 'missing';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "missing")?;
    Ok(())
}

#[test]
fn query_selector_all_foreach_and_element_variables_work() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.querySelectorAll('.item').forEach((item, idx) => {
              item.setAttribute('data-idx', idx);
              item.classList.toggle('picked', idx === 1);
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + item.textContent + item.getAttribute('data-idx');
            });
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + document.querySelectorAll('.item')[1].classList.contains('picked');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A0B1:true")?;
    Ok(())
}

#[test]
fn query_selector_all_foreach_single_arg_callback_works() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
        document.querySelectorAll('.item').forEach(item => {
              item.classList.add('seen');
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + item.textContent;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn parse_for_each_callback_accepts_arrow_expression_body() -> Result<()> {
    let (item_var, index_var, body) = parse_for_each_callback("item => 1")?;
    assert_eq!(item_var, "item");
    assert!(index_var.is_none());
    assert_eq!(body.len(), 1);
    match body
        .first()
        .expect("callback body should include one statement")
    {
        Stmt::Expr(Expr::Number(value)) => assert_eq!(*value, 1),
        other => panic!("unexpected callback body stmt: {other:?}"),
    }
    Ok(())
}

#[test]
fn listener_arrow_expression_callback_body_executes() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click',
            () => 1
          );
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn for_of_loop_supports_query_selector_all() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let output = '';
            for (const item of document.querySelectorAll('.item')) {
              output = output + item.textContent;
            }
            document.getElementById('result').textContent = output;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC")?;
    Ok(())
}

#[test]
fn for_in_loop_supports_query_selector_all_indexes() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let output = '';
            for (let index in document.querySelectorAll('.item')) {
              output = output + index + ',';
            }
            document.getElementById('result').textContent = output;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0,1,2,")?;
    Ok(())
}

#[test]
fn for_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (let i = 0; i < 5; i = i + 1) {
              if (i === 0) {
                continue;
              }
              if (i === 3) {
                break;
              }
              out = out + i;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12")?;
    Ok(())
}

#[test]
fn while_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            let i = 0;
            while (i < 5) {
              i = i + 1;
              if (i === 1) {
                continue;
              }
              if (i === 4) {
                break;
              }
              out = out + i;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "23")?;
    Ok(())
}

#[test]
fn do_while_executes_at_least_once() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let count = 0;
            do {
              count = count + 1;
            } while (false);
            document.getElementById('result').textContent = count;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn do_while_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            let out = '';
            do {
              i = i + 1;
              if (i === 1) {
                continue;
              }
              if (i === 4) {
                break;
              }
              out = out + i;
            } while (i < 5);
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "23")?;
    Ok(())
}
