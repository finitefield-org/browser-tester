use super::*;

#[test]
fn regex_literal_test_and_exec_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = /ab+c/i;
            const ok1 = re.test('xxABBCyy');
            const ok2 = /foo.bar/s.test('foo\nbar');
            const hit = /(ab)(cd)/.exec('xabcdz');
            document.getElementById('result').textContent =
              ok1 + ':' + ok2 + ':' + hit[0] + ':' + hit[1] + ':' + hit[2];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:abcd:ab:cd")?;
    Ok(())
}

#[test]
fn regexp_constructor_and_global_sticky_exec_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = new RegExp('a.', 'g');
            const m1 = re.exec('a1a2');
            const m2 = re.exec('a1a2');
            const m3 = re.exec('a1a2');

            const sticky = /a./y;
            const y1 = sticky.exec('a1xa2');
            const y2 = sticky.exec('a1xa2');
            const y3 = sticky.exec('a1xa2');

            document.getElementById('result').textContent =
              m1[0] + ':' + m2[0] + ':' + m3 + ':' +
              y1[0] + ':' + y2 + ':' + y3[0];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a1:a2:null:a1:null:a1")?;
    Ok(())
}

#[test]
fn regex_parse_and_runtime_errors_are_reported() -> Result<()> {
    let parse_err = Harness::from_html("<script>const re = /a/gg;</script>")
        .expect_err("duplicate regex flags should fail during parse");
    match parse_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression flags")),
        other => panic!("unexpected regex parse error: {other:?}"),
    }

    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            new RegExp('(', 'g');
          });
        </script>
        "#;
    let mut h = Harness::from_html(html)?;
    let runtime_err = h
        .click("#btn")
        .expect_err("invalid RegExp constructor pattern should fail");
    match runtime_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected regex runtime error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_constructor_properties_and_escape_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = RegExp('a.', 'gimsydu');
            re.lastIndex = 3.8;
            const info =
              re.source + ':' + re.flags + ':' +
              re.global + ':' + re.ignoreCase + ':' + re.multiline + ':' +
              re.dotAll + ':' + re.sticky + ':' + re.hasIndices + ':' +
              re.unicode + ':' + re.unicodeSets + ':' +
              re.lastIndex + ':' + (re.constructor === RegExp) + ':' + typeof RegExp;
            const escaped = RegExp.escape('a+b*c?');
            const escapedWindow = window.RegExp.escape('x.y');
            document.getElementById('result').textContent =
              info + '|' + escaped + '|' + escapedWindow;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "a.:gimsydu:true:true:true:true:true:true:true:false:3:true:function|a\\+b\\*c\\?|x\\.y",
    )?;
    Ok(())
}

#[test]
fn regexp_string_match_split_and_replace_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = /(\w+)\s(\w+)/;
            const changed = 'Maria Cruz'.replace(re, '$2, $1');
            const text = 'Some text\nAnd some more\r\nAnd yet\nThis is the end';
            const lines = text.split(/\r?\n/);
            const multi = 'Please yes\nmake my day!';
            const noDotAll = multi.match(/yes.*day/) === null;
            const withDotAll = multi.match(/yes.*day/s);
            const withDotAllOk = withDotAll[0] === 'yes\nmake my day';
            const order = 'Let me get some bacon and eggs, please';
            const picks = order.match(new RegExp('\\b(bacon|eggs)\\b', 'g'));

            document.getElementById('result').textContent =
              changed + '|' +
              lines[0] + ':' + lines[1] + ':' + lines[2] + ':' + lines[3] + '|' +
              noDotAll + ':' + withDotAllOk + '|' +
              picks[0] + ':' + picks[1];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Cruz, Maria|Some text:And some more:And yet:This is the end|true:true|bacon:eggs",
    )?;
    Ok(())
}

