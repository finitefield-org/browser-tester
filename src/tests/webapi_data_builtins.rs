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

    let unicode_quantifier_parse_err = Harness::from_html("<script>const re = /(?=a)+/u;</script>")
        .expect_err("quantified lookahead in unicode mode should fail during parse");
    match unicode_quantifier_parse_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode quantifier parse error: {other:?}"),
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

    let html_unicode = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            new RegExp('(?=a)+', 'u');
          });
        </script>
        "#;
    let mut h_unicode = Harness::from_html(html_unicode)?;
    let runtime_unicode_err = h_unicode
        .click("#btn")
        .expect_err("quantified lookahead in unicode mode should fail at runtime");
    match runtime_unicode_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode quantifier runtime error: {other:?}"),
    }

    let quantifier_parse_err = Harness::from_html("<script>const re = /\\b+/;</script>")
        .expect_err("word boundary quantified regex should fail during parse");
    match quantifier_parse_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected quantifier parse error: {other:?}"),
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
            const escapedCompat =
              RegExp.escape('foo') + ':' +
              RegExp.escape(' a-b') + ':' +
              RegExp.escape('\u00A0') + ':' +
              RegExp.escape('\u2028') + ':' +
              RegExp.escape('_x') + ':' +
              RegExp.escape('\n') + ':' +
              RegExp.escape('[]') + ':' +
              RegExp.escape('/');
            document.getElementById('result').textContent =
              info + '|' + escaped + '|' + escapedWindow + '|' + escapedCompat;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "a.:gimsydu:true:true:true:true:true:true:true:false:3:true:function|\\x61\\+b\\*c\\?|\\x78\\.y|\\x66oo:\\x20a\\x2db:\\xa0:\\u2028:_x:\\n:\\[\\]:\\/",
    )?;
    Ok(())
}

