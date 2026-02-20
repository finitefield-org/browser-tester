use super::*;

#[test]
fn insert_adjacent_html_invalid_position_expression_fails() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const pos = 'outer';
            const b = document.getElementById('b');
            b.insertAdjacentHTML(pos, '<i>T</i>');
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h.click("#btn").expect_err("invalid position should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("unsupported insertAdjacentHTML position"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn inner_html_set_replaces_children_and_updates_id_index() -> Result<()> {
    let html = r#"
        <div id='box'><span id='old'>O</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML = '<span id="new">N</span><b>B</b>';
            const same = box.innerHTML === '<span id="new">N</span><b>B</b>';
            document.getElementById('result').textContent =
              box.textContent + ':' +
              document.querySelectorAll('#old').length + ':' +
              document.querySelectorAll('#new').length + ':' +
              same;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "NB:0:1:true")?;
    Ok(())
}

#[test]
fn inner_html_getter_returns_markup_with_text_nodes() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML = 'A<i id="x">X</i>C';
            document.getElementById('result').textContent =
              box.innerHTML + ':' + document.getElementById('x').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A<i id=\"x\">X</i>C:X")?;
    Ok(())
}

#[test]
fn inner_html_set_sanitizes_scripts_and_dangerous_attrs() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML =
              '<script id="evil">document.getElementById("result").textContent = "pwned";</script>' +
              '<a id="link" href="javascript:alert(1)" onclick="alert(1)">safe</a>';
            document.getElementById('result').textContent =
              document.querySelectorAll('#evil').length + ':' +
              document.getElementById('link').hasAttribute('onclick') + ':' +
              document.getElementById('link').hasAttribute('href') + ':' +
              box.innerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:false:false:<a id=\"link\">safe</a>")?;
    Ok(())
}

#[test]
fn inner_html_getter_escapes_text_and_attr_and_keeps_void_tags() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML = '<span id="x" title="a&b"></span><br>';
            document.getElementById('x').textContent = '1 < 2 & 3 > 0';
            document.getElementById('result').textContent = box.innerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "<span id=\"x\" title=\"a&amp;b\">1 &lt; 2 &amp; 3 &gt; 0</span><br>",
    )?;
    Ok(())
}

#[test]
fn detached_element_id_is_not_queryable_until_attached() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const node = document.createElement('div');
            node.id = 'late';
            document.getElementById('result').textContent =
              document.querySelectorAll('#late').length + ':';
            document.getElementById('root').appendChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              document.querySelectorAll('#late').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:1")?;
    Ok(())
}

#[test]
fn create_text_node_append_and_remove_work() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const text = document.createTextNode('A');
            root.appendChild(text);
            document.getElementById('result').textContent = root.textContent + ':';
            text.remove();
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + root.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A:")?;
    Ok(())
}

#[test]
fn node_remove_detaches_and_updates_id_index() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const el = document.createElement('div');
            el.id = 'gone';
            root.appendChild(el);
            el.remove();
            el.remove();
            document.getElementById('result').textContent =
              document.querySelectorAll('#gone').length + ':' + root.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:")?;
    Ok(())
}

#[test]
fn duplicate_id_prefers_first_match_for_id_selector_api() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='dup'>first</span>
          <span id='dup'>second</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const byId = document.getElementById('dup');
            const all = document.querySelectorAll('#dup').length;
            const bySelector = document.querySelector('#dup');
            document.getElementById('result').textContent =
              byId.textContent + ':' + all + ':' + bySelector.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first:2:first")?;
    Ok(())
}

#[test]
fn duplicate_id_returns_next_match_after_removal_of_first() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='first'>first</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.getElementById('first');
            first.remove();

            const a = document.createElement('span');
            a.id = 'dup';
            a.textContent = 'a';
            const b = document.createElement('span');
            b.id = 'dup';
            b.textContent = 'b';
            const root = document.getElementById('root');
            root.appendChild(a);
            root.appendChild(b);

            const active = document.getElementById('dup');
            const all = document.querySelectorAll('#dup').length;
            document.getElementById('result').textContent =
              active.textContent + ':' + all + ':' + document.querySelectorAll('#first').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:2:0")?;
    Ok(())
}