#[test]
fn regexp_constructor_call_without_new_and_to_string_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = RegExp(/ab+c/, 'i');
            const text = re.toString();
            const ok = re.test('xxABBCyy');
            const hit = re.exec('xxABBCyy');
            document.getElementById('result').textContent = text + ':' + ok + ':' + hit[0];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "/ab+c/i:true:ABBC")?;
    Ok(())
}

#[test]
fn string_ends_with_rejects_regexp_argument() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            'foobar'.endsWith(/bar/);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("endsWith should reject RegExp arguments");
    match err {
        Error::ScriptRuntime(msg) => assert!(
            msg.contains("must not be a regular expression"),
            "unexpected message: {msg}"
        ),
        other => panic!("unexpected endsWith error: {other:?}"),
    }
    Ok(())
}

#[test]
fn symbol_constructor_and_typeof_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sym1 = Symbol();
            const sym2 = Symbol('foo');
            const sym3 = Symbol('foo');
            document.getElementById('result').textContent =
              (typeof sym1) + ':' +
              (typeof sym2) + ':' +
              (typeof Symbol.iterator) + ':' +
              (sym2 === sym3) + ':' +
              (sym1.description === undefined) + ':' +
              sym2.description + ':' +
              (Symbol.iterator === Symbol.iterator);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "symbol:symbol:symbol:false:true:foo:true")?;
    Ok(())
}

#[test]
fn symbol_for_and_key_for_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const reg1 = Symbol.for('tokenString');
            const reg2 = Symbol.for('tokenString');
            const local = Symbol('tokenString');
            document.getElementById('result').textContent =
              (reg1 === reg2) + ':' +
              (reg1 === local) + ':' +
              Symbol.keyFor(reg1) + ':' +
              (Symbol.keyFor(local) === undefined) + ':' +
              (Symbol.keyFor(Symbol.for('tokenString')) === 'tokenString') + ':' +
              (Symbol.keyFor(Symbol.iterator) === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:tokenString:true:true:true")?;
    Ok(())
}

#[test]
fn symbol_properties_and_get_own_property_symbols_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = {};
            obj[Symbol('a')] = 'a';
            obj[Symbol.for('b')] = 'b';
            obj['c'] = 'c';
            obj.d = 'd';

            const keys = Object.keys(obj);
            const values = Object.values(obj);
            const entries = Object.entries(obj);
            const symbols = Object.getOwnPropertySymbols(obj);
            const first = obj[symbols[0]];
            const second = obj[symbols[1]];

            document.getElementById('result').textContent =
              keys.join(',') + '|' +
              values.join(',') + '|' +
              entries.length + '|' +
              symbols.length + '|' +
              first + ':' + second + '|' +
              JSON.stringify(obj);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "c,d|c,d|2|2|a:b|{\"c\":\"c\",\"d\":\"d\"}")?;
    Ok(())
}

#[test]
fn symbol_wrapper_objects_can_be_used_as_property_keys() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sym = Symbol('foo');
            const obj = { [sym]: 1 };
            document.getElementById('result').textContent =
              (typeof sym) + ':' +
              (typeof Object(sym)) + ':' +
              obj[sym] + ':' +
              obj[Object(sym)] + ':' +
              (Object(sym) == sym);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "symbol:object:1:1:true")?;
    Ok(())
}

#[test]
fn symbol_constructor_and_key_for_errors_are_reported() -> Result<()> {
    let err =
        Harness::from_html("<script>new Symbol();</script>").expect_err("new Symbol should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Symbol is not a constructor")),
        other => panic!("unexpected new Symbol error: {other:?}"),
    }

    let err = Harness::from_html("<script>Symbol.keyFor('x');</script>")
        .expect_err("Symbol.keyFor non-symbol should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Symbol.keyFor argument must be a Symbol"))
        }
        other => panic!("unexpected Symbol.keyFor error: {other:?}"),
    }

    Ok(())
}