#[test]
fn regexp_v_flag_and_unicode_sets_property_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const literal = /a/v;
            const ctor = new RegExp('a', 'v');
            const uvError = (() => {
              try {
                new RegExp('a', 'uv');
                return 'noerr';
              } catch (err) {
                return String(err).includes('invalid regular expression flags');
              }
            })();
            document.getElementById('result').textContent =
              literal.test('a') + ':' +
              literal.unicode + ':' + literal.unicodeSets + ':' +
              ctor.unicode + ':' + ctor.unicodeSets + ':' +
              uvError;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true:false:true:true")?;

    let parse_err = Harness::from_html("<script>const re = /a/uv;</script>")
        .expect_err("u and v flags together should fail during parse");
    match parse_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression flags")),
        other => panic!("unexpected uv flag parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_v_flag_scalar_class_set_operations_follow_js_constraints() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /[a&&a]/v.test('a');
            const b = /[a&&b]/v.test('a');
            const c = /[a--b]/v.test('a');
            const d = /[a--a]/v.test('a');
            const e = /[a&&a&&a]/v.test('a');
            const f = /[a--b--c]/v.test('a');
            const g = /[\d&&\w]/v.test('1') && !/[\d&&\w]/v.test('a');
            const h = /[\d--\w]/v.test('1');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' +
              e + ':' + f + ':' + g + ':' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true:false:true:true:true:false")?;

    let multi_item_err = Harness::from_html("<script>const re = /[ab&&c]/v;</script>")
        .expect_err("set operands with multiple items should fail in v mode");
    match multi_item_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected multi-item set operand parse error: {other:?}"),
    }

    let range_operand_err = Harness::from_html("<script>const re = /[a-b&&b]/v;</script>")
        .expect_err("range operand in set operation should fail in v mode");
    match range_operand_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected range set operand parse error: {other:?}"),
    }

    let mixed_op_err = Harness::from_html("<script>const re = /[a&&b--c]/v;</script>")
        .expect_err("mixing set operators at one level should fail in v mode");
    match mixed_op_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected mixed set operator parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_v_flag_nested_class_set_operands_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /[[ab]&&[bc]]/v.test('b');
            const b = /[[ab]--[bc]]/v.test('a');
            const c = /[[ab]&&[[bc]--[c]]]/v.test('b');
            const d = /[[ab]--[[bc]&&[c]]]/v.test('b');
            const e = /[[a-b]&&b]/v.test('b');
            const f = /[[a-b]--b]/v.test('a');
            const g = /[a&&[bc]]/v.test('a');
            const h = /[[ab]&&c]/v.test('b');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' +
              e + ':' + f + ':' + g + ':' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true:true:true:false:false")?;

    let rhs_multi_item_err = Harness::from_html("<script>const re = /[a&&bc]/v;</script>")
        .expect_err("set operand with multiple RHS items should fail without nesting");
    match rhs_multi_item_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected RHS multi-item set operand parse error: {other:?}"),
    }

    let nested_rhs_multi_item_err =
        Harness::from_html("<script>const re = /[[ab]&&bc]/v;</script>")
            .expect_err("nested left operand and multi-item RHS should fail without nesting");
    match nested_rhs_multi_item_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected nested RHS multi-item parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_v_flag_class_string_disjunction_q_escape_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const aHit = /[\q{ab}]/v.exec('xxabyy');
            const a = aHit !== null && aHit[0] === 'ab';
            const bHit = /[\q{a|bc}]/v.exec('zzbc');
            const b = bHit !== null && bHit[0] === 'bc';
            const c = /[\q{a\|b}]/v.test('a|b');
            const d = /[\q{a\}b}]/v.test('a}b');
            const e = /[\q{\x41}]/v.test('A');
            const fHit = /[\q{}]/v.exec('x');
            const f = fHit !== null && fHit[0] === '';
            const g = /[^\q{a|b}]/v.test('c') && !/[^\q{a|b}]/v.test('a');
            const h = /[\q{a|b}&&[ab]]/v.test('a') && /[\q{a|b}--\q{a}]/v.test('b');
            const iHit = /[\q{ab}--\q{a}]/v.exec('xxabyy');
            const i = iHit !== null && iHit[0] === 'ab';
            const j = /[[\q{ab}]&&\q{ab|x}]/v.test('ab');
            const kHit = /[[\q{ab}]--\q{a}]/v.exec('ab');
            const k = kHit !== null && kHit[0] === 'ab';
            const lHit = /[\q{ab|abc}]/v.exec('abcd');
            const l = lHit !== null && lHit[0] === 'abc';
            const mHit = /[[\q{ab}]]/v.exec('zab');
            const m = mHit !== null && mHit[0] === 'ab';
            const nHit = /[\p{RGI_Emoji}\q{🙂🙂}]/v.exec('🙂🙂');
            const n = nHit !== null && nHit[0] === '🙂🙂';
            const oHit = /[\q{🙂🙂}\p{RGI_Emoji}]/v.exec('🙂🙂');
            const o = oHit !== null && oHit[0] === '🙂🙂';
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' +
              e + ':' + f + ':' + g + ':' + h + ':' +
              i + ':' + j + ':' + k + ':' + l + ':' + m + ':' + n + ':' + o;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:true:true:true:true:true:true:true:true:true:true",
    )?;

    let missing_brace_err = Harness::from_html("<script>const re = /[\\q]/v;</script>")
        .expect_err("q escape in v mode requires class string disjunction braces");
    match missing_brace_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected q missing brace parse error: {other:?}"),
    }

    let invalid_escape_err = Harness::from_html("<script>const re = /[\\q{\\d}]/v;</script>")
        .expect_err("class string disjunction should reject class escapes like \\d");
    match invalid_escape_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected q invalid class escape parse error: {other:?}"),
    }

    let invalid_decimal_err = Harness::from_html("<script>const re = /[\\q{\\1}]/v;</script>")
        .expect_err("class string disjunction should reject decimal escapes");
    match invalid_decimal_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected q invalid decimal escape parse error: {other:?}"),
    }

    let negated_string_err = Harness::from_html("<script>const re = /[^\\q{ab}]/v;</script>")
        .expect_err("negated class should reject non-scalar string disjunction alternatives");
    match negated_string_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected q negated string parse error: {other:?}"),
    }

    let negated_set_string_err =
        Harness::from_html("<script>const re = /[^\\q{ab}--\\q{a}]/v;</script>")
            .expect_err("negated set operation should reject non-scalar string alternatives");
    match negated_set_string_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected q negated set string parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_v_flag_unicode_string_property_set_operations_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /[\p{RGI_Emoji}&&\p{RGI_Emoji}]/v.test('🙂');
            const b = /[\p{RGI_Emoji}--\p{RGI_Emoji}]/v.test('🙂');
            const c = /[\p{RGI_Emoji}--\q{🙂}]/v.test('👨‍👩‍👧‍👦');
            const d = /[\p{RGI_Emoji}--\q{🙂}]/v.test('🙂');
            const e = /[\q{🙂}&&\p{RGI_Emoji}]/v.test('🙂');
            const f = /[[\p{RGI_Emoji}]&&\p{RGI_Emoji}]/v.test('👨‍👩‍👧‍👦');
            const g = /[[\p{RGI_Emoji}]--\q{🙂}]/v.test('🙂');
            const h = /[[\p{RGI_Emoji}]--\q{🙂}]/v.test('👨‍👩‍👧‍👦');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' +
              e + ':' + f + ':' + g + ':' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true:false:true:true:false:true")?;

    let negated_set_err =
        Harness::from_html("<script>const re = /[^\\p{RGI_Emoji}&&\\p{RGI_Emoji}]/v;</script>")
            .expect_err("negated set containing unicode string properties should fail");
    match negated_set_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected negated unicode string set parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_flag_modifier_groups_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a1 = /(?i:a)/.test('A');
            const a2 = /(?-i:a)/.test('A');
            const a3 = /(?i:a)(?-i:b)/.test('Ab');
            const a4 = /(?m:^a$)/.test('x\na\ny');
            const a5 = /(?s:.)/.test('\n');
            const a6 = /(?i-:a)/.test('A');
            document.getElementById('result').textContent =
              a1 + ':' + a2 + ':' + a3 + ':' + a4 + ':' + a5 + ':' + a6;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true:true:true:true")?;

    let dup_flag_err = Harness::from_html("<script>const re = /(?ii:a)/;</script>")
        .expect_err("duplicate modifier flags should fail during parse");
    match dup_flag_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected duplicate flag parse error: {other:?}"),
    }

    let empty_disable_err = Harness::from_html("<script>const re = /(?-:a)/;</script>")
        .expect_err("invalid empty disable modifier should fail during parse");
    match empty_disable_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected invalid modifier parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_last_index_inside_surrogate_pair_behaves_like_js_search_and_sticky() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const s = '🙂a';

            const gSearch = /a/g;
            gSearch.lastIndex = 1;
            const mSearch = gSearch.exec(s);

            const gEmoji = /🙂/g;
            gEmoji.lastIndex = 1;
            const mEmoji = gEmoji.exec(s);

            const sticky = /a/y;
            sticky.lastIndex = 1;
            const mSticky = sticky.exec(s);

            const uStickyDot = /./uy;
            uStickyDot.lastIndex = 1;
            const mUStickyDot = uStickyDot.exec(s);

            const uSearch = /a/ug;
            uSearch.lastIndex = 1;
            const mUSearch = uSearch.exec(s);

            const uLow = /\uDE42/ug;
            uLow.lastIndex = 1;
            const mULow = uLow.exec(s);

            const uStickyA = /a/uy;
            uStickyA.lastIndex = 1;
            const mUStickyA = uStickyA.exec(s);

            document.getElementById('result').textContent =
              mSearch[0] + ':' + mSearch.index + ':' + gSearch.lastIndex + '|' +
              String(mEmoji === null) + ':' + gEmoji.lastIndex + '|' +
              String(mSticky === null) + ':' + sticky.lastIndex + '|' +
              mUStickyDot[0] + ':' + mUStickyDot.index + ':' + uStickyDot.lastIndex + '|' +
              mUSearch[0] + ':' + mUSearch.index + ':' + uSearch.lastIndex + '|' +
              String(mULow === null) + ':' + uLow.lastIndex + '|' +
              String(mUStickyA === null) + ':' + uStickyA.lastIndex;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:2:3|true:0|true:0|🙂:0:2|a:2:3|true:0|true:0")?;
    Ok(())
}