#[test]
fn selector_child_combinator_and_attr_exists_work() -> Result<()> {
    let html = r#"
        <div id='wrap'>
          <div><span id='nested' data-role='x'></span></div>
          <span id='direct' data-role='x'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.querySelector('#wrap>[data-role]').id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "direct")?;
    Ok(())
}

#[test]
fn selector_group_and_document_order_dedup_work() -> Result<()> {
    let html = r#"
        <div>
          <span id='second'></span>
          <span id='first'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const firstMatch = document.querySelector('#first, #second').id;
            const same = document.querySelectorAll('#first, #first').length;
            const both = document.querySelectorAll('#first, #second').length;
            document.getElementById('result').textContent =
              firstMatch + ':' + same + ':' + both;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "second:1:2")?;
    Ok(())
}

#[test]
fn selector_adjacent_and_general_sibling_combinators_work() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='a' class='item'>A</li>
          <li id='b' class='item'>B</li>
          <li id='c' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const adjacent = document.querySelector('#a + .item').id;
            const siblings = document.querySelectorAll('#a ~ .item').length;
            document.getElementById('result').textContent = adjacent + ':' + siblings;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "b:2")?;
    Ok(())
}

#[test]
fn selector_compound_tag_id_class_and_attr_work() -> Result<()> {
    let html = r#"
        <div>
          <span id='target' class='x y' data-role='main' data-on='1'></span>
          <span id='other' class='x' data-role='main'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const exact = document.querySelector("span#target.x.y[data-role='main'][data-on]").id;
            const many = document.querySelectorAll("span.x[data-role='main']").length;
            document.getElementById('result').textContent = exact + ':' + many;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "target:2")?;
    Ok(())
}

#[test]
fn selector_attr_operators_work() -> Result<()> {
    let html = r#"
        <div>
          <span id='first'
            data-code='pre-middle-post'
            tags='alpha one beta'
            lang='en-US'></span>
          <span id='second' data-code='other' tags='two three' lang='fr'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const p1 = document.querySelector('[data-code^=\"pre\"]').id;
            const p2 = document.querySelector('[data-code$=\"post\"]').id;
            const p3 = document.querySelector('[data-code*=\"middle\"]').id;
            const p4 = document.querySelector('[tags~=\"one\"]').id;
            const p5 = document.querySelector('[lang|=\"en\"]').id;
            document.getElementById('result').textContent =
              p1 + ':' + p2 + ':' + p3 + ':' + p4 + ':' + p5;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first:first:first:first:first")?;
    Ok(())
}

#[test]
fn selector_attr_empty_and_case_insensitive_key_work() -> Result<()> {
    let html = r#"
        <div>
          <span id='target' data-empty='' data-flag='X'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const exact = document.querySelector('[data-empty=\"\"]').id;
            const empty = document.querySelector('[data-empty=]').id;
            const keycase = document.querySelector('[DATA-EMPTY=\"\"]').id;
            document.getElementById('result').textContent = exact + ':' + empty + ':' + keycase;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "target:target:target")?;
    Ok(())
}

#[test]
fn selector_universal_selector_matches_first_element() -> Result<()> {
    let html = r#"
        <div id='root'>
          <section id='first'>A</section>
          <p id='second'>B</p>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = document.querySelector('*').id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "root")?;
    Ok(())
}

#[test]
fn selector_universal_with_class_selector_work() -> Result<()> {
    let html = r#"
        <main id='root'>
          <p id='first' class='x'>A</p>
          <span id='second' class='x'>B</span>
          <div id='third' class='x'>C</div>
        </main>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = document.querySelector('*.x').id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first")?;
    Ok(())
}

#[test]
fn selector_pseudo_classes_work() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first' class='item'>A</li>
          <li id='second' class='item'>B</li>
          <li id='third' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('.item:first-child').id;
            const last = document.querySelector('.item:last-child').id;
            const second = document.querySelector('li:nth-child(2)').id;
            document.getElementById('result').textContent = first + ':' + last + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first:third:second")?;
    Ok(())
}