#[test]
fn symbol_implicit_conversion_errors_are_reported() {
    let err = Harness::from_html("<script>const sym = Symbol('foo'); sym + 'bar';</script>")
        .expect_err("symbol string concatenation should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Cannot convert a Symbol value to a string"))
        }
        other => panic!("unexpected symbol concat error: {other:?}"),
    }

    let err = Harness::from_html("<script>const sym = Symbol('foo'); +sym;</script>")
        .expect_err("unary plus on symbol should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Cannot convert a Symbol value to a number"))
        }
        other => panic!("unexpected unary plus symbol error: {other:?}"),
    }
}

#[test]
fn numeric_literals_support_hex_octal_binary_and_scientific_notation() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const hex = 0x10;
            const oct = 0o10;
            const bin = 0b10;
            const exp = 1e3;
            document.getElementById('result').textContent =
              hex + ':' + oct + ':' + bin + ':' + exp;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "16:8:2:1000")?;
    Ok(())
}

#[test]
fn encode_decode_uri_global_functions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = encodeURI('https://a.example/a b?x=1&y=2#f');
            const b = encodeURIComponent('a b&c=d');
            const c = decodeURI('https://a.example/a%20b?x=1&y=2#f');
            const d = decodeURI('%3Fx%3D1');
            const e = decodeURIComponent('a%20b%26c%3Dd');
            const f = window.encodeURIComponent('x y');
            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "https://a.example/a%20b?x=1&y=2#f|a%20b%26c%3Dd|https://a.example/a b?x=1&y=2#f|%3Fx%3D1|a b&c=d|x%20y",
        )?;
    Ok(())
}

#[test]
fn decode_uri_invalid_sequence_returns_runtime_error_for_decode_uri() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            decodeURIComponent('%E0%A4%A');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("decodeURIComponent should fail for malformed input");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("malformed URI sequence")),
        other => panic!("unexpected decode URI error: {other:?}"),
    }
    Ok(())
}

#[test]
fn escape_and_unescape_global_functions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const kana = unescape('%u3042');
            const escaped = escape('ABC abc +/' + kana);
            const unescaped = unescape(escaped);
            const viaWindow = window.unescape('%u3042%20A');
            const viaWindowEscaped = window.escape('hello world');
            document.getElementById('result').textContent =
              escaped + '|' + unescaped + '|' + viaWindow + '|' + viaWindowEscaped;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "ABC%20abc%20+/%u3042|ABC abc +/あ|あ A|hello%20world",
    )?;
    Ok(())
}

#[test]
fn window_aliases_for_global_functions_match_direct_calls() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              window.encodeURI('a b?x=1') + '|' + encodeURI('a b?x=1') + '|' +
              window.decodeURIComponent('a%20b%2Bc') + '|' + decodeURIComponent('a%20b%2Bc') + '|' +
              window.unescape(window.escape('A B')) + '|' +
              window.atob(window.btoa('ok')) + '|' +
              window.isNaN('x') + '|' +
              window.isFinite('3') + '|' +
              window.parseInt('11', 2) + '|' +
              window.parseFloat('2.5z');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "a%20b?x=1|a%20b?x=1|a b+c|a b+c|A B|ok|true|true|3|2.5",
    )?;
    Ok(())
}

#[test]
fn fetch_uses_registered_mock_response_and_records_calls() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = fetch('/api/message');
            const second = window.fetch('/api/message');
            document.getElementById('result').textContent = first + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_fetch_mock("/api/message", "ok");
    h.click("#btn")?;
    h.assert_text("#result", "ok:ok")?;
    assert_eq!(
        h.take_fetch_calls(),
        vec!["/api/message".to_string(), "/api/message".to_string()]
    );
    Ok(())
}

#[test]
fn fetch_without_mock_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            fetch('/api/missing');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("fetch without mock should fail with runtime error");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("fetch mock not found")),
        other => panic!("unexpected fetch error: {other:?}"),
    }
    Ok(())
}