#[test]
fn regexp_global_sticky_string_match_respects_sticky_and_resets_last_index() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const m1 = 'ba'.match(/a/gy);
            const m2 = 'ba'.match(/a/g);
            const re = /a/gy;
            re.lastIndex = 1;
            const m3 = 'aa'.match(re);
            document.getElementById('result').textContent =
              String(m1 === null) + ':' + m2[0] + ':' + m3.length + ':' + re.lastIndex;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:a:2:0")?;
    Ok(())
}

#[test]
fn regexp_replace_respects_sticky_and_global_last_index_semantics() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const s = 'za';
            const sticky = /a/y;
            sticky.lastIndex = 1;
            const out1 = s.replace(sticky, 'X');
            const li1 = sticky.lastIndex;

            sticky.lastIndex = 0;
            const out2 = s.replace(sticky, 'X');
            const li2 = sticky.lastIndex;

            const globalSticky = /a/gy;
            globalSticky.lastIndex = 1;
            const out3 = s.replace(globalSticky, 'X');
            const li3 = globalSticky.lastIndex;

            document.getElementById('result').textContent =
              out1 + ':' + li1 + '|' + out2 + ':' + li2 + '|' + out3 + ':' + li3;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "zX:2|za:0|za:0")?;
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
fn regexp_backreference_and_named_groups_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const numericOk = /([a-z]+)-\1/.test('abc-abc');
            const numericNg = /([a-z]+)-\1/.test('abc-abd');
            const named = /(?<word>[a-z]+):\k<word>/.exec('go:go');
            document.getElementById('result').textContent =
              numericOk + ':' + numericNg + ':' + named[0] + ':' + named[1];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:go:go:go")?;
    Ok(())
}

#[test]
fn regexp_named_backreference_forward_reference_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const hit = /\k<a>(?<a>x)/.exec('xx');
            const hit2 = /(?<a>x)\k<a>/.exec('xx');
            document.getElementById('result').textContent =
              hit[0] + ':' + hit.groups.a + '|' + hit2[0] + ':' + hit2.groups.a;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "x:x|xx:x")?;
    Ok(())
}