#[test]
fn selector_empty_works() -> Result<()> {
    let html = r#"
        <div id='root'><span id='empty'></span><span id='filled'>A</span><span id='nested'><em></em></span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('#root span:empty').id;
            const total = document.querySelectorAll('#root span:empty').length;
            document.getElementById('result').textContent =
              first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "empty:1")?;
    Ok(())
}

#[test]
fn selector_nth_child_odd_even_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-child(odd)').id;
            const even = document.querySelector('li:nth-child(even)').id;
            document.getElementById('result').textContent = odd + ':' + even;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "one:two")?;
    Ok(())
}

#[test]
fn selector_nth_child_n_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const every = document.querySelector('li:nth-child(n)').id;
            const count = document.querySelectorAll('li:nth-child( n )').length;
            document.getElementById('result').textContent = every + ':' + count;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "one:3")?;
    Ok(())
}

#[test]
fn selector_parse_rejects_invalid_nth_child() {
    assert!(
        parse_selector_step("li:nth-child(0)").is_err(),
        "nth-child(0) should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:nth-child(-1)").is_err(),
        "negative nth-child should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:nth-child(2n+)").is_err(),
        "malformed expression nth-child should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-child(n1)").is_err(),
        "invalid expression nth-child should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-last-child(n1)").is_err(),
        "invalid expression nth-last-child should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-last-child(0)").is_err(),
        "nth-last-child(0) should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:nth-last-child(2n+)").is_err(),
        "malformed expression nth-last-child should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-of-type(0)").is_err(),
        "nth-of-type(0) should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:nth-of-type(2n+)").is_err(),
        "malformed expression nth-of-type should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-last-of-type(2n+)").is_err(),
        "malformed expression nth-last-of-type should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-last-of-type(0)").is_err(),
        "nth-last-of-type(0) should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:not()").is_err(),
        "empty :not should be invalid"
    );

    assert_eq!(
        split_selector_groups("li:not([data='a,b']) , #x").map(|groups| groups.len()),
        Ok(2)
    );
    assert_eq!(
        parse_selector_groups("li:not(.skip, #first), #x").map(|groups| groups.len()),
        Ok(2)
    );
}

#[test]
fn selector_nth_child_an_plus_b_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
          <li id='five' class='item'>E</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first_odd = document.querySelector('li:nth-child(2n+1)').id;
            const odd_count = document.querySelectorAll('li:nth-child(2n+1)').length;
            const shifted = document.querySelector('li:nth-child(-n+3)').id;
            document.getElementById('result').textContent = first_odd + ':' + odd_count + ':' + shifted;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "one:3:one")?;
    Ok(())
}

#[test]
fn selector_nth_last_child_an_plus_b_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
          <li id='five' class='item'>E</li>
          <li id='six' class='item'>F</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-last-child(2n+1)').id;
            const count = document.querySelectorAll('li:nth-last-child(2n+1)').length;
            const shifted = document.querySelector('li:nth-last-child(-n+3)').id;
            document.getElementById('result').textContent = first + ':' + count + ':' + shifted;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "two:3:four")?;
    Ok(())
}

#[test]
fn selector_nth_of_type_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-of-type(odd)').id;
            const even = document.querySelector('li:nth-of-type(even)').id;
            const exact = document.querySelector('li:nth-of-type(3)').id;
            const expression = document.querySelectorAll('li:nth-of-type(2n)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + exact + ':' + expression;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first-li:second-li:third-li:2")?;
    Ok(())
}

#[test]
fn selector_nth_of_type_n_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-of-type(n)').id;
            const all = document.querySelectorAll('li:nth-of-type(n)').length;
            document.getElementById('result').textContent = first + ':' + all;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first-li:4")?;
    Ok(())
}

#[test]
fn selector_nth_last_of_type_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-last-of-type(odd)').id;
            const even = document.querySelector('li:nth-last-of-type(even)').id;
            const exact = document.querySelector('li:nth-last-of-type(2)').id;
            const expression = document.querySelectorAll('li:nth-last-of-type(2n)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + exact + ':' + expression;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "second-li:first-li:third-li:2")?;
    Ok(())
}

#[test]
fn selector_nth_last_of_type_n_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-last-of-type(n)').id;
            const all = document.querySelectorAll('li:nth-last-of-type(n)').length;
            document.getElementById('result').textContent = first + ':' + all;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first-li:4")?;
    Ok(())
}