#[test]
fn match_media_uses_registered_mocks_and_records_calls() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = matchMedia('(min-width: 768px)');
            const b = window.matchMedia('(prefers-color-scheme: dark)');
            const c = matchMedia('(min-width: 768px)').matches;
            const d = window.matchMedia('(prefers-color-scheme: dark)').media;
            document.getElementById('result').textContent =
              a.matches + ':' + a.media + ':' +
              b.matches + ':' + b.media + ':' +
              c + ':' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_match_media_mock("(min-width: 768px)", true);
    h.set_match_media_mock("(prefers-color-scheme: dark)", false);
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "true:(min-width: 768px):false:(prefers-color-scheme: dark):true:(prefers-color-scheme: dark)",
        )?;
    assert_eq!(
        h.take_match_media_calls(),
        vec![
            "(min-width: 768px)".to_string(),
            "(prefers-color-scheme: dark)".to_string(),
            "(min-width: 768px)".to_string(),
            "(prefers-color-scheme: dark)".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn match_media_default_value_can_be_configured() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = matchMedia('(unknown-query)').matches;
            const second = window.matchMedia('(unknown-query)').matches;
            document.getElementById('result').textContent = first + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:false")?;

    h.set_default_match_media_matches(true);
    h.click("#btn")?;
    h.assert_text("#result", "true:true")?;
    Ok(())
}

#[test]
fn navigator_clipboard_read_text_then_updates_dom() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p class='clip-text'>initial</p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            navigator.clipboard
              .readText()
              .then((clipText) => {
                document.querySelector('.clip-text').textContent = clipText;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("from-clipboard");
    h.click("#btn")?;
    h.assert_text(".clip-text", "from-clipboard")?;
    Ok(())
}

#[test]
fn navigator_clipboard_read_text_returns_empty_string_when_clipboard_is_empty() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p class='clip-text'>keep</p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            navigator.clipboard.readText().then((clipText) => {
              document.querySelector('.clip-text').textContent = clipText;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(".clip-text", "")?;
    Ok(())
}

#[test]
fn navigator_clipboard_write_text_and_window_alias_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const same = navigator.clipboard === window.navigator.clipboard;
            window.navigator.clipboard
              .writeText('saved')
              .then(() => navigator.clipboard.readText())
              .then((clipText) => {
                document.getElementById('result').textContent = same + ':' + clipText;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:saved")?;
    assert_eq!(h.clipboard_text(), "saved");
    Ok(())
}

#[test]
fn navigator_clipboard_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          navigator.clipboard = null;
        </script>
        "#,
    )
    .expect_err("navigator.clipboard should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "navigator.clipboard is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn structured_clone_deep_copies_objects_arrays_and_dates() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const source = { nested: { value: 1 }, items: [1, 2] };
            const clone = structuredClone(source);
            const sourceNested = source.nested;
            const cloneNested = clone.nested;
            const sourceItems = source.items;
            const cloneItems = clone.items;

            cloneNested.value = 9;
            cloneItems.push(3);

            const date = new Date('2020-01-02T03:04:05Z');
            const dateClone = structuredClone(date);
            dateClone.setTime(0);

            document.getElementById('result').textContent =
              sourceNested.value + ':' + cloneNested.value + ':' +
              sourceItems.length + ':' + cloneItems.length + ':' +
              (source === clone) + ':' + (sourceNested === cloneNested) + ':' +
              (sourceItems === cloneItems) + ':' +
              (date.getTime() != dateClone.getTime()) + ':' + (date === dateClone);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:9:2:3:false:false:false:true:false")?;
    Ok(())
}

#[test]
fn structured_clone_rejects_non_cloneable_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fn = () => {};
            structuredClone(fn);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("structuredClone should reject functions");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("not cloneable")),
        other => panic!("unexpected structuredClone error: {other:?}"),
    }
    Ok(())
}