#[test]
fn regexp_duplicate_named_groups_across_alternatives_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const r = /(?:(?<a>x)|(?<a>y))\k<a>/;
            const m1 = r.exec('xx');
            const m2 = r.exec('yy');
            const m3 = /(?<a>x)|(?<a>y)/.exec('y');

            const ok1 =
              m1 !== null &&
              m1[0] === 'xx' &&
              m1[1] === 'x' &&
              m1[2] === undefined &&
              m1.groups.a === 'x';
            const ok2 =
              m2 !== null &&
              m2[0] === 'yy' &&
              m2[1] === undefined &&
              m2[2] === 'y' &&
              m2.groups.a === 'y';
            const ok3 = !r.test('xy');
            const ok4 =
              m3 !== null &&
              m3[0] === 'y' &&
              m3[1] === undefined &&
              m3[2] === 'y' &&
              m3.groups.a === 'y';

            document.getElementById('result').textContent =
              ok1 + ':' + ok2 + ':' + ok3 + ':' + ok4;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true")?;

    let same_path_err = Harness::from_html("<script>const re = /(?<a>x)(?<a>y)/;</script>")
        .expect_err("duplicate named groups in one path should fail");
    match same_path_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected duplicate same-path parse error: {other:?}"),
    }

    let partial_overlap_err =
        Harness::from_html("<script>const re = /(?:(?<a>x)|y)(?<a>z)/;</script>")
            .expect_err("duplicate names should fail when any alternative path overlaps");
    match partial_overlap_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected duplicate overlapping-path parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_named_group_identifier_name_compat_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /(?<$>x)\k<$>/.exec('xx');
            const b = /(?<é>x)\k<é>/.exec('xx');
            const c = /(?<\u0061>x)\k<a>/.exec('xx');
            const d = /(?<a>x)\k<\u0061>/.exec('xx');
            const e = /(?<a\u0301>x)\k<a\u0301>/.exec('xx');
            const f = /(?<\u{62}>y)\k<b>/.exec('yy');
            const ok =
              a !== null && a.groups.$ === 'x' &&
              b !== null && b.groups['é'] === 'x' &&
              c !== null && c.groups.a === 'x' &&
              d !== null && d.groups.a === 'x' &&
              e !== null && e.groups['á'] === 'x' &&
              f !== null && f.groups.b === 'y';
            document.getElementById('result').textContent = String(ok);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true")?;

    let err = Harness::from_html("<script>const re = /(?<😀>x)/;</script>")
        .expect_err("emoji should be rejected as capture group name");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode emoji group-name parse error: {other:?}"),
    }

    let err_ref = Harness::from_html("<script>const re = /(?<a>x)\\k<😀>/;</script>")
        .expect_err("emoji should be rejected as named backreference name");
    match err_ref {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode emoji backreference parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_named_backreference_literals_without_named_groups_in_non_unicode_mode() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /\k/.test('k');
            const b = /\k<a>/.test('k<a>');
            const c = /\k<->/.test('k<->');
            const d = /\k<>/.test('k<>');
            const e = /\k<\u0061>/.test('k<a>');
            const f = /\k<\x41>/.test('k<A>');
            const g = /\k<\07>/.test('k<' + String.fromCharCode(7) + '>');
            const h = /\k<\cA>/.test('k<' + String.fromCharCode(1) + '>');
            const i = /\k<\>/.test('k<>');
            const j = /\k<\\>/.test('k<\\>');
            const k = /\k<a{2}>/.test('k<aa>') && !/\k<a{2}>/.test('k<a{2}>');
            const l = /\k<\u{61}>/.test('k<' + 'u'.repeat(61) + '>');
            const m = /\k<(a)>/.exec('k<a>');
            const n =
              m !== null &&
              m[0] === 'k<a>' &&
              m[1] === 'a' &&
              /\k<(?:(a))>/.exec('k<a>')[1] === 'a';
            const o = /\1\k<(a)>/.exec('k<a>');
            const p =
              o !== null &&
              o[0] === 'k<a>' &&
              o[1] === 'a' &&
              /\1\k<(a)>\1/.test('k<a>a') &&
              !/\1\k<(a)>\1/.test('k<a>');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' +
              e + ':' + f + ':' + g + ':' + h + ':' + i + ':' + j + ':' +
              k + ':' + l + ':' + n + ':' + p;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:true:true:true:true:true:true:true:true:true",
    )?;

    let err_complex_named = Harness::from_html("<script>const re = /\\k<(?<a>x)>/;</script>")
        .expect_err("legacy \\k<...> fallback should still reject named capture syntax");
    match err_complex_named {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected complex named fallback parse error: {other:?}"),
    }
    Ok(())
}