#[test]
fn selector_not_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='a' class='target'>A</span>
          <span id='b'>B</span>
          <span id='c' class='target'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.target)').id;
            const total = document.querySelectorAll('span:not(.target)').length;
            const explicit = document.querySelector('span:not(#b)').id;
            document.getElementById('result').textContent = first + ':' + total + ':' + explicit;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "b:1:a")?;
    Ok(())
}

#[test]
fn selector_nested_not_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <li id='a' class='target'>A</li>
          <li id='b'>B</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const matched = document.querySelector('li:not(:not(.target))').id;
            const total = document.querySelectorAll('li:not(:not(.target))').length;
            document.getElementById('result').textContent = matched + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:1")?;
    Ok(())
}

#[test]
fn selector_not_with_multiple_selectors_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <li id='first' class='target'>A</li>
          <li id='middle'>B</li>
          <li id='skip' class='skip'>C</li>
          <li id='last'>D</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:not(.skip, #first)').id;
            const total = document.querySelectorAll('li:not(.skip, #first)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "middle:2")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_list_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='forbidden' class='scope'>
            <span id='forbidden-a'>A</span>
            <span id='forbidden-b'>B</span>
          </div>
          <span id='skip-me'>C</span>
          <div id='safe'>
            <span id='safe-a'>D</span>
          </div>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope *, #skip-me)').id;
            const total = document.querySelectorAll('span:not(.scope *, #skip-me)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "safe-a:1")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_adjacent_combinator_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='scope' class='scope'></div>
          <span id='excluded'>A</span>
          <span id='included'>B</span>
          <span id='included-2'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope + span)').id;
            const total = document.querySelectorAll('span:not(.scope + span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "included:2")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_general_sibling_combinator_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='included-before'>A</span>
          <div id='scope' class='scope'></div>
          <span id='excluded-1'>B</span>
          <span id='excluded-2'>C</span>
          <p>between</p>
          <span id='excluded-3'>D</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope ~ span)').id;
            const total = document.querySelectorAll('span:not(.scope ~ span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "included-before:1")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_list_general_sibling_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='included-before'>A</span>
          <div id='scope' class='scope'></div>
          <span id='excluded-id'>B</span>
          <span id='excluded-sibling'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope ~ span, #excluded-id)').id;
            const total = document.querySelectorAll('span:not(.scope ~ span, #excluded-id)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "included-before:1")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_child_combinator_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='scope' class='scope'>
            <span id='excluded'>A</span>
          </div>
          <span id='included'>B</span>
          <span id='included-2'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope > span)').id;
            const total = document.querySelectorAll('span:not(.scope > span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "included:2")?;
    Ok(())
}

#[test]
fn selector_not_with_multiple_not_chain_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <li id='both' class='foo bar'>A</li>
          <li id='foo-only' class='foo'>B</li>
          <li id='bar-only' class='bar'>C</li>
          <li id='none'>D</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:not(:not(.foo), :not(.bar))').id;
            const total = document.querySelectorAll('li:not(:not(.foo), :not(.bar))').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "both:1")?;
    Ok(())
}

#[test]
fn selector_first_last_of_type_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <p id='first-p'>A</p>
          <span id='first-span'>B</span>
          <p id='last-p'>C</p>
          <span id='middle-span'>D</span>
          <span id='last-span'>E</span>
          <li id='first-li'>F</li>
          <li id='last-li'>G</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const firstSpan = document.querySelector('span:first-of-type').id;
            const lastSpan = document.querySelector('span:last-of-type').id;
            const firstP = document.querySelector('p:first-of-type').id;
            const lastP = document.querySelector('p:last-of-type').id;
            const firstLi = document.querySelector('li:first-of-type').id;
            const lastLi = document.querySelector('li:last-of-type').id;
            document.getElementById('result').textContent = firstSpan + ':' + lastSpan + ':' + firstP + ':' + lastP + ':' + firstLi + ':' + lastLi;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "first-span:last-span:first-p:last-p:first-li:last-li",
    )?;
    Ok(())
}