#[test]
fn request_animation_frame_and_cancel_animation_frame_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            const canceled = requestAnimationFrame((ts) => {
              out.textContent = out.textContent + 'C' + ts;
            });
            window.cancelAnimationFrame(canceled);
            window.requestAnimationFrame((ts) => {
              out.textContent = out.textContent + 'R' + ts;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(15)?;
    h.assert_text("#result", "")?;
    h.advance_time(1)?;
    h.assert_text("#result", "R16")?;
    Ok(())
}

#[test]
fn function_constructor_uses_global_scope_while_closure_keeps_local_scope() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          var x = 10;

          function createFunction1() {
            const x = 20;
            return new Function("return x;");
          }

          function createFunction2() {
            const x = 20;
            function f() {
              return x;
            }
            return f;
          }

          document.getElementById('btn').addEventListener('click', () => {
            const f1 = createFunction1();
            const f2 = createFunction2();
            document.getElementById('result').textContent = f1() + ':' + f2();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "10:20")?;
    Ok(())
}

#[test]
fn alert_confirm_prompt_support_mocked_responses() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const accepted = confirm('continue?');
            const name = prompt('name?', 'guest');
            window.alert('hello ' + name);
            document.getElementById('result').textContent = accepted + ':' + name;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enqueue_confirm_response(true);
    h.enqueue_prompt_response(Some("kazu"));
    h.click("#btn")?;
    h.assert_text("#result", "true:kazu")?;
    assert_eq!(h.take_alert_messages(), vec!["hello kazu".to_string()]);
    Ok(())
}

#[test]
fn prompt_uses_default_argument_when_no_mock_response() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const name = prompt('name?', 'guest');
            document.getElementById('result').textContent = name;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "guest")?;
    Ok(())
}

#[test]
fn global_function_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>encodeURI();</script>",
            "encodeURI requires exactly one argument",
        ),
        (
            "<script>window.encodeURIComponent('a', 'b');</script>",
            "encodeURIComponent requires exactly one argument",
        ),
        (
            "<script>decodeURI('a', 'b');</script>",
            "decodeURI requires exactly one argument",
        ),
        (
            "<script>window.decodeURIComponent();</script>",
            "decodeURIComponent requires exactly one argument",
        ),
        (
            "<script>escape();</script>",
            "escape requires exactly one argument",
        ),
        (
            "<script>window.unescape('a', 'b');</script>",
            "unescape requires exactly one argument",
        ),
        (
            "<script>isNaN();</script>",
            "isNaN requires exactly one argument",
        ),
        (
            "<script>window.isFinite();</script>",
            "isFinite requires exactly one argument",
        ),
        (
            "<script>atob('YQ==', 'x');</script>",
            "atob requires exactly one argument",
        ),
        (
            "<script>window.btoa();</script>",
            "btoa requires exactly one argument",
        ),
        (
            "<script>parseFloat('1', 10);</script>",
            "parseFloat requires exactly one argument",
        ),
        (
            "<script>window.parseInt('1', 10, 10);</script>",
            "parseInt requires one or two arguments",
        ),
        (
            "<script>JSON.parse();</script>",
            "JSON.parse requires exactly one argument",
        ),
        (
            "<script>window.JSON.stringify('x', 1);</script>",
            "JSON.stringify requires exactly one argument",
        ),
        (
            "<script>fetch();</script>",
            "fetch requires exactly one argument",
        ),
        (
            "<script>matchMedia();</script>",
            "matchMedia requires exactly one argument",
        ),
        (
            "<script>navigator.clipboard.readText('x');</script>",
            "navigator.clipboard.readText takes no arguments",
        ),
        (
            "<script>window.navigator.clipboard.writeText();</script>",
            "navigator.clipboard.writeText requires exactly one argument",
        ),
        (
            "<script>structuredClone();</script>",
            "structuredClone requires exactly one argument",
        ),
        (
            "<script>alert();</script>",
            "alert requires exactly one argument",
        ),
        (
            "<script>window.confirm('ok', 'ng');</script>",
            "confirm requires exactly one argument",
        ),
        (
            "<script>prompt();</script>",
            "prompt requires one or two arguments",
        ),
        (
            "<script>window.prompt('x', );</script>",
            "prompt default argument cannot be empty",
        ),
        (
            "<script>requestAnimationFrame();</script>",
            "requestAnimationFrame requires exactly one argument",
        ),
        (
            "<script>cancelAnimationFrame();</script>",
            "cancelAnimationFrame requires 1 argument",
        ),
        (
            "<script>Array.isArray();</script>",
            "Array.isArray requires exactly one argument",
        ),
        (
            "<script>Object.keys();</script>",
            "Object.keys requires exactly one argument",
        ),
        (
            "<script>window.Object.values(1, 2);</script>",
            "Object.values requires exactly one argument",
        ),
        (
            "<script>Object.entries();</script>",
            "Object.entries requires exactly one argument",
        ),
        (
            "<script>Object.hasOwn({ a: 1 });</script>",
            "Object.hasOwn requires exactly two arguments",
        ),
        (
            "<script>const obj = {}; obj.hasOwnProperty();</script>",
            "hasOwnProperty requires exactly one argument",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn global_function_parser_respects_identifier_boundaries() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const escaped = escape('A B');
            const encodedValue = encodeURIComponent('x y');
            const parseIntValue = 7;
            const parseFloatValue = 1.25;
            const escapedValue = escaped;
            const round = unescape(escapedValue);
            document.getElementById('result').textContent =
              escapedValue + ':' + encodedValue + ':' + round + ':' +
              parseIntValue + ':' + parseFloatValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A%20B:x%20y:A B:7:1.25")?;
    Ok(())
}