#[test]
fn regexp_named_backreference_to_unknown_name_is_parse_error() -> Result<()> {
    let err = Harness::from_html("<script>const re = /\\k<a>(?<b>x)/;</script>")
        .expect_err("unknown named backreference should fail during parse");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unknown named backreference parse error: {other:?}"),
    }

    let err_u = Harness::from_html("<script>const re = /\\k<a>/u;</script>")
        .expect_err("named backreference syntax should be strict in unicode mode");
    match err_u {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode named backreference parse error: {other:?}"),
    }

    let err_invalid_name_with_named_capture =
        Harness::from_html("<script>const re = /(?<a>x)\\k<->/;</script>")
            .expect_err("invalid named backreference name should fail when named groups exist");
    match err_invalid_name_with_named_capture {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected invalid name parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_numeric_backreference_and_legacy_octal_behavior_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const c1 = String.fromCharCode(1);
            const c2 = String.fromCharCode(2);
            const c8 = String.fromCharCode(8);

            const a = /\1(a)/.exec('a');
            const b = /(a)\2/.test('a' + c2);
            const c = /(a)\18/.test('a' + c1 + '8');
            const d = /(a)\8/.test('a8');
            const e = /[\1]/.test(c1);
            const f = /[\10]/.test(c8);
            const g = /[\18]/.test(c1) && /[\18]/.test('8');

            document.getElementById('result').textContent =
              a[0] + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:true:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn regexp_numeric_backreference_invalid_in_unicode_mode_is_parse_error() -> Result<()> {
    let err1 = Harness::from_html("<script>const re = /\\2(a)/u;</script>")
        .expect_err("numeric backreference beyond capture count should fail in unicode mode");
    match err1 {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode numeric backreference parse error: {other:?}"),
    }

    let err2 = Harness::from_html("<script>const re = /[\\1]/u;</script>")
        .expect_err("numeric class escape should fail in unicode mode");
    match err2 {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode class numeric escape parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_legacy_brace_and_empty_class_compat_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const braceLiteral = /a{b}/.test('a{b}');
            const braceLiteral2 = /a{1,a}/.test('a{1,a}');
            const emptyClass = /[]/.test('a');
            const negEmpty = /[^]/.test('\n');
            const octal = /\07/.test('\x07');
            const nul8 = /\08/.test(String.fromCharCode(0) + '8');
            const classOctal = /[\07]/.test('\x07');
            const classNulOrEight =
              /[\08]/.test(String.fromCharCode(0)) && /[\08]/.test('8');
            document.getElementById('result').textContent =
              braceLiteral + ':' + braceLiteral2 + ':' +
              emptyClass + ':' + negEmpty + ':' +
              octal + ':' + nul8 + ':' + classOctal + ':' + classNulOrEight;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn regexp_class_range_order_and_unicode_decimal_escape_errors_match_js() -> Result<()> {
    let class_range_err = Harness::from_html("<script>const re = /[z-a]/;</script>")
        .expect_err("range out of order should fail during parse");
    match class_range_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected class range parse error: {other:?}"),
    }

    let unicode_decimal_err = Harness::from_html("<script>const re = /\\07/u;</script>")
        .expect_err("unicode decimal escape should fail during parse");
    match unicode_decimal_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode decimal parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_unicode_mode_rejects_invalid_identity_escapes() -> Result<()> {
    let err1 = Harness::from_html("<script>const re = /\\a/u;</script>")
        .expect_err("unicode mode should reject identity escapes like \\a");
    match err1 {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode identity escape parse error: {other:?}"),
    }

    let err2 = Harness::from_html("<script>const re = /[\\q]/u;</script>")
        .expect_err("unicode mode should reject class identity escapes like \\q");
    match err2 {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode class identity parse error: {other:?}"),
    }

    let err3 = Harness::from_html("<script>const re = /\\-/u;</script>")
        .expect_err("unicode mode should reject escaped hyphen outside class");
    match err3 {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode escaped hyphen parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_unicode_escape_surrogate_patterns_parse_like_js() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /\uD800/.test('x');
            const b = /\uD800/u.test('x');
            const c = /[\uD800]/.test('x');
            const d = /[\uD800]/u.test('x');
            const e = /\u{D800}/u.test('x');
            const loneHigh = String.fromCharCode(0xD800);
            const loneLow = String.fromCharCode(0xDFFF);
            const f = /\uD800/.test(loneHigh);
            const g = /\uD800/u.test(loneHigh);
            const h = /[\uD800]/.test(loneHigh);
            const i = /[\uD800]/u.test(loneHigh);
            const j = /\u{D800}/u.test(loneHigh);
            const k = /\uDFFF/.test(loneLow);
            const l = /\uDFFF/u.test(loneLow);
            const m = /\u{DFFF}/u.test(loneLow);
            const n = /\uD800/.test('\uD800');
            const o = /\uD800/u.test('\uD800');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' +
              f + ':' + g + ':' + h + ':' + i + ':' + j + ':' +
              k + ':' + l + ':' + m + ':' + n + ':' + o;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "false:false:false:false:false:true:true:true:true:true:true:true:true:true:true",
    )?;

    let err = Harness::from_html("<script>const re = /\\u{110000}/u;</script>")
        .expect_err("unicode code point escape beyond 0x10FFFF should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected out-of-range unicode escape parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_unicode_property_escapes_and_non_unicode_u_escape_compat_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /\p{L}/u.test('A');
            const b = /\P{L}/u.test('1');
            const c = /\p{Nd}/u.test('9') && /\p{Nd}/u.test('٣');
            const d = /\p{ASCII}/u.test('A') && !/\p{ASCII}/u.test('🙂');
            const e = /\p{Any}/u.test('🙂');
            const f = /\p{gc=Letter}/u.test('Z');
            const g = /\p{sc=Latin}/u.test('A') && !/\p{sc=Latin}/u.test('Γ');
            const g2 = /\p{sc=Greek}/u.test('Γ') && !/\p{sc=Greek}/u.test('A');
            const g3 = /\p{Script=Greek}/u.test('Ω');
            const g4 = /\p{scx=Greek}/u.test('Γ') && !/\p{scx=Greek}/u.test('A');
            const g5 = /\p{Script_Extensions=Latin}/u.test('A');
            const h = /\p{L}/.test('p{L}');
            const i = /\u{61}/.test('u'.repeat(61));
            const j = /\u{61}/u.test('a');
            const k = /\p{RGI_Emoji}/v.test('🙂');
            const l = /[\p{RGI_Emoji}]/v.test('🙂');
            const m = /\p{RGI_Emoji}/v.test('A');
            const n = /\p{RGI_Emoji}/v.test('👨‍👩‍👧‍👦');
            const o = /\p{RGI_Emoji}/v.test('0️⃣');
            const p = /[\p{RGI_Emoji}a]/v.test('a');
            const q = /[\p{RGI_Emoji}\p{ASCII}]/v.exec('0️⃣')[0] === '0️⃣';
            const cpEmoji = String.fromCodePoint(0x1F642);
            const r = /\p{RGI_Emoji}/v.test(cpEmoji) && /[\p{RGI_Emoji}]/v.test(cpEmoji);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' +
              f + ':' + g + ':' + g2 + ':' + g3 + ':' + g4 + ':' + g5 + ':' + h + ':' + i + ':' + j + ':' +
              k + ':' + l + ':' + m + ':' + n + ':' + o + ':' + p + ':' + q + ':' + r;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:false:true:true:true:true:true",
    )?;

    let err = Harness::from_html("<script>const re = /\\p{RGI_Emoji}/u;</script>")
        .expect_err("unsupported unicode property name should fail in unicode mode");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected unicode property parse error: {other:?}"),
    }

    let negated_err = Harness::from_html("<script>const re = /\\P{RGI_Emoji}/v;</script>")
        .expect_err("negated unicode string property should fail in unicode sets mode");
    match negated_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected negated unicode string property parse error: {other:?}"),
    }

    let negated_class_err = Harness::from_html("<script>const re = /[^\\p{RGI_Emoji}]/v;</script>")
        .expect_err("negated class containing unicode string property should fail");
    match negated_class_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected negated class unicode string parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_unicode_property_aliases_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /\p{sc=Latn}/u.test('A') && !/\p{sc=Latn}/u.test('Γ');
            const b = /\p{Script=Latn}/u.test('A') && !/\p{Script=Latn}/u.test('Γ');
            const c = /\p{sc=Grek}/u.test('Γ') && !/\p{sc=Grek}/u.test('A');
            const d = /\p{gc=Lu}/u.test('A') && !/\p{gc=Lu}/u.test('a');
            const e = /\p{Lu}/u.test('Z') && !/\p{Lu}/u.test('z');
            const f = /\p{General_Category=Lowercase_Letter}/u.test('z') &&
                      !/\p{General_Category=Lowercase_Letter}/u.test('Z');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn regexp_control_escape_sequences_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const ctrlA = String.fromCharCode(1);
            const ctrl0 = String.fromCharCode(16);
            const ctrl1 = String.fromCharCode(17);
            const ctrlU = String.fromCharCode(31);
            const ok1 = /\cA/.test(ctrlA);
            const ok2 = /[\cA]/.test(ctrlA);
            const ok3 = /\c1/.test('\\c1') && !/\c1/.test('c1');
            const ok4 = /[\c1]/.test(ctrl1) && !/[\c1]/.test('1') && !/[\c1]/.test('c');
            const ok5 = /[\c_]/.test(ctrlU) && !/[\c_]/.test('_');
            const ok6 = /[\c0]/.test(ctrl0) && !/[\c0]/.test('0');
            const ok7 = /[\c*]/.test('\\') && /[\c*]/.test('c') && /[\c*]/.test('*');
            const ok8 = /\c/.test('\\c') && !/\c/.test('c');
            document.getElementById('result').textContent =
              ok1 + ':' + ok2 + ':' + ok3 + ':' + ok4 + ':' +
              ok5 + ':' + ok6 + ':' + ok7 + ':' + ok8;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn regexp_space_and_not_space_follow_ecmascript_whitespace_set() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /\s/.test('\uFEFF');
            const b = /\S/.test('\uFEFF');
            const c = /\s/.test('\u0085');
            const d = /\S/.test('\u0085');
            const e = /\s/.test('\u1680');
            const f = /\s/.test('\u2009');
            const g = /\s/.test('\u2029');
            const h = /\s/.test('\u200B');
            const i = /\S/.test('\u200B');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' +
              e + ':' + f + ':' + g + ':' + h + ':' + i;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:false:true:true:true:true:false:true")?;
    Ok(())
}

#[test]
fn regexp_ignore_case_unicode_canonicalization_matches_js() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /K/i.test('k');
            const b = /K/iu.test('k');
            const c = /ſ/i.test('s');
            const d = /ſ/iu.test('s');
            const e = /ß/i.test('ẞ');
            const f = /ß/iu.test('ẞ');
            const g = /Σ/i.test('ς');
            const h = /\w/i.test('K');
            const i = /\w/iu.test('K');
            const j = /\w/iu.test('ſ');
            const k = /\bK/i.test('K!');
            const l = /\bK/iu.test('K!');
            const m = /\bſ/i.test('ſ!');
            const n = /\bſ/iu.test('ſ!');
            const o = /[a-z]/i.test('ſ');
            const p = /[a-z]/iu.test('ſ');
            const q = /[a-z]/i.test('K');
            const r = /[a-z]/iu.test('K');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' +
              e + ':' + f + ':' + g + ':' + h + ':' +
              i + ':' + j + ':' + k + ':' + l + ':' +
              m + ':' + n + ':' + o + ':' + p + ':' + q + ':' + r;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "false:true:false:true:false:true:true:false:true:true:false:true:false:true:false:true:false:true",
    )?;
    Ok(())
}