#[test]
fn selector_only_child_and_only_of_type_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='single-p'>
            <p id='lonely-p'>A</p>
          </div>
          <div id='group'>
            <span id='only-span'>B</span>
          </div>
          <section id='mixed-of-type'>
            <span id='mixed-only-span'>C</span>
            <em id='mixed-only-em'>D</em>
          </section>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const lonely = document.querySelector('p:only-child').id;
            const onlySpanInGroup = document.querySelector('#group span:only-child').id;
            const onlySpanOfType = document.querySelector('#mixed-of-type span:only-of-type').id;
            const onlyEmOfType = document.querySelector('#mixed-of-type em:only-of-type').id;
            document.getElementById('result').textContent = lonely + ':' + onlySpanInGroup + ':' + onlySpanOfType + ':' + onlyEmOfType;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "lonely-p:only-span:mixed-only-span:mixed-only-em",
    )?;
    Ok(())
}

#[test]
fn selector_checked_disabled_enabled_works() -> Result<()> {
    let html = r#"
        <input id='enabled' value='ok'>
        <input id='disabled' disabled value='ng'>
        <input id='unchecked' type='checkbox'>
        <input id='checked' type='checkbox' checked>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const checked = document.querySelector('input:checked').id;
            const disabled = document.querySelector('input:disabled').id;
            const enabled = document.querySelector('input:enabled').id;
            document.getElementById('result').textContent = checked + ':' + disabled + ':' + enabled;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "checked:disabled:enabled")?;
    Ok(())
}

#[test]
fn selector_required_optional_readonly_readwrite_works() -> Result<()> {
    let html = r#"
        <input id='r' required value='r'>
        <input id='o'>
        <input id='ro' readonly>
        <input id='rw'>
        <input id='r2'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const required = document.querySelector('input:required').id;
            const optional = document.querySelector('input:optional').id;
            const readOnly = document.querySelector('input:readonly').id;
            const readWrite = document.querySelector('input:read-write').id;
            const summary =
              required + ':' + optional + ':' + readOnly + ':' + readWrite;
            document.getElementById('r').required = false;
            document.getElementById('r2').required = true;
            const afterRequired = document.querySelector('input:required').id;
            const afterOptional = document.querySelector('input:optional').id;
            document.getElementById('result').textContent =
              summary + ':' + afterRequired + ':' + afterOptional;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "r:o:ro:r:r2:r")?;
    Ok(())
}

#[test]
fn selector_trailing_group_separator_is_rejected() -> Result<()> {
    let html = r#"<div id='x'></div>"#;
    let h = Harness::from_html(html)?;
    let err = h
        .assert_exists("#x,")
        .expect_err("selector should be invalid");
    match err {
        Error::UnsupportedSelector(selector) => assert_eq!(selector, "#x,"),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn selector_parse_supports_nth_child_single_arg() {
    let step = parse_selector_step("li:nth-child(2)").expect("parse should succeed");
    assert_eq!(step.tag, Some("li".into()));
    assert_eq!(
        step.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::Exact(2))]
    );
}

#[test]
fn selector_parse_supports_nth_child_odd_even() {
    let odd = parse_selector_step("li:nth-child(odd)").expect("parse should succeed");
    let even = parse_selector_step("li:nth-child(even)").expect("parse should succeed");
    assert_eq!(
        odd.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::Odd)]
    );
    assert_eq!(
        even.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::Even)]
    );
}

#[test]
fn selector_parse_supports_nth_child_n() {
    let n = parse_selector_step("li:nth-child(n)").expect("parse should succeed");
    assert_eq!(
        n.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
            1, 0
        ))]
    );
}

#[test]
fn selector_parse_supports_nth_last_child_an_plus_b() {
    let direct = parse_selector_step("li:nth-last-child(2n+1)").expect("parse should succeed");
    assert_eq!(
        direct.pseudo_classes,
        vec![SelectorPseudoClass::NthLastChild(
            NthChildSelector::AnPlusB(2, 1)
        )]
    );
}

#[test]
fn selector_parse_supports_nth_last_child_odd_even_exact() {
    let odd = parse_selector_step("li:nth-last-child(odd)").expect("parse should succeed");
    let even = parse_selector_step("li:nth-last-child(even)").expect("parse should succeed");
    let exact = parse_selector_step("li:nth-last-child(2)").expect("parse should succeed");
    assert_eq!(
        odd.pseudo_classes,
        vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Odd)]
    );
    assert_eq!(
        even.pseudo_classes,
        vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Even)]
    );
    assert_eq!(
        exact.pseudo_classes,
        vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Exact(
            2
        ))]
    );
}