#[test]
fn btoa_non_latin1_input_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nonLatin1 = unescape('%u3042');
            btoa(nonLatin1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("btoa should reject non-Latin1 input");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("non-Latin1")),
        other => panic!("unexpected btoa error: {other:?}"),
    }
    Ok(())
}

#[test]
fn decode_uri_invalid_sequence_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            decodeURI('%E0%A4%A');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("decodeURI should fail for malformed input");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("malformed URI sequence")),
        other => panic!("unexpected decode URI error: {other:?}"),
    }
    Ok(())
}

#[test]
fn is_nan_and_is_finite_global_functions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = isNaN('abc');
            const b = isNaN('  ');
            const c = isNaN(undefined);
            const d = isFinite('1.5');
            const e = isFinite(Infinity);
            const f = window.isFinite(null);
            const g = window.isNaN(NaN);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true:true:false:true:true")?;
    Ok(())
}

#[test]
fn atob_and_btoa_global_functions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const encoded = btoa('abc123!?');
            const decoded = atob(encoded);
            const viaWindow = window.atob('QQ==');
            document.getElementById('result').textContent =
              encoded + ':' + decoded + ':' + viaWindow;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "YWJjMTIzIT8=:abc123!?:A")?;
    Ok(())
}

#[test]
fn atob_invalid_input_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='atob'>atob</button>
        <script>
          document.getElementById('atob').addEventListener('click', () => {
            atob('@@@');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let atob_err = h
        .click("#atob")
        .expect_err("atob should reject invalid base64");
    match atob_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("invalid base64")),
        other => panic!("unexpected atob error: {other:?}"),
    }

    Ok(())
}

#[test]
fn parse_int_global_function_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = parseInt('42px');
            const b = parseInt('  -0x10');
            const c = parseInt('10', 2);
            const d = parseInt('10', 8);
            const e = parseInt('0x10', 16);
            const bad1 = parseInt('xyz');
            const bad2 = parseInt('10', 1);
            const f = window.parseInt('12', 10);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' +
              (bad1 === bad1) + ':' + (bad2 === bad2) + ':' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "42:-16:2:8:16:false:false:12")?;
    Ok(())
}