#[test]
fn regexp_lookbehind_positive_and_negative_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const positiveOk = /(?<=foo)bar/.test('foobar');
            const positiveNg = /(?<=foo)bar/.test('xxbar');
            const negativeOk = /(?<!foo)bar/.test('xxbar');
            const negativeNg = /(?<!foo)bar/.test('foobar');
            const hit = /(?<=foo)bar/.exec('foobar');
            document.getElementById('result').textContent =
              positiveOk + ':' + positiveNg + ':' + negativeOk + ':' + negativeNg + ':' + hit[0];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true:false:bar")?;
    Ok(())
}

#[test]
fn regexp_lookaround_captures_propagate_to_following_pattern() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /(?=(a+))\1/.exec('aa');
            const b = /(?=(a+))a*b\1/.exec('baabac');
            const c = /(?<=(foo))bar/.exec('foobar');
            const d = /(?!([a-z]+))\1/.exec('123');
            const e = /(?<!([a-z]+))\1/.exec('123');
            document.getElementById('result').textContent =
              a[0] + ':' + a[1] + '|' +
              b[0] + ':' + b[1] + ':' + b.index + '|' +
              c[0] + ':' + c[1] + '|' +
              String(d[1] === undefined) + ':' + String(e[1] === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "aa:aa|aba:a:2|bar:foo|true:true")?;
    Ok(())
}