#[test]
fn selector_parse_supports_nth_child_an_plus_b() {
    let direct = parse_selector_step("li:nth-child(2n+1)").expect("parse should succeed");
    let shifted = parse_selector_step("li:nth-child(-n+3)").expect("parse should succeed");
    assert_eq!(
        direct.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
            2, 1
        ))]
    );
    assert_eq!(
        shifted.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
            -1, 3
        ))]
    );
}

#[test]
fn selector_parse_supports_first_last_of_type() {
    let first = parse_selector_step("li:first-of-type").expect("parse should succeed");
    let last = parse_selector_step("li:last-of-type").expect("parse should succeed");
    assert_eq!(first.pseudo_classes, vec![SelectorPseudoClass::FirstOfType]);
    assert_eq!(last.pseudo_classes, vec![SelectorPseudoClass::LastOfType]);
}

#[test]
fn selector_parse_supports_empty() {
    let parsed = parse_selector_step("span:empty").expect("parse should succeed");
    assert_eq!(parsed.pseudo_classes, vec![SelectorPseudoClass::Empty]);
}

#[test]
fn selector_parse_supports_only_child_and_only_of_type() {
    let only_child = parse_selector_step("li:only-child").expect("parse should succeed");
    let only_of_type = parse_selector_step("li:only-of-type").expect("parse should succeed");
    assert_eq!(
        only_child.pseudo_classes,
        vec![SelectorPseudoClass::OnlyChild]
    );
    assert_eq!(
        only_of_type.pseudo_classes,
        vec![SelectorPseudoClass::OnlyOfType]
    );
}

#[test]
fn selector_parse_supports_checked_disabled_enabled() {
    let checked = parse_selector_step("input:checked").expect("parse should succeed");
    let indeterminate = parse_selector_step("input:indeterminate").expect("parse should succeed");
    let disabled = parse_selector_step("input:disabled").expect("parse should succeed");
    let enabled = parse_selector_step("input:enabled").expect("parse should succeed");
    assert_eq!(checked.pseudo_classes, vec![SelectorPseudoClass::Checked]);
    assert_eq!(
        indeterminate.pseudo_classes,
        vec![SelectorPseudoClass::Indeterminate]
    );
    assert_eq!(disabled.pseudo_classes, vec![SelectorPseudoClass::Disabled]);
    assert_eq!(enabled.pseudo_classes, vec![SelectorPseudoClass::Enabled]);
}

#[test]
fn selector_parse_supports_required_optional_readonly_readwrite() {
    let required = parse_selector_step("input:required").expect("parse should succeed");
    let optional = parse_selector_step("input:optional").expect("parse should succeed");
    let read_only = parse_selector_step("input:read-only").expect("parse should succeed");
    let read_only_alias = parse_selector_step("input:readonly").expect("parse should succeed");
    let read_write = parse_selector_step("input:read-write").expect("parse should succeed");
    assert_eq!(required.pseudo_classes, vec![SelectorPseudoClass::Required]);
    assert_eq!(optional.pseudo_classes, vec![SelectorPseudoClass::Optional]);
    assert_eq!(
        read_only.pseudo_classes,
        vec![SelectorPseudoClass::Readonly]
    );
    assert_eq!(
        read_only_alias.pseudo_classes,
        vec![SelectorPseudoClass::Readonly]
    );
    assert_eq!(
        read_write.pseudo_classes,
        vec![SelectorPseudoClass::Readwrite]
    );
}

#[test]
fn selector_parse_supports_focus_and_focus_within() {
    let focus = parse_selector_step("input:focus").expect("parse should succeed");
    let focus_within = parse_selector_step("div:focus-within").expect("parse should succeed");
    assert_eq!(focus.pseudo_classes, vec![SelectorPseudoClass::Focus]);
    assert_eq!(
        focus_within.pseudo_classes,
        vec![SelectorPseudoClass::FocusWithin]
    );
}