#[test]
fn parse_float_global_function_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = parseFloat('3.5px');
            const b = parseFloat('  -2.5e2x');
            const invalid = parseFloat('abc');
            const d = window.parseFloat('42');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + (invalid === invalid) + ':' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3.5:-250:false:42")?;
    Ok(())
}

#[test]
fn json_parse_and_stringify_roundtrip_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const source = '{"a":1,"b":[true,null,"x"],"c":{"d":2}}';
            const parsed = JSON.parse(source);
            const out = JSON.stringify(parsed);
            const arr = JSON.parse('[1,2,3]');
            const viaWindow = window.JSON.stringify(window.JSON.parse('{"x":"y"}'));
            document.getElementById('result').textContent = out + '|' + JSON.stringify(arr) + '|' + viaWindow;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "{\"a\":1,\"b\":[true,null,\"x\"],\"c\":{\"d\":2}}|[1,2,3]|{\"x\":\"y\"}",
    )?;
    Ok(())
}

#[test]
fn json_stringify_handles_special_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parsed = JSON.parse('"\\u3042\\n\\t"');
            const encoded = JSON.stringify(parsed);
            const topUndefined = JSON.stringify(undefined);
            const finite = JSON.stringify(1.5);
            const nan = JSON.stringify(NaN);
            const inf = JSON.stringify(Infinity);
            const arr = JSON.stringify([1, undefined, NaN, Infinity]);
            const obj = JSON.stringify(JSON.parse('{"a":1,"b":null}'));
            document.getElementById('result').textContent =
              encoded + '|' + topUndefined + '|' + finite + '|' + nan + '|' + inf + '|' + arr + '|' + obj;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "\"あ\\n\\t\"|undefined|1.5|null|null|[1,null,null,null]|{\"a\":1,\"b\":null}",
    )?;
    Ok(())
}

#[test]
fn json_parse_invalid_input_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            JSON.parse('{bad json}');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("JSON.parse should fail for invalid input");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("JSON.parse invalid JSON")),
        other => panic!("unexpected JSON.parse error: {other:?}"),
    }
    Ok(())
}

#[test]
fn json_stringify_circular_array_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1];
            arr.push(arr);
            JSON.stringify(arr);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("JSON.stringify should fail for circular array");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("JSON.stringify circular structure")),
        other => panic!("unexpected JSON.stringify error: {other:?}"),
    }
    Ok(())
}

#[test]
fn object_literal_property_access_and_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { a: 1, "b": 2, a: 3 };
            obj.c = 4;
            obj['d'] = obj.a + obj.b;
            obj.value = 'v';

            const keys = Object.keys(obj);
            const values = Object.values(obj);
            const entries = Object.entries(obj);
            const firstEntry = entries[0];
            const lastEntry = entries[4];
            const ownA = Object.hasOwn(obj, 'a');
            const ownZ = window.Object.hasOwn(obj, 'z');
            const ownD = obj.hasOwnProperty('d');

            document.getElementById('result').textContent =
              obj.a + ':' + obj.b + ':' + obj.c + ':' + obj.d + ':' + obj.value + '|' +
              keys.join(',') + '|' +
              values.join(',') + '|' +
              firstEntry[0] + ':' + firstEntry[1] + ':' + lastEntry[0] + ':' + lastEntry[1] + '|' +
              ownA + ':' + ownZ + ':' + ownD;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "3:2:4:5:v|a,b,c,d,value|3,2,4,5,v|a:3:value:v|true:false:true",
    )?;
    Ok(())
}

#[test]
fn object_property_access_missing_key_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { ok: 'yes' };
            document.getElementById('result').textContent =
              obj.missing + ':' + (typeof obj.missing) + ':' + obj.ok;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:undefined:yes")?;
    Ok(())
}

#[test]
fn member_call_expression_on_nested_object_path_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const api = {
              a: {
                b: {
                  method: (x, y) => x + y,
                  tag: 'ok'
                }
              }
            };
            const first = api.a.b.method(2, 3);
            const second = api.a.b.method(10, -4);
            document.getElementById('result').textContent =
              api.a.b.tag + ':' + first + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok:5:6")?;
    Ok(())
}