#[test]
fn regexp_lookbehind_capture_order_matches_js_backward_evaluation() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /(?<=([ab]+)([bc]+))$/.exec('abc');
            const b = /(?<=([ab]+)([bc]+))c/.exec('abc');
            document.getElementById('result').textContent =
              a[0].length + ':' + a[1] + ':' + a[2] + ':' + a.index + '|' +
              b[0] + ':' + b[1] + ':' + b[2] + ':' + b.index;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:a:bc:3|c:a:b:2")?;
    Ok(())
}

#[test]
fn regexp_lookbehind_greedy_and_lazy_capture_selection_matches_js() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /(?<=([ab]+?)([bc]+))$/.exec('abc');
            const b = /(?<=([ab]+?)([bc]+?))$/.exec('abc');
            const c = /(?<=([ab]+)([bc]+?))$/.exec('abc');
            const d = /(?<=([ab]+)([bc]+))$/.exec('abbc');
            const e = /(?<=([ab]+)([bc]+?))$/.exec('abbc');
            document.getElementById('result').textContent =
              a[1] + ':' + a[2] + '|' +
              b[1] + ':' + b[2] + '|' +
              c[1] + ':' + c[2] + '|' +
              d[1] + ':' + d[2] + '|' +
              e[1] + ':' + e[2];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:bc|b:c|ab:c|a:bbc|abb:c")?;
    Ok(())
}

#[test]
fn regexp_lookbehind_backreference_behavior_matches_js() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = /(?<=([ab]+)\1)c/.exec('abc');
            const b = /(?<=([ab]+)\1)c/.exec('aabc');
            const c = /(?<=\1([ab]+))c/.exec('aabc');
            const d = /(?<=\1(a))b/.exec('ab');
            const e = /(?<=(a)\1)b/.exec('aab');
            const f = /(?<=([ab]+)([bc]+)\2)c/.exec('abcc');
            const g = /(?<=([ab]+)([bc]+)\1)c/.exec('abac');
            document.getElementById('result').textContent =
              a[0] + ':' + a[1] + ':' + a.index + '|' +
              b[0] + ':' + b[1] + ':' + b.index + '|' +
              String(c === null) + ':' + String(d === null) + '|' +
              e[0] + ':' + e[1] + ':' + e.index + '|' +
              f[0] + ':' + f[1] + ':' + f[2] + ':' + f.index + '|' +
              String(g === null);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "c:ab:2|c:aab:3|true:true|b:a:2|c:a:b:2|true")?;
    Ok(())
}

#[test]
fn regexp_quantifier_resets_unmatched_alternative_captures() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const m1 = /(?:(x)|(y))+/.exec('xy');
            const ok1 =
              m1 !== null &&
              m1[0] === 'xy' &&
              m1[1] === undefined &&
              m1[2] === 'y';

            const m2 = /((a)|(b))+/.exec('ab');
            const ok2 =
              m2 !== null &&
              m2[0] === 'ab' &&
              m2[1] === 'b' &&
              m2[2] === undefined &&
              m2[3] === 'b';

            const m3 = /(?:(?<a>x)|(?<a>y))+/.exec('xy');
            const ok3 =
              m3 !== null &&
              m3[1] === undefined &&
              m3[2] === 'y' &&
              m3.groups.a === 'y';

            const ok4 = /(?:(?<a>x)|(?<a>y))+\k<a>/.test('xyy');

            document.getElementById('result').textContent =
              ok1 + ':' + ok2 + ':' + ok3 + ':' + ok4;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true")?;
    Ok(())
}

#[test]
fn regexp_quantified_lookahead_works_but_lookbehind_quantifier_is_invalid() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const hit = /(?=a)+/.exec('a');
            const hit2 = /(?:)+/.exec('a');
            const hit3 = /(?:){2}/.exec('a');
            document.getElementById('result').textContent =
              hit[0].length + ':' + hit2[0].length + ':' + hit3[0].length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:0:0")?;

    let parse_err = Harness::from_html("<script>const re = /(?<=a)+/;</script>")
        .expect_err("quantified lookbehind should fail during parse");
    match parse_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected lookbehind quantifier parse error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_exec_unmatched_capture_is_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const hit = /(a)?b/.exec('b');
            document.getElementById('result').textContent =
              hit[0] + ':' + String(hit[1] === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "b:true")?;
    Ok(())
}

#[test]
fn regexp_exec_and_match_expose_index_input_and_groups() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const input = 'xx abc-42 yy';
            const hit = /(?<word>[a-z]+)-(\d+)/.exec(input);
            const matched = input.match(/(?<word>[a-z]+)-(\d+)/);
            const noNamed = /(ab)(\d)/.exec('zzab3');
            document.getElementById('result').textContent =
              hit[0] + ':' + hit.index + ':' + (hit.input === input) + ':' + hit.groups.word + '|' +
              matched[0] + ':' + matched.index + ':' + (matched.input === input) + ':' + matched.groups.word + '|' +
              String(noNamed.groups === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "abc-42:3:true:abc|abc-42:3:true:abc|true")?;
    Ok(())
}