#[test]
fn member_call_expression_reports_non_function_target() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const api = { a: { b: { method: 1 } } };
            api.a.b.method('x');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("member call on non-function should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("'method' is not a function")),
        other => panic!("unexpected member call error: {other:?}"),
    }
    Ok(())
}

#[test]
fn object_method_runtime_type_errors_are_reported() -> Result<()> {
    let html = r#"
        <button id='keys'>keys</button>
        <button id='own'>own</button>
        <script>
          document.getElementById('keys').addEventListener('click', () => {
            Object.keys(1);
          });
          document.getElementById('own').addEventListener('click', () => {
            const x = 1;
            x.hasOwnProperty('a');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let keys_err = h
        .click("#keys")
        .expect_err("Object.keys should reject non-object argument");
    match keys_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Object.keys argument must be an object"))
        }
        other => panic!("unexpected Object.keys error: {other:?}"),
    }

    let own_err = h
        .click("#own")
        .expect_err("hasOwnProperty should reject non-object receiver");
    match own_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("is not an object")),
        other => panic!("unexpected hasOwnProperty error: {other:?}"),
    }

    Ok(())
}

#[test]
fn array_literal_and_basic_mutation_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2];
            const isArray1 = Array.isArray(arr);
            const isArray2 = window.Array.isArray('x');
            const lenBefore = arr.length;
            const first = arr[0];
            const pushed = arr.push(3, 4);
            const popped = arr.pop();
            const shifted = arr.shift();
            const unshifted = arr.unshift(9);
            document.getElementById('result').textContent =
              isArray1 + ':' + isArray2 + ':' + lenBefore + ':' + first + ':' +
              pushed + ':' + popped + ':' + shifted + ':' + unshifted + ':' + arr.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:2:1:4:4:1:3:9,2,3")?;
    Ok(())
}

#[test]
fn array_map_filter_and_reduce_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2, 3, 4];
            const mapped = arr.map((value, index) => value * 2 + index);
            const filtered = mapped.filter(value => value > 5);
            const sum = filtered.reduce((acc, value) => acc + value, 0);
            const sumNoInitial = filtered.reduce((acc, value) => acc + value);
            document.getElementById('result').textContent =
              mapped.join(',') + '|' + filtered.join(',') + '|' + sum + '|' + sumNoInitial;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2,5,8,11|8,11|19|19")?;
    Ok(())
}

#[test]
fn array_foreach_find_some_every_and_includes_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [2, 4, 6];
            let total = 0;
            arr.forEach((value, idx) => {
              total += value + idx;
            });
            const found = arr.find(value => value > 3);
            const some = arr.some(value => value === 4);
            const every = arr.every(value => value % 2 === 0);
            const includesDirect = arr.includes(4);
            const includesFrom = arr.includes(2, 1);
            document.getElementById('result').textContent =
              total + ':' + found + ':' + some + ':' + every + ':' +
              includesDirect + ':' + includesFrom;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "15:4:true:true:true:false")?;
    Ok(())
}

#[test]
fn array_slice_splice_and_join_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2, 3, 4];
            const firstSlice = arr.slice(1, 3);
            const secondSlice = arr.slice(-2);
            const removed = arr.splice(1, 2, 9, 8);
            document.getElementById('result').textContent =
              firstSlice.join(',') + '|' + secondSlice.join(',') + '|' +
              removed.join(',') + '|' + arr.join('-');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2,3|3,4|2,3|1-9-8-4")?;
    Ok(())
}

#[test]
fn reduce_empty_array_without_initial_value_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [];
            arr.reduce((acc, value) => acc + value);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("reduce without initial on empty array should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("reduce of empty array with no initial value"))
        }
        other => panic!("unexpected reduce error: {other:?}"),
    }
    Ok(())
}