#[test]
fn regexp_d_flag_exposes_indices_and_named_groups() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const input = 'xx abc-42 yy';
            const hit = /(?<word>[a-z]+)-(\d+)/d.exec(input);
            const matched = input.match(/(?<word>[a-z]+)-(\d+)/d);
            const optional = /(?<opt>a)?b/d.exec('b');
            const plain = /ab/.exec('zab');
            document.getElementById('result').textContent =
              hit.indices[0][0] + ':' + hit.indices[0][1] + ':' + hit.indices[1][0] + ':' + hit.indices[2][1] + ':' + hit.indices.groups.word[0] + '|' +
              matched.indices[0][0] + ':' + matched.indices.groups.word[0] + '|' +
              String(optional.indices[1] === undefined) + ':' + String(optional.indices.groups.opt === undefined) + '|' +
              String(plain.indices === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:9:3:9:3|3:3|true:true|true")?;
    Ok(())
}

#[test]
fn regexp_named_group_property_order_follows_source_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const hit = /(?<second>foo)(?<first>bar)/d.exec('xxfoobar');
            const keys = Object.keys(hit.groups).join(',');
            const idxKeys = Object.keys(hit.indices.groups).join(',');
            document.getElementById('result').textContent =
              keys + '|' + idxKeys + '|' +
              hit.groups.second + ':' + hit.groups.first + '|' +
              hit.indices.groups.second[0] + ':' + hit.indices.groups.first[0];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "second,first|second,first|foo:bar|2:5")?;
    Ok(())
}

#[test]
fn regexp_utf16_index_last_index_and_indices_use_code_units() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const s = '🙂a🙂a';
            const re = /a/dg;
            const first = re.exec(s);
            const li1 = re.lastIndex;
            const second = re.exec(s);
            const li2 = re.lastIndex;
            const third = re.exec(s);
            const li3 = re.lastIndex;
            const plain = /a/.exec('🙂a');
            const search = '🙂a'.search(/a/);
            document.getElementById('result').textContent =
              first.index + ':' + first.indices[0][0] + ':' + li1 + '|' +
              second.index + ':' + second.indices[0][0] + ':' + li2 + '|' +
              String(third === null) + ':' + li3 + '|' +
              plain.index + ':' + search;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:2:3|5:5:6|true:0|2:2")?;
    Ok(())
}

#[test]
fn regexp_replace_callback_offset_uses_utf16_code_units() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const s = '🙂a🙂a';
            const single = s.replace(/a/, (m, offset) => String(offset));
            const all = s.replace(/a/g, (m, offset) => String(offset));
            document.getElementById('result').textContent = single + '|' + all;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "🙂2🙂a|🙂2🙂5")?;
    Ok(())
}

#[test]
fn regexp_search_respects_sticky_and_restores_last_index() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sticky = /a/y;
            sticky.lastIndex = 2;
            const s1 = 'ba'.search(sticky);
            const li1 = sticky.lastIndex;

            const global = /a/g;
            global.lastIndex = 2;
            const s2 = 'ba'.search(global);
            const li2 = global.lastIndex;

            document.getElementById('result').textContent =
              s1 + ':' + li1 + '|' + s2 + ':' + li2;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "-1:2|1:2")?;
    Ok(())
}

#[test]
fn regexp_search_non_regex_argument_uses_regexp_semantics() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const s1 = 'a.b'.search('.');
            const s2 = '🙂a'.search('a');
            const re = /a/g;
            re.lastIndex = 1;
            const s3 = 'ba'.search(re);
            const li = re.lastIndex;
            document.getElementById('result').textContent =
              s1 + ':' + s2 + ':' + s3 + ':' + li;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:2:1:1")?;
    Ok(())
}

#[test]
fn regexp_replace_supports_prefix_suffix_and_named_tokens() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = 'xxfoobarzz'.replace(
              /(?<left>foo)(bar)/,
              '$`|$&|$\'|$1|$2|$<left>'
            );
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "xxxx|foobar|zz|foo|bar|foozz")?;
    Ok(())
}

#[test]
fn regexp_replace_out_of_range_tokens_are_literal_without_named_groups() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 'foo'.replace(/foo/, '$1');
            const b = 'foo'.replace(/foo/, '$2');
            const c = 'foo'.replace(/foo/, '$<x>');
            const d = 'foo'.replace(/(?<x>foo)/, '$<x>:$<y>');
            const e = 'bar'.replace(/(f)?bar/, '$1');
            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "$1|$2|$<x>|foo:|0")?;
    Ok(())
}

#[test]
fn regexp_replace_callback_receives_named_groups_object() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = 'foo:foo'.replace(
              /(?<word>[a-z]+):\k<word>/,
              (all, cap1, offset, input, groups) =>
                groups.word + ':' + (typeof groups) + ':' + offset + ':' + input
            );
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "foo:object:0:foo:foo")?;
    Ok(())
}

#[test]
fn regexp_split_includes_captured_groups() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parts = 'ab12cd'.split(/(\d+)/);
            document.getElementById('result').textContent =
              parts.length + ':' + parts[0] + ':' + parts[1] + ':' + parts[2];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:ab:12:cd")?;
    Ok(())
}

#[test]
fn regexp_split_zero_width_matches_follow_js_iteration_rules() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 'ab'.split(/(?:)/);
            const b = 'ab'.split(/()/);
            const c = ''.split(/(?:)/);
            const re = /()/g;
            re.lastIndex = 1;
            const d = 'ab'.split(re);
            const li = re.lastIndex;
            document.getElementById('result').textContent =
              a.join(',') + '|' +
              b.join(',') + '|' +
              c.length + '|' +
              d.join(',') + ':' + li;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a,b|a,,b|0|a,,b:1")?;
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
