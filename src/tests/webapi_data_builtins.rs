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
fn regexp_exec_accepts_missing_argument_and_coerces_inputs_to_string() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const missing = /undefined/.exec();
            const explicit = /undefined/.exec(undefined);
            const fromNumber = /42/.exec(42);
            const fromNull = /null/.exec(null);

            const global = /undefined/g;
            const g1 = global.exec();
            const li1 = global.lastIndex;
            const g2 = global.exec();
            const li2 = global.lastIndex;

            document.getElementById('result').textContent =
              missing[0] + ':' + explicit[0] + ':' + fromNumber[0] + ':' + fromNull[0] + '|' +
              g1[0] + ':' + li1 + ':' + String(g2 === null) + ':' + li2;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:undefined:42:null|undefined:9:true:0")?;
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
            const request = new Request('/api/message', {
              method: 'GET',
              headers: new Headers({ 'Accept': 'text/plain' }),
            });
            Promise.all([
              fetch('/api/message').then((response) => response.text()),
              window.fetch('/api/message').then((response) => response.text()),
              fetch(request).then((response) => response.text()),
              window.fetch('/api/message', { method: 'POST' }).then((response) => response.text()),
            ]).then((values) => {
              document.getElementById('result').textContent = values.join(':');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_fetch_mock("/api/message", "ok");
    h.click("#btn")?;
    h.assert_text("#result", "ok:ok:ok:ok")?;
    assert_eq!(
        h.take_fetch_calls(),
        vec![
            "/api/message".to_string(),
            "/api/message".to_string(),
            "/api/message".to_string(),
            "/api/message".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn fetch_without_mock_rejects_promise_with_type_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            fetch('/api/missing')
              .then(() => {
                document.getElementById('result').textContent = 'ok';
              })
              .catch((reason) => {
                document.getElementById('result').textContent = String(reason);
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "TypeError: Failed to fetch")?;
    Ok(())
}

#[test]
fn fetch_resolves_http_error_status_and_exposes_response_metadata() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            fetch('/api/missing')
              .then((response) => {
                document.getElementById('result').textContent =
                  response.ok + ':' + response.status + ':' + response.statusText;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_fetch_mock_response("/api/missing", 404, "not found");
    h.click("#btn")?;
    h.assert_text("#result", "false:404:Not Found")?;
    assert_eq!(h.take_fetch_calls(), vec!["/api/missing".to_string()]);
    Ok(())
}

#[test]
fn fetch_invalid_absolute_urls_reject_and_protocol_relative_uses_canonical_mock_key() -> Result<()>
{
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Promise.all([
              fetch('https://example.com:abc/api').catch((reason) => String(reason)),
              fetch('https://example.com:65536/api').catch((reason) => String(reason)),
              fetch('http://[::1/api').catch((reason) => String(reason)),
              fetch('//Example.COM:080/api/ok').then((response) => response.text())
            ]).then((values) => {
              document.getElementById('result').textContent = values.join('|');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.set_fetch_mock("https://example.com:80/api/ok", "ok");
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "TypeError: Invalid URL|TypeError: Invalid URL|TypeError: Invalid URL|ok",
    )?;
    assert_eq!(
        h.take_fetch_calls(),
        vec!["//Example.COM:080/api/ok".to_string()]
    );
    Ok(())
}

#[test]
fn fetch_special_host_edge_inputs_canonicalize_and_empty_host_reject_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Promise.all([
              fetch('http:Example.COM:080/api/root').then((response) => response.text()),
              fetch('http:\\Example.COM\\api\\backslash').then((response) => response.text()),
              fetch('//Example.COM\\api\\proto').then((response) => response.text()),
              fetch('http://').catch((reason) => String(reason)),
              fetch('http:?x').catch((reason) => String(reason)),
              fetch('http://?x').catch((reason) => String(reason))
            ]).then((values) => {
              document.getElementById('result').textContent = values.join('|');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.set_fetch_mock("http://example.com/api/root", "root");
    h.set_fetch_mock("http://example.com/api/backslash", "backslash");
    h.set_fetch_mock("https://example.com/api/proto", "proto");
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "root|backslash|proto|TypeError: Invalid URL|TypeError: Invalid URL|TypeError: Invalid URL",
    )?;
    assert_eq!(
        h.take_fetch_calls(),
        vec![
            "http:Example.COM:080/api/root".to_string(),
            "http:\\Example.COM\\api\\backslash".to_string(),
            "//Example.COM\\api\\proto".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn fetch_delimiter_inputs_use_canonical_mock_key_and_credentials_reject_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Promise.all([
              fetch('https://example.com\\docs\\a b?a\'b').then((response) => response.text()),
              fetch('https://a@b:p@q:r@example.com\\docs\\a b?a\'b').catch((reason) => String(reason))
            ]).then((values) => {
              document.getElementById('result').textContent = values.join('|');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.set_fetch_mock("https://example.com/docs/a%20b?a%27b", "ok");
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "ok|TypeError: URL with credentials is not allowed",
    )?;
    assert_eq!(
        h.take_fetch_calls(),
        vec!["https://example.com\\docs\\a b?a'b".to_string()]
    );
    Ok(())
}

#[test]
fn fetch_authority_and_percent_residuals_canonicalize_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Promise.all([
              fetch('https://ExA%41mple.org/%2f%zz?x=%2f%zz').then((response) => response.text()),
              fetch('foo://example.com/%2f%zz?x=%2f%zz').then((response) => response.text()),
              fetch('https://exa%mple.org/').catch((reason) => String(reason)),
              fetch('https://user:@example.com/%2f%zz').catch((reason) => String(reason))
            ]).then((values) => {
              document.getElementById('result').textContent = values.join('|');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.set_fetch_mock("https://exaample.org/%2f%zz?x=%2f%zz", "host-decoded");
    h.set_fetch_mock("foo://example.com/%2f%zz?x=%2f%zz", "custom");
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "host-decoded|custom|TypeError: Invalid URL|TypeError: URL with credentials is not allowed",
    )?;
    assert_eq!(
        h.take_fetch_calls(),
        vec![
            "https://ExA%41mple.org/%2f%zz?x=%2f%zz".to_string(),
            "foo://example.com/%2f%zz?x=%2f%zz".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn fetch_malformed_query_and_host_code_point_residuals_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Promise.all([
              fetch('https://\uFF21example.com/?a=%zz&b=%E0%A4&c=%C3%28').then((response) => response.text()),
              fetch('https://\u00E9xample.com/').then((response) => response.text()),
              fetch('https://%C3%A9xample.com/').then((response) => response.text()),
              fetch('https://%00example.com/').catch((reason) => String(reason))
            ]).then((values) => {
              document.getElementById('result').textContent = values.join('|');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.set_fetch_mock(
        "https://aexample.com/?a=%zz&b=%E0%A4&c=%C3%28",
        "query-host",
    );
    h.set_fetch_mock("https://xn--xample-9ua.com/", "idna-host");
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "query-host|idna-host|idna-host|TypeError: Invalid URL",
    )?;
    assert_eq!(
        h.take_fetch_calls(),
        vec![
            "https://\u{FF21}example.com/?a=%zz&b=%E0%A4&c=%C3%28".to_string(),
            "https://\u{E9}xample.com/".to_string(),
            "https://%C3%A9xample.com/".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn fetch_idna_invalid_label_and_dot_variant_residuals_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Promise.all([
              fetch('https://example\u3002com./').then((response) => response.text()),
              fetch('https://\u05D0.com/').then((response) => response.text()),
              fetch('https://xn--/').catch((reason) => String(reason)),
              fetch('https://a\u200Db.com/').catch((reason) => String(reason))
            ]).then((values) => {
              document.getElementById('result').textContent = values.join('|');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.set_fetch_mock("https://example.com./", "dot-host");
    h.set_fetch_mock("https://xn--4db.com/", "bidi-host");
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "dot-host|bidi-host|TypeError: Invalid URL|TypeError: Invalid URL",
    )?;
    assert_eq!(
        h.take_fetch_calls(),
        vec![
            "https://example\u{3002}com./".to_string(),
            "https://\u{5d0}.com/".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn cookie_store_set_get_get_all_and_delete_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            await cookieStore.set('cookie1', 'cookie1-value');
            await cookieStore.set({
              name: 'cookie2',
              value: 'cookie2-value',
              expires: Date.now() + 24 * 60 * 60 * 1000,
              partitioned: true
            });

            const cookie1 = await cookieStore.get('cookie1');
            const cookie2 = await cookieStore.get({ name: 'cookie2' });
            const allBefore = await cookieStore.getAll();
            const namedByString = await cookieStore.getAll('cookie2');
            const namedByObject = await cookieStore.getAll({ name: 'cookie2' });

            await cookieStore.delete('cookie1');

            const allAfter = await cookieStore.getAll();
            const namesAfter = allAfter.map((cookie) => cookie.name).join(',');
            document.getElementById('result').textContent =
              cookie1.name + ':' +
              cookie1.value + ':' +
              cookie2.partitioned + ':' +
              allBefore.length + ':' +
              namedByString.length + ':' +
              namedByObject.length + ':' +
              namesAfter;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.click("#btn")?;
    h.assert_text("#result", "cookie1:cookie1-value:true:2:1:1:cookie2")?;
    Ok(())
}

#[test]
fn primitive_raw_getter_and_incompatible_receiver_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = 'Hi';
            const textToString = text['toString'];
            const textValueOf = text['valueOf'];
            const stringResult = [
              textToString.call(text),
              textValueOf.call(text),
              text['length'],
              text[1]
            ].join(',');

            const number = 255;
            const numberToString = number['toString'];
            const numberValueOf = number['valueOf'];
            const numberResult = [
              numberToString.call(number, 16),
              String(numberValueOf.call(number))
            ].join(',');

            const bigint = 255n;
            const bigintToString = bigint['toString'];
            const bigintValueOf = bigint['valueOf'];
            const bigintResult = [
              bigintToString.call(bigint, 16),
              String(bigintValueOf.call(bigint))
            ].join(',');

            const flag = false;
            const boolResult = [
              flag['toString'].call(flag),
              String(flag['valueOf'].call(flag))
            ].join(',');

            const sym = Symbol('id');
            const symbolResult = [
              sym['toString'].call(sym),
              sym['valueOf'].call(sym).description
            ].join(',');

            let bad = 'none';
            try {
              numberToString.call(text);
            } catch (e) {
              bad = String(e);
            }

            document.getElementById('result').textContent = [
              stringResult,
              numberResult,
              bigintResult,
              boolResult,
              symbolResult,
              bad
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Hi,Hi,2,i|ff,255|ff,255|false,false|Symbol(id),id|Number method called on incompatible receiver",
    )?;
    Ok(())
}

#[test]
fn cookie_store_alias_variable_calls_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            const store = cookieStore;
            await store.set('cookie1', 'cookie1-value');
            const one = await store.get('cookie1');
            const named = await store.getAll('cookie1');
            await store.delete('cookie1');
            const after = await cookieStore.get('cookie1');
            document.getElementById('result').textContent =
              one.value + ':' + named.length + ':' + (after === null);
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.click("#btn")?;
    h.assert_text("#result", "cookie1-value:1:true")?;
    Ok(())
}

#[test]
fn cookie_store_raw_getter_and_inherited_receiver_parity_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            const setCookie = cookieStore['set'];
            const getCookie = cookieStore.get;
            const getAllCookies = cookieStore['getAll'];
            const deleteCookie = cookieStore.delete;

            await setCookie.call(cookieStore, 'cookie1', 'value1');
            const cookie = await getCookie.call(cookieStore, 'cookie1');
            const countBeforeDelete = (await getAllCookies.call(cookieStore)).length;
            await deleteCookie.call(cookieStore, 'cookie1');
            const afterDelete = await getCookie.call(cookieStore, 'cookie1');

            const receiverError = (() => {
              const inheritor = Object.create(cookieStore);
              try {
                inheritor.get('cookie1');
                return false;
              } catch (e) {
                return String(e).includes('CookieStore method called on incompatible receiver');
              }
            })();

            document.getElementById('result').textContent = [
              setCookie.name,
              setCookie.length,
              getCookie.name,
              getCookie.length,
              cookie.value,
              countBeforeDelete,
              afterDelete === null,
              receiverError
            ].join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.click("#btn")?;
    h.assert_text("#result", "set,1,get,1,value1,1,true,true")?;
    Ok(())
}

#[test]
fn cookie_store_integrates_with_document_cookie() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            document.cookie = 'favorite_food=tripe; SameSite=None; Secure';
            await cookieStore.set('cookie1', 'cookie1-value');
            const favorite = await cookieStore.get('favorite_food');
            const raw = document.cookie;
            document.getElementById('result').textContent =
              favorite.name + ':' +
              favorite.value + ':' +
              raw.includes('favorite_food=tripe') + ':' +
              raw.includes('cookie1=cookie1-value');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/path", html)?;
    h.click("#btn")?;
    h.assert_text("#result", "favorite_food:tripe:true:true")?;
    Ok(())
}

#[test]
fn cookie_store_change_event_fires_for_set_and_delete() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let log = [];
          cookieStore.addEventListener('change', (event) => {
            const changed = event.changed.map((cookie) => cookie.name).join(',');
            const deleted = event.deleted.map((cookie) => cookie.name).join(',');
            log.push(changed + '|' + deleted);
          });

          document.getElementById('btn').addEventListener('click', async () => {
            await cookieStore.set('cookie1', 'value1');
            await cookieStore.delete('cookie1');
            document.getElementById('result').textContent = log.join(';');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.click("#btn")?;
    h.assert_text("#result", "cookie1|;|cookie1")?;
    Ok(())
}

#[test]
fn cookie_store_is_available_only_in_secure_context() -> Result<()> {
    let insecure = Harness::from_html(
        r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent = typeof cookieStore;
        </script>
        "#,
    )?;
    insecure.assert_text("#result", "undefined")?;

    let secure = Harness::from_html_with_url(
        "https://app.local/",
        r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent =
            typeof cookieStore + ':' + (cookieStore === window.cookieStore);
        </script>
        "#,
    )?;
    secure.assert_text("#result", "object:true")?;
    Ok(())
}

#[test]
fn cache_storage_is_available_only_in_secure_context() -> Result<()> {
    let insecure = Harness::from_html(
        r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent = typeof caches;
        </script>
        "#,
    )?;
    insecure.assert_text("#result", "undefined")?;

    let secure = Harness::from_html_with_url(
        "https://app.local/",
        r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent =
            typeof caches + ':' + (caches === window.caches);
        </script>
        "#,
    )?;
    secure.assert_text("#result", "object:true")?;
    Ok(())
}

#[test]
fn cache_storage_open_has_keys_and_delete_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            const v1 = await caches.open('v1');
            const v1Again = await caches.open('v1');
            await caches.open('v2');

            const hasV1 = await caches.has('v1');
            const hasV2 = await caches.has('v2');
            const names = await caches.keys();
            const deletedV1 = await caches.delete('v1');
            const deletedMissing = await caches.delete('missing');
            const hasV1AfterDelete = await caches.has('v1');

            document.getElementById('result').textContent =
              (v1 === v1Again) + ':' +
              hasV1 + ':' +
              hasV2 + ':' +
              names.join(',') + ':' +
              deletedV1 + ':' +
              deletedMissing + ':' +
              hasV1AfterDelete;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:v1,v2:true:false:false")?;
    Ok(())
}

#[test]
fn cache_storage_match_and_cache_put_delete_keys_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            const cache = await caches.open('v1');
            const network = await fetch('/api/cached');
            await cache.put('/api/cached', network.clone());

            const fromCache = await cache.match('/api/cached');
            const fromStorage = await caches.match('/api/cached');
            const bodyCache = fromCache ? await fromCache.text() : 'none';
            const bodyStorage = fromStorage ? await fromStorage.text() : 'none';

            const keysBeforeDelete = await cache.keys();
            const keyUrl = keysBeforeDelete[0].url;
            const deleted = await cache.delete('/api/cached');
            const afterDelete = await cache.match('/api/cached');

            document.getElementById('result').textContent =
              bodyCache + ':' +
              bodyStorage + ':' +
              keyUrl + ':' +
              deleted + ':' +
              (afterDelete === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.set_fetch_mock("/api/cached", "payload");
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "payload:payload:https://app.local/api/cached:true:true",
    )?;
    Ok(())
}

#[test]
fn cache_storage_and_cache_alias_variables_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            const storage = caches;
            const cache = await storage.open('alias');
            const response = await fetch('/api/alias');
            await cache.put('/api/alias', response);

            const hasAlias = await storage.has('alias');
            const names = await storage.keys();
            const deleted = await cache.delete('/api/alias');
            const keysAfterDelete = await cache.keys();

            document.getElementById('result').textContent =
              hasAlias + ':' +
              names.join(',') + ':' +
              deleted + ':' +
              keysAfterDelete.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.set_fetch_mock("/api/alias", "ok");
    h.click("#btn")?;
    h.assert_text("#result", "true:alias:true:0")?;
    Ok(())
}

#[test]
fn cache_storage_and_cache_raw_getter_and_inherited_receiver_parity_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            const openCache = caches['open'];
            const storageMatch = caches.match;
            const cache = await openCache.call(caches, 'v1');
            const putEntry = cache['put'];
            const matchEntry = cache.match;
            const listKeys = cache['keys'];

            const network = await fetch('/api/raw-getter');
            await putEntry.call(cache, '/api/raw-getter', network.clone());

            const matched = await matchEntry.call(cache, '/api/raw-getter');
            const matchedText = await matched.text();
            const storageMatched = await storageMatch.call(caches, '/api/raw-getter');
            const storageText = await storageMatched.text();
            const keyCount = (await listKeys.call(cache)).length;

            const storageReceiverError = (() => {
              const inheritor = Object.create(caches);
              try {
                inheritor.open('v2');
                return false;
              } catch (e) {
                return String(e).includes('CacheStorage method called on incompatible receiver');
              }
            })();

            const cacheReceiverError = (() => {
              const inheritor = Object.create(cache);
              const inheritedKeys = inheritor['keys'];
              try {
                inheritedKeys.call(inheritor);
                return false;
              } catch (e) {
                return String(e).includes('Cache method called on incompatible receiver');
              }
            })();

            document.getElementById('result').textContent = [
              openCache.name,
              openCache.length,
              putEntry.name,
              putEntry.length,
              matchedText,
              storageText,
              keyCount,
              storageReceiverError,
              cacheReceiverError
            ].join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.set_fetch_mock("/api/raw-getter", "payload");
    h.click("#btn")?;
    h.assert_text("#result", "open,1,put,2,payload,payload,1,true,true")?;
    Ok(())
}

#[test]
fn storage_cache_and_cookie_store_constructor_surface_and_branding_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            const cache = await caches.open('v1');
            const constructors = [Storage, CookieStore, CacheStorage, Cache];
            const illegal = constructors.map((Ctor) => {
              try {
                new Ctor();
                return false;
              } catch (e) {
                return String(e).includes('Illegal constructor');
              }
            }).join(',');

            document.getElementById('result').textContent = [
              typeof Storage,
              typeof CookieStore,
              typeof CacheStorage,
              typeof Cache,
              String(window.Storage === Storage),
              String(window.CookieStore === CookieStore),
              String(window.CacheStorage === CacheStorage),
              String(window.Cache === Cache),
              String(localStorage.constructor === Storage),
              String(Object.getPrototypeOf(localStorage) === Storage.prototype),
              String(caches.constructor === CacheStorage),
              String(Object.getPrototypeOf(caches) === CacheStorage.prototype),
              String(cache.constructor === Cache),
              String(Object.getPrototypeOf(cache) === Cache.prototype),
              String(cookieStore.constructor === CookieStore),
              String(Object.getPrototypeOf(cookieStore) === CookieStore.prototype),
              Object.prototype.toString.call(localStorage),
              Object.prototype.toString.call(caches),
              Object.prototype.toString.call(cache),
              Object.prototype.toString.call(cookieStore),
              String(Storage.prototype.constructor === Storage),
              String(CookieStore.prototype.constructor === CookieStore),
              String(CacheStorage.prototype.constructor === CacheStorage),
              String(Cache.prototype.constructor === Cache),
              illegal
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "function|function|function|function|true|true|true|true|true|true|true|true|true|true|true|true|[object Storage]|[object CacheStorage]|[object Cache]|[object CookieStore]|true|true|true|true|true,true,true,true",
    )?;
    Ok(())
}

#[test]
fn secure_storage_like_constructors_are_hidden_in_insecure_contexts_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent = [
            typeof Storage,
            typeof CookieStore,
            typeof CacheStorage,
            typeof Cache,
            typeof localStorage,
            typeof cookieStore,
            typeof caches
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "function|undefined|undefined|undefined|object|undefined|undefined",
    )?;
    Ok(())
}

#[test]
fn cache_add_and_add_all_work_with_fetch_mocks() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', async () => {
            const cache = await caches.open('v1');
            await cache.add('/a');
            await cache.addAll(['/b', '/c']);

            const a = await (await cache.match('/a')).text();
            const b = await (await cache.match('/b')).text();
            const c = await (await cache.match('/c')).text();

            document.getElementById('result').textContent =
              a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.set_fetch_mock("/a", "A");
    h.set_fetch_mock("/b", "B");
    h.set_fetch_mock("/c", "C");
    h.click("#btn")?;
    h.assert_text("#result", "A:B:C")?;
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
fn match_media_event_target_methods_work_in_expression_context() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const mql = matchMedia('(width <= 600px)');
          let calls = 0;
          const listener = () => { calls += 1; };
          document.getElementById('btn').addEventListener('click', () => {
            const ret = mql.addEventListener('change', listener);
            mql.dispatchEvent(new Event('change'));
            const removed = mql.removeEventListener('change', listener);
            mql.dispatchEvent(new Event('change'));
            document.getElementById('result').textContent =
              (ret === undefined) + ':' + (removed === undefined) + ':' + calls;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:1")?;
    Ok(())
}

#[test]
fn match_media_add_listener_alias_and_onchange_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const mql = matchMedia('(prefers-reduced-motion: reduce)');
          let count = 0;
          const legacy = () => { count += 1; };
          document.getElementById('btn').addEventListener('click', () => {
            mql.onchange = () => { count += 10; };
            mql.addListener(legacy);
            mql.dispatchEvent(new Event('change'));
            mql.removeListener(legacy);
            mql.onchange = null;
            mql.dispatchEvent(new Event('change'));
            document.getElementById('result').textContent =
              count + ':' + (mql.onchange === null);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "11:true")?;
    Ok(())
}

#[test]
fn match_media_raw_getter_and_inherited_property_paths_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const mql = matchMedia('(prefers-reduced-motion: reduce)');
          let count = 0;
          const legacy = () => { count += 1; };
          const modern = () => { count += 10; };

          document.getElementById('btn').addEventListener('click', () => {
            const addLegacy = mql['addListener'];
            const removeLegacy = Object.create(mql).removeListener;
            const addModern = mql.addEventListener;
            const dispatch = mql['dispatchEvent'];

            let incompatible = false;
            try {
              addLegacy.call({}, legacy);
            } catch (error) {
              incompatible = String(error).includes('MediaQueryList');
            }

            document.getElementById('result').textContent = [
              addLegacy.name,
              addLegacy.length,
              removeLegacy.name,
              removeLegacy.length,
              addModern.name,
              addModern.length,
              dispatch.name,
              dispatch.length,
              incompatible
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "addListener|1|removeListener|1|addEventListener|2|dispatchEvent|1|true",
    )?;
    Ok(())
}

#[test]
fn placeholder_backed_host_methods_are_non_enumerable_and_shadowable_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const mql = matchMedia('(prefers-reduced-motion: reduce)');

          document.getElementById('btn').addEventListener('click', async () => {
            const mqlDesc = Object.getOwnPropertyDescriptor(mql, 'addListener');
            const mqlSummary = [
              mqlDesc.value.name,
              mqlDesc.value.length,
              mqlDesc.enumerable,
              Object.keys(mql).includes('addListener'),
              Object.getOwnPropertyNames(mql).includes('addListener')
            ].join(':');
            Object.defineProperty(mql, 'addListener', {
              value() { return 'shadow-mql'; },
              configurable: true
            });
            const mqlShadow = mql.addListener(() => {});
            const mqlDeleted = delete mql.addListener;
            const mqlGone = mql.addListener === undefined;

            const cookieDesc = Object.getOwnPropertyDescriptor(cookieStore, 'get');
            const cookieSummary = [
              cookieDesc.value.name,
              cookieDesc.value.length,
              cookieDesc.enumerable,
              Object.keys(cookieStore).includes('get'),
              Object.getOwnPropertyNames(cookieStore).includes('get')
            ].join(':');
            Object.defineProperty(cookieStore, 'get', {
              value() { return Promise.resolve('shadow-cookie'); },
              configurable: true
            });
            const cookieShadow = await cookieStore.get('alpha');
            const cookieDeleted = delete cookieStore.get;
            const cookieGone = cookieStore.get === undefined;

            const storageDesc = Object.getOwnPropertyDescriptor(caches, 'keys');
            const storageSummary = [
              storageDesc.value.name,
              storageDesc.value.length,
              storageDesc.enumerable,
              Object.keys(caches).includes('keys'),
              Object.getOwnPropertyNames(caches).includes('keys')
            ].join(':');
            Object.defineProperty(caches, 'keys', {
              value() { return Promise.resolve(['shadow-storage']); },
              configurable: true
            });
            const storageShadow = (await caches.keys()).join(',');
            const storageDeleted = delete caches.keys;
            const storageGone = caches.keys === undefined;

            const cache = await caches.open('v1');
            const cacheDesc = Object.getOwnPropertyDescriptor(cache, 'keys');
            const cacheSummary = [
              cacheDesc.value.name,
              cacheDesc.value.length,
              cacheDesc.enumerable,
              Object.keys(cache).includes('keys'),
              Object.getOwnPropertyNames(cache).includes('keys')
            ].join(':');
            Object.defineProperty(cache, 'keys', {
              value() { return Promise.resolve(['shadow-cache']); },
              configurable: true
            });
            const cacheShadow = (await cache.keys()).join(',');
            const cacheDeleted = delete cache.keys;
            const cacheGone = cache.keys === undefined;

            document.getElementById('result').textContent = [
              mqlSummary,
              mqlShadow,
              mqlDeleted,
              mqlGone,
              cookieSummary,
              cookieShadow,
              cookieDeleted,
              cookieGone,
              storageSummary,
              storageShadow,
              storageDeleted,
              storageGone,
              cacheSummary,
              cacheShadow,
              cacheDeleted,
              cacheGone
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/", html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "addListener:1:false:false:true|shadow-mql|true|true|get:1:false:false:true|shadow-cookie|true|true|keys:0:false:false:true|shadow-storage|true|true|keys:0:false:false:true|shadow-cache|true|true",
    )?;
    Ok(())
}

#[test]
fn match_media_matches_property_is_live_for_existing_objects() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const mql = matchMedia('(unknown-query)');
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              mql.matches + ':' + mql.media;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:(unknown-query)")?;

    h.set_default_match_media_matches(true);
    h.click("#btn")?;
    h.assert_text("#result", "true:(unknown-query)")?;
    Ok(())
}

#[test]
fn match_media_synthesized_properties_respect_override_delete_and_inherited_reads_work()
-> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const mql = matchMedia('(unknown-query)');
          const child = Object.create(mql);

          document.getElementById('btn').addEventListener('click', () => {
            const defaultDesc = Object.getOwnPropertyDescriptor(mql, 'matches');
            const defaultSummary = [
              String(mql.matches),
              String(child.matches),
              String(defaultDesc === undefined),
              String(Object.keys(mql).includes('matches')),
              String(Reflect.ownKeys(mql).includes('matches'))
            ].join(':');

            Object.defineProperty(mql, 'matches', {
              get() { return 'shadow-match'; },
              configurable: true
            });
            Object.defineProperty(mql, 'media', {
              value: 'shadow-media',
              enumerable: true,
              configurable: true
            });

            const overrideDesc = Object.getOwnPropertyDescriptor(mql, 'matches');
            const overrideSummary = [
              String(mql.matches),
              String(child.matches),
              String(mql.media),
              String(child.media),
              String(typeof overrideDesc.get === 'function'),
              String(Object.keys(mql).includes('media')),
              String(Reflect.ownKeys(mql).includes('media'))
            ].join(':');

            delete mql.matches;
            delete mql.media;

            const afterDelete = [
              String(mql.matches),
              String(child.matches),
              String(mql.media),
              String(child.media),
              String(Object.keys(mql).includes('media')),
              String(Reflect.ownKeys(mql).includes('media'))
            ].join(':');

            document.getElementById('result').textContent = [
              defaultSummary,
              overrideSummary,
              afterDelete
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "false:false:true:false:false|shadow-match:shadow-match:shadow-media:shadow-media:true:true:true|false:false:(unknown-query):(unknown-query):false:false",
    )?;
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
fn navigator_clipboard_property_can_be_overridden_for_stubbing() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          navigator.clipboard = {
            readText: function() { return Promise.resolve('stubbed-read'); },
            writeText: function(_value) { return Promise.resolve('stubbed-write'); },
          };

          document.getElementById('btn').addEventListener('click', () => {
            navigator.clipboard.writeText('saved')
              .then(() => navigator.clipboard.readText())
              .then((value) => {
                document.getElementById('result').textContent = value;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "stubbed-read")?;
    assert_eq!(h.clipboard_text(), "");
    Ok(())
}

#[test]
fn navigator_clipboard_read_text_rejection_can_be_mocked() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            navigator.clipboard.readText()
              .then((value) => {
                document.getElementById('result').textContent = 'ok:' + value;
              })
              .catch((reason) => {
                document.getElementById('result').textContent = 'err:' + String(reason);
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_read_error(Some("NotAllowedError"));
    h.click("#btn")?;
    h.assert_text("#result", "err:NotAllowedError")?;

    h.clear_clipboard_errors();
    h.set_clipboard_text("after-clear");
    h.click("#btn")?;
    h.assert_text("#result", "ok:after-clear")?;
    Ok(())
}

#[test]
fn navigator_clipboard_write_text_rejection_can_be_mocked() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            navigator.clipboard.writeText('saved')
              .then(() => navigator.clipboard.readText())
              .then((value) => {
                document.getElementById('result').textContent = 'ok:' + value;
              })
              .catch((reason) => {
                document.getElementById('result').textContent = 'err:' + String(reason);
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_write_error(Some("WriteBlocked"));
    h.click("#btn")?;
    h.assert_text("#result", "err:WriteBlocked")?;
    assert_eq!(h.clipboard_text(), "");

    h.clear_clipboard_errors();
    h.click("#btn")?;
    h.assert_text("#result", "ok:saved")?;
    assert_eq!(h.clipboard_text(), "saved");
    Ok(())
}

#[test]
fn navigator_clipboard_method_override_is_used_even_with_special_clipboard_ast() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          navigator.clipboard.readText = () => Promise.resolve('stubbed-read');
          navigator.clipboard.writeText = () => Promise.resolve('stubbed-write');

          document.getElementById('btn').addEventListener('click', () => {
            navigator.clipboard.writeText('saved')
              .then(() => navigator.clipboard.readText())
              .then((value) => {
                document.getElementById('result').textContent = value;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "stubbed-read")?;
    assert_eq!(h.clipboard_text(), "");
    Ok(())
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
fn structured_clone_preserves_circular_and_shared_references() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const source = { name: 'MDN' };
            source.self = source;
            source.shared = { value: 1 };
            source.left = source.shared;
            source.right = source.shared;

            const clone = structuredClone(source);
            const left = clone.left;
            left.value = 9;

            document.getElementById('result').textContent =
              (clone !== source) + ':' +
              (clone.self === clone) + ':' +
              (clone.left === clone.right) + ':' +
              (clone.left !== source.shared) + ':' +
              source.shared.value + ':' +
              clone.right.value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true:1:9")?;
    Ok(())
}

#[test]
fn structured_clone_transfers_array_buffer() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const buffer = new ArrayBuffer(4);
            const source = new Uint8Array(buffer);
            source[0] = 1;
            source[1] = 2;

            const clone = structuredClone({ buffer }, { transfer: [buffer] });
            const moved = new Uint8Array(clone.buffer);
            const detachedView = new Uint8Array(buffer);

            document.getElementById('result').textContent =
              buffer.byteLength + ':' +
              moved[0] + ':' +
              moved[1] + ':' +
              detachedView.byteLength;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:1:2:0")?;
    Ok(())
}

#[test]
fn structured_clone_window_method_reference_supports_options() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sc = window.structuredClone;
            const buffer = new ArrayBuffer(2);
            const source = new Uint8Array(buffer);
            source[0] = 7;

            const clone = sc({ buffer }, { transfer: [buffer] });
            const moved = new Uint8Array(clone.buffer);

            document.getElementById('result').textContent =
              buffer.byteLength + ':' + moved[0];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:7")?;
    Ok(())
}

#[test]
fn structured_clone_invalid_transfer_list_item_throws_data_clone_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            try {
              structuredClone({ x: 1 }, { transfer: [1] });
              document.getElementById('result').textContent = 'no-error';
            } catch (e) {
              document.getElementById('result').textContent =
                String(e).includes('DataCloneError') &&
                String(e).includes('transfer list items');
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true")?;
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
fn request_animation_frame_window_method_reference_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            const raf = window.requestAnimationFrame;
            raf((ts) => {
              out.textContent = 'ts=' + ts;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(16)?;
    h.assert_text("#result", "ts=16")?;
    Ok(())
}

#[test]
fn request_animation_frame_method_requires_callable_callback() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const raf = window.requestAnimationFrame;
            raf(123);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("requestAnimationFrame should reject non-callable callbacks");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("requestAnimationFrame callback must be callable"))
        }
        other => panic!("unexpected requestAnimationFrame error: {other:?}"),
    }
    Ok(())
}

#[test]
fn cancel_animation_frame_window_method_reference_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            const cancel = window.cancelAnimationFrame;
            const canceled = requestAnimationFrame(() => {
              out.textContent = 'CANCELED';
            });
            const returnValue = cancel(canceled);
            requestAnimationFrame((ts) => {
              out.textContent = String(returnValue === undefined) + '|R' + ts;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(16)?;
    h.assert_text("#result", "true|R16")?;
    Ok(())
}

#[test]
fn clear_interval_window_method_reference_cancels_interval() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            const clear = window.clearInterval;
            let count = 0;
            const id = setInterval(() => {
              count += 1;
              out.textContent = String(count);
              if (count === 2) {
                clear(id);
              }
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "1")?;
    h.advance_time(5)?;
    h.assert_text("#result", "2")?;
    h.advance_time(20)?;
    h.assert_text("#result", "2")?;
    Ok(())
}

#[test]
fn set_interval_window_method_reference_supports_callback_and_extra_arguments() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            const setIntervalRef = window.setInterval;
            let count = 0;
            const id = setIntervalRef((left, right) => {
              count += 1;
              out.textContent = String(count) + ':' + left + right;
            }, 5, 'A', 'B');
            out.textContent = 'scheduled:' + String(id);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "scheduled:1")?;
    h.advance_time(5)?;
    h.assert_text("#result", "1:AB")?;
    h.advance_time(5)?;
    h.assert_text("#result", "2:AB")?;
    assert!(h.clear_timer(1));
    h.advance_time(20)?;
    h.assert_text("#result", "2:AB")?;
    Ok(())
}

#[test]
fn set_interval_window_method_reference_supports_string_code_callback() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const setIntervalRef = window.setInterval;
            const id = setIntervalRef(
              "window.__intervalCodeCount = (window.__intervalCodeCount || 0) + 1; document.getElementById('result').textContent = 'code:' + window.__intervalCodeCount;",
              5
            );
            document.getElementById('result').textContent = 'scheduled:' + String(id);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "scheduled:1")?;
    h.advance_time(5)?;
    h.assert_text("#result", "code:1")?;
    assert!(h.clear_timer(1));
    Ok(())
}

#[test]
fn set_interval_window_method_reference_requires_at_least_one_argument() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const setIntervalRef = window.setInterval;
            setIntervalRef();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("setInterval should reject empty argument list");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("setInterval requires at least one argument"))
        }
        other => panic!("unexpected setInterval error: {other:?}"),
    }
    Ok(())
}

#[test]
fn set_interval_window_method_reference_rejects_unsupported_callback_type() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const setIntervalRef = window.setInterval;
            setIntervalRef(123, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("setInterval should reject non-callable and non-string callbacks");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("TypeError: setInterval callback must be callable or a string"),)
        }
        other => panic!("unexpected setInterval error: {other:?}"),
    }
    Ok(())
}

#[test]
fn set_timeout_window_method_reference_supports_callback_and_extra_arguments() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            const setTimeoutRef = window.setTimeout;
            const id = setTimeoutRef((left, right) => {
              out.textContent = left + right;
            }, 5, 'A', 'B');
            out.textContent = 'scheduled:' + String(id);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "scheduled:1")?;
    h.advance_time(5)?;
    h.assert_text("#result", "AB")?;
    h.advance_time(20)?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn set_timeout_window_method_reference_supports_string_code_callback() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const setTimeoutRef = window.setTimeout;
            const id = setTimeoutRef(
              "window.__timeoutCodeCount = (window.__timeoutCodeCount || 0) + 1; document.getElementById('result').textContent = 'code:' + window.__timeoutCodeCount;",
              5
            );
            document.getElementById('result').textContent = 'scheduled:' + String(id);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "scheduled:1")?;
    h.advance_time(5)?;
    h.assert_text("#result", "code:1")?;
    h.advance_time(20)?;
    h.assert_text("#result", "code:1")?;
    Ok(())
}

#[test]
fn set_timeout_window_method_reference_requires_at_least_one_argument() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const setTimeoutRef = window.setTimeout;
            setTimeoutRef();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("setTimeout should reject empty argument list");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("setTimeout requires at least one argument"))
        }
        other => panic!("unexpected setTimeout error: {other:?}"),
    }
    Ok(())
}

#[test]
fn set_timeout_window_method_reference_rejects_unsupported_callback_type() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const setTimeoutRef = window.setTimeout;
            setTimeoutRef(123, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("setTimeout should reject non-callable and non-string callbacks");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("TypeError: setTimeout callback must be callable or a string"),)
        }
        other => panic!("unexpected setTimeout error: {other:?}"),
    }
    Ok(())
}

#[test]
fn clear_interval_unknown_id_is_noop_and_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = window.clearInterval(999999);
            document.getElementById('result').textContent = String(result === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn clear_timeout_window_method_reference_cancels_timeout() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            const clear = window.clearTimeout;
            const timeoutId = setTimeout(() => {
              out.textContent = 'TIMEOUT_FIRED';
            }, 10);
            clear(timeoutId);
            setTimeout(() => {
              out.textContent = 'still-canceled';
            }, 12);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(10)?;
    h.assert_text("#result", "")?;
    h.advance_time(2)?;
    h.assert_text("#result", "still-canceled")?;
    Ok(())
}

#[test]
fn clear_timeout_unknown_id_is_noop_and_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = window.clearTimeout(1234567);
            document.getElementById('result').textContent = String(result === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn request_animation_frame_callbacks_queued_for_same_frame_share_timestamp() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            requestAnimationFrame((ts) => {
              out.textContent = out.textContent + 'A' + ts + '|';
            });
            setTimeout(() => {
              requestAnimationFrame((ts) => {
                out.textContent = out.textContent + 'B' + ts;
              });
            }, 1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;

    h.advance_time(1)?;
    h.assert_text("#result", "")?;

    h.advance_time(15)?;
    h.assert_text("#result", "A16|B16")?;
    Ok(())
}

#[test]
fn request_animation_frame_is_one_shot_and_requires_reregistration() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            let count = 0;
            function step(ts) {
              count += 1;
              out.textContent = out.textContent + count + ':' + ts + '|';
              if (count < 3) {
                requestAnimationFrame(step);
              }
            }
            requestAnimationFrame(step);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;

    h.advance_time(16)?;
    h.assert_text("#result", "1:16|")?;

    h.advance_time(16)?;
    h.assert_text("#result", "1:16|2:32|")?;

    h.advance_time(16)?;
    h.assert_text("#result", "1:16|2:32|3:48|")?;
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
fn alert_supports_optional_message_and_window_method_reference() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const alias = window.alert;
            alert();
            window.alert(123);
            alias(true);
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok")?;
    assert_eq!(
        h.take_alert_messages(),
        vec!["".to_string(), "123".to_string(), "true".to_string()]
    );
    Ok(())
}

#[test]
fn confirm_supports_optional_message_and_window_method_reference() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const alias = window.confirm;
            const a = confirm();
            const b = window.confirm(123);
            const c = alias(true);
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enqueue_confirm_response(true);
    h.enqueue_confirm_response(false);
    h.enqueue_confirm_response(true);
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true")?;
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
fn prompt_supports_optional_message_and_window_method_reference() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const alias = window.prompt;
            const a = prompt();
            const b = window.prompt('name?');
            const c = alias('role?', 'dev');
            document.getElementById('result').textContent =
              String(a === null) + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enqueue_prompt_response(None);
    h.enqueue_prompt_response(Some("kazu"));
    h.click("#btn")?;
    h.assert_text("#result", "true:kazu:dev")?;
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
            "<script>window.JSON.stringify();</script>",
            "JSON.stringify requires one to three arguments",
        ),
        (
            "<script>fetch();</script>",
            "fetch requires one or two arguments",
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
            "structuredClone requires one or two arguments",
        ),
        (
            "<script>window.alert('ok', 'ng');</script>",
            "alert requires zero or one argument",
        ),
        (
            "<script>window.confirm('ok', 'ng');</script>",
            "confirm requires zero or one argument",
        ),
        (
            "<script>prompt('x', 'y', 'z');</script>",
            "prompt requires zero to two arguments",
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
            "<script>setTimeout();</script>",
            "setTimeout requires at least 1 argument",
        ),
        (
            "<script>cancelAnimationFrame();</script>",
            "cancelAnimationFrame requires 1 argument",
        ),
        (
            "<script>clearInterval();</script>",
            "clearInterval requires 1 argument",
        ),
        (
            "<script>clearTimeout();</script>",
            "clearTimeout requires 1 argument",
        ),
        (
            "<script>queueMicrotask();</script>",
            "queueMicrotask requires exactly one argument",
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
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("InvalidCharacterError"));
            assert!(msg.contains("non-Latin1"));
        }
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
fn atob_window_method_reference_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const decode = window.atob;
            const decoded = decode('Qg==');
            document.getElementById('result').textContent = decoded;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B")?;
    Ok(())
}

#[test]
fn btoa_window_method_reference_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const encode = window.btoa;
            const encoded = encode('B');
            document.getElementById('result').textContent = encoded;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Qg==")?;
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
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("InvalidCharacterError"));
            assert!(msg.contains("invalid base64"));
        }
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
fn json_stringify_supports_space_argument() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { a: 1, b: { c: 2 } };
            const pretty = JSON.stringify(obj, null, 2);
            const compact = JSON.stringify(obj, null, 0);
            const prettyHasTopIndent = pretty.includes('\n  "a": 1');
            const prettyHasNestedIndent = pretty.includes('\n    "c": 2');
            document.getElementById('result').textContent =
              prettyHasTopIndent + ':' + prettyHasNestedIndent + ':' + compact;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:{\"a\":1,\"b\":{\"c\":2}}")?;
    Ok(())
}

#[test]
fn dom_parser_and_tree_walker_basics_are_supported() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parser = new DOMParser();
            const doc = parser.parseFromString('<div id="root">A<!--x--><span>B</span></div>', 'text/html');
            const root = doc.getElementById('root');
            const walker = doc.createTreeWalker(root, NodeFilter.SHOW_TEXT);
            const values = [];
            let current = walker.nextNode();
            while (current) {
              values.push(current.textContent.trim());
              current = walker.nextNode();
            }

            const commentWalker = doc.createTreeWalker(doc.body, NodeFilter.SHOW_COMMENT);
            const noComment = commentWalker.nextNode() === null;
            root.remove();

            document.getElementById('result').textContent =
              (root !== null) + ':' +
              (root.nodeType === Node.ELEMENT_NODE) + ':' +
              (Node.TEXT_NODE === 3) + ':' +
              values.join(',') + ':' +
              noComment + ':' +
              (doc.getElementById('root') === null);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:A,B:true:true")?;
    Ok(())
}

#[test]
fn dom_parser_tree_walker_and_parsed_document_placeholder_methods_support_extracted_and_inherited_calls_work()
-> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parser = new DOMParser();
            const parse = parser.parseFromString;
            const parsed = parse.call(
              parser,
              '<div id="root">A<span>B</span></div>',
              'text/html'
            );
            const parsedGet = parsed.getElementById;
            const root = parsedGet.call(parsed, 'root');
            const walker = parsed.createTreeWalker(root, NodeFilter.SHOW_TEXT);
            const next = walker.nextNode;
            const first = next.call(walker).textContent.trim();
            const second = next.call(walker).textContent.trim();

            const parserReceiverError = (() => {
              const inheritor = Object.create(parser);
              try {
                inheritor.parseFromString('<p></p>', 'text/html');
                return false;
              } catch (e) {
                return String(e).includes('DOMParser method called on incompatible receiver');
              }
            })();

            const parsedReceiverError = (() => {
              const inheritor = Object.create(parsed);
              try {
                inheritor.getElementById('root');
                return false;
              } catch (e) {
                return String(e).includes('Document method called on incompatible receiver');
              }
            })();

            const walkerReceiverError = (() => {
              const inheritor = Object.create(walker);
              try {
                inheritor.nextNode();
                return false;
              } catch (e) {
                return String(e).includes('TreeWalker method called on incompatible receiver');
              }
            })();

            document.getElementById('result').textContent = [
              parse.name,
              parse.length,
              root.id,
              next.name,
              next.length,
              first + second,
              parserReceiverError,
              parsedReceiverError,
              walkerReceiverError
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "parseFromString|2|root|nextNode|0|AB|true|true|true",
    )?;
    Ok(())
}

#[test]
fn dom_parser_parsed_document_and_tree_walker_methods_respect_shadowing_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parser = new DOMParser();
            Object.defineProperty(parser, 'parseFromString', {
              value() { return 'shadow-parser'; },
              configurable: true
            });
            const parserShadow = parser.parseFromString('<p></p>', 'text/html');
            const parserDeleted = delete parser.parseFromString;
            const parserGone = parser.parseFromString === undefined;

            const parsed = new DOMParser().parseFromString(
              '<div id="root"><span>A</span></div>',
              'text/html'
            );
            Object.defineProperty(parsed, 'getElementById', {
              value() { return 'shadow-parsed'; },
              configurable: true
            });
            const parsedShadow = parsed.getElementById('root');
            const parsedDeleted = delete parsed.getElementById;
            const parsedGone = parsed.getElementById === undefined;

            const walker = parsed.createTreeWalker(parsed.documentElement, NodeFilter.SHOW_TEXT);
            Object.defineProperty(walker, 'nextNode', {
              value() { return 'shadow-walker'; },
              configurable: true
            });
            const walkerShadow = walker.nextNode();
            const walkerDeleted = delete walker.nextNode;
            const walkerGone = walker.nextNode === undefined;

            document.getElementById('result').textContent = [
              parserShadow,
              parserDeleted,
              parserGone,
              parsedShadow,
              parsedDeleted,
              parsedGone,
              walkerShadow,
              walkerDeleted,
              walkerGone
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "shadow-parser|true|true|shadow-parsed|true|true|shadow-walker|true|true",
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
fn object_and_reflect_support_array_function_and_collection_targets_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [10, 0, 30];
            function sample(a, b) {}
            const map = new Map();

            Object.defineProperty(arr, 'extra', { value: 'x', enumerable: true });
            Reflect.set(arr, '1', 20);

            Object.defineProperty(sample, 'x', { value: 'fx', enumerable: true });
            Reflect.set(sample, 'y', 'fy');

            Object.defineProperty(map, 'note', { value: 'm', enumerable: true });
            Reflect.set(map, 'tag', 't');

            const arrIndexDesc = Object.getOwnPropertyDescriptor(arr, '1');
            const arrLengthDesc = Object.getOwnPropertyDescriptor(arr, 'length');
            const fnDesc = Object.getOwnPropertyDescriptor(sample, 'x');
            const fnLengthDesc = Object.getOwnPropertyDescriptor(sample, 'length');
            const mapDesc = Object.getOwnPropertyDescriptor(map, 'note');

            const fnKeysBeforeDelete = Object.keys(sample).join(',');
            const deletedFnX = delete sample.x;

            document.getElementById('result').textContent = [
              Object.keys(arr).join(','),
              Object.values(arr).join(','),
              Object.entries(arr).map((pair) => pair.join('=')).join(','),
              String(arrIndexDesc.value === 20 && arrIndexDesc.enumerable === true),
              String(arrLengthDesc.value === 3 && arrLengthDesc.enumerable === false),
              String(fnDesc.value === 'fx' && fnDesc.enumerable === true),
              String(fnLengthDesc.value === 2 && fnLengthDesc.enumerable === false),
              fnKeysBeforeDelete,
              sample.y,
              String(deletedFnX),
              String(!Object.hasOwn(sample, 'x') && Object.hasOwn(sample, 'length')),
              String(mapDesc.value === 'm' && mapDesc.enumerable === true),
              Object.keys(map).join(','),
              String(Object.hasOwn(map, 'note') && Object.hasOwn(map, 'tag')),
              map.note + ':' + map.tag
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "0,1,2,extra|10,20,30,x|0=10,1=20,2=30,extra=x|true|true|true|true|x,y|fy|false|false|true|note,tag|true|m:t",
    )?;
    Ok(())
}

#[test]
fn object_descriptor_and_reflect_work_on_callable_object_surfaces() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const descName = Object.getOwnPropertyDescriptor(Object.keys, 'name');
            const descLength = Object.getOwnPropertyDescriptor(Reflect.set, 'length');

            Reflect.set(Object.keys, 'tag', 'callable');
            Object.defineProperty(Reflect.set, 'note', {
              value: 'reflect',
              enumerable: true
            });

            document.getElementById('result').textContent = [
              String(descName.value === 'keys' && descName.enumerable === false),
              String(descLength.value === 3 && descLength.enumerable === false),
              String(Object.hasOwn(Object.keys, 'name')),
              String(Object.hasOwn(Reflect.set, 'length')),
              Object.keys(Object.keys).join(','),
              Object.keys(Reflect.set).join(','),
              Object.keys.tag,
              Reflect.set.note,
              String(delete Object.keys.tag),
              String(!Object.hasOwn(Object.keys, 'tag'))
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|true|true|true|tag|note|callable|reflect|true|true",
    )?;
    Ok(())
}

#[test]
fn object_and_reflect_own_keys_and_descriptor_attributes_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const describeKey = (key) =>
              typeof key === 'symbol' ? 'sym:' + (Symbol.keyFor(key) || key.description) : key;

            const arr = [10, 20];
            const arrSym = Symbol('a');
            Object.defineProperty(arr, 'hidden', { value: 'h', enumerable: false });
            arr.extra = 'x';
            arr[arrSym] = 'sa';

            function sample(a, b) {}
            const fnSym = Symbol.for('b');
            Object.defineProperty(sample, 'note', { value: 'n', enumerable: true });
            sample[fnSym] = 'sb';

            Reflect.set(Object.keys, 'tag', 'callable');

            const map = new Map();
            Object.defineProperty(map, 'note', { value: 'm', enumerable: true });
            map[arrSym] = 'ms';

            const getNames = Object.getOwnPropertyNames;
            const ownKeys = Reflect.ownKeys;

            const arrLengthDesc = Object.getOwnPropertyDescriptor(arr, 'length');
            const fnNameDesc = Object.getOwnPropertyDescriptor(sample, 'name');
            const fnLengthDesc = Object.getOwnPropertyDescriptor(sample, 'length');
            const fnPrototypeDesc = Object.getOwnPropertyDescriptor(sample, 'prototype');
            const callableNameDesc = Object.getOwnPropertyDescriptor(Object.keys, 'name');
            const callableLengthDesc = Object.getOwnPropertyDescriptor(Object.keys, 'length');
            const mapSizeDesc = Object.getOwnPropertyDescriptor(map, 'size');

            document.getElementById('result').textContent = [
              getNames(arr).join(','),
              ownKeys(arr).map(describeKey).join(','),
              String(
                arrLengthDesc.writable === true &&
                arrLengthDesc.enumerable === false &&
                arrLengthDesc.configurable === false
              ),
              getNames(sample).join(','),
              ownKeys(sample).map(describeKey).join(','),
              String(
                fnNameDesc.writable === false &&
                fnNameDesc.enumerable === false &&
                fnNameDesc.configurable === true
              ),
              String(
                fnLengthDesc.writable === false &&
                fnLengthDesc.enumerable === false &&
                fnLengthDesc.configurable === true
              ),
              String(
                fnPrototypeDesc.writable === true &&
                fnPrototypeDesc.enumerable === false &&
                fnPrototypeDesc.configurable === false
              ),
              String(
                Object.getOwnPropertyDescriptor(sample, 'call') === undefined &&
                Object.hasOwn(sample, 'call') === false
              ),
              getNames(Object.keys).join(','),
              ownKeys(Object.keys).map(describeKey).join(','),
              String(
                callableNameDesc.writable === false &&
                callableNameDesc.enumerable === false &&
                callableNameDesc.configurable === true
              ),
              String(
                callableLengthDesc.writable === false &&
                callableLengthDesc.enumerable === false &&
                callableLengthDesc.configurable === true
              ),
              String(
                Object.getOwnPropertyDescriptor(Object.keys, 'call') === undefined &&
                Object.hasOwn(Object.keys, 'call') === false
              ),
              String(Object.getOwnPropertyNames(Object).includes('getOwnPropertyNames')),
              String(Reflect.ownKeys(Reflect).map(describeKey).join(',') === 'set,ownKeys,sym:Symbol.toStringTag'),
              getNames(map).join(','),
              ownKeys(map).map(describeKey).join(','),
              String(
                mapSizeDesc.writable === false &&
                mapSizeDesc.enumerable === false &&
                mapSizeDesc.configurable === true
              )
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "0,1,length,hidden,extra|0,1,length,hidden,extra,sym:a|true|length,name,prototype,note|length,name,prototype,note,sym:b|true|true|true|true|length,name,tag|length,name,tag|true|true|true|true|true|size,note|size,note,sym:a|true",
    )?;
    Ok(())
}

#[test]
fn object_and_reflect_descriptor_mutation_and_symbol_key_residuals_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const describeKey = (key) =>
              typeof key === 'symbol' ? 'sym:' + (Symbol.keyFor(key) || key.description) : key;

            const arr = [1, 2];
            const arrSymA = Symbol('a');
            const arrSymB = Symbol('b');
            Object.defineProperty(arr, '0', {
              value: 7,
              enumerable: false,
              writable: false,
              configurable: false
            });
            Object.defineProperty(arr, arrSymA, {
              value: 'sa',
              enumerable: false,
              writable: false,
              configurable: false
            });
            Reflect.set(arr, arrSymB, 'sb');
            const arrSet = Reflect.set(arr, '0', 9);
            const arrDelete = delete arr[0];
            const arrDesc = Object.getOwnPropertyDescriptor(arr, '0');
            const arrSymDesc = Object.getOwnPropertyDescriptor(arr, arrSymA);

            function sample(a, b) {}
            Object.defineProperty(sample, 'length', {
              value: 9,
              enumerable: true,
              writable: false,
              configurable: false
            });
            const fnSet = Reflect.set(sample, 'length', 10);
            const fnDelete = delete sample.length;
            const fnDesc = Object.getOwnPropertyDescriptor(sample, 'length');

            Object.defineProperty(Object.keys, 'name', {
              value: 'keys2',
              enumerable: true,
              writable: false,
              configurable: false
            });
            const callableSet = Reflect.set(Object.keys, 'name', 'keys3');
            const callableDesc = Object.getOwnPropertyDescriptor(Object.keys, 'name');

            const map = new Map();
            const mapSym = Symbol('m');
            Object.defineProperty(map, 'size', {
              value: 99,
              enumerable: true,
              writable: false,
              configurable: false
            });
            Object.defineProperty(map, mapSym, {
              value: 'ms',
              enumerable: false,
              writable: false,
              configurable: false
            });
            const mapSet = Reflect.set(map, 'size', 100);
            const mapDelete = delete map.size;
            const mapDesc = Object.getOwnPropertyDescriptor(map, 'size');
            const mapSymDesc = Object.getOwnPropertyDescriptor(map, mapSym);

            document.getElementById('result').textContent = [
              Object.keys(arr).join(','),
              Object.getOwnPropertyNames(arr).join(','),
              Reflect.ownKeys(arr).map(describeKey).join(','),
              String(arr[0] === 7 && arr[1] === 2),
              String(arrSet === false && arrDelete === false),
              String(
                arrDesc.value === 7 &&
                arrDesc.writable === false &&
                arrDesc.enumerable === false &&
                arrDesc.configurable === false
              ),
              String(
                arrSymDesc.value === 'sa' &&
                arrSymDesc.writable === false &&
                arrSymDesc.enumerable === false &&
                arrSymDesc.configurable === false
              ),
              Object.keys(sample).join(','),
              String(sample.length === 9),
              String(fnSet === false && fnDelete === false),
              String(
                fnDesc.value === 9 &&
                fnDesc.writable === false &&
                fnDesc.enumerable === true &&
                fnDesc.configurable === false
              ),
              Object.keys(Object.keys).join(','),
              String(Object.keys.name === 'keys2'),
              String(callableSet),
              String(
                callableDesc.value === 'keys2' &&
                callableDesc.writable === false &&
                callableDesc.enumerable === true &&
                callableDesc.configurable === false
              ),
              Object.keys(map).join(','),
              Reflect.ownKeys(map).map(describeKey).join(','),
              String(map.size === 99),
              String(mapSet === false && mapDelete === false),
              String(
                mapDesc.value === 99 &&
                mapDesc.writable === false &&
                mapDesc.enumerable === true &&
                mapDesc.configurable === false
              ),
              String(
                mapSymDesc.value === 'ms' &&
                mapSymDesc.writable === false &&
                mapSymDesc.enumerable === false &&
                mapSymDesc.configurable === false
              )
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "1|0,1,length|0,1,length,sym:a,sym:b|true|true|true|true|length|true|true|true|name|true|false|true|size|size,sym:m|true|true|true|true",
    )?;
    Ok(())
}

#[test]
fn callable_delete_and_builtin_surface_residuals_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            function sample(a, b) {}
            Object.defineProperty(sample, 'length', {
              value: 9,
              enumerable: true,
              writable: false,
              configurable: true
            });
            const fnDelete = delete sample.length;
            const fnSet = Reflect.set(sample, 'length', 10);
            sample.length = 11;
            const fnDesc = Object.getOwnPropertyDescriptor(sample, 'length');

            Object.defineProperty(Object.keys, 'name', {
              value: 'keys2',
              enumerable: true,
              writable: false,
              configurable: true
            });
            const callableDelete = delete Object.keys.name;
            const callableSet = Reflect.set(Object.keys, 'name', 'keys3');
            Object.keys.name = 'keys4';
            const callableDesc = Object.getOwnPropertyDescriptor(Object.keys, 'name');

            const map = new Map([[1, 'a']]);
            Object.defineProperty(map, 'size', {
              value: 99,
              enumerable: true,
              writable: false,
              configurable: true
            });
            const mapDelete = delete map.size;
            const mapSet = Reflect.set(map, 'size', 100);
            map.size = 101;
            const mapDesc = Object.getOwnPropertyDescriptor(map, 'size');

            const re = /ab/g;
            Object.defineProperty(re, 'source', {
              value: 'override',
              enumerable: true,
              writable: false,
              configurable: true
            });
            const reDelete = delete re.source;
            const reSet = Reflect.set(re, 'source', 'again');
            re.source = 'later';
            const reDesc = Object.getOwnPropertyDescriptor(re, 'source');

            document.getElementById('result').textContent = [
              String(fnDelete),
              String(fnSet),
              String(fnDesc === undefined),
              Object.getOwnPropertyNames(sample).join(','),
              String(sample.length === 0),
              String(callableDelete),
              String(callableSet),
              String(callableDesc === undefined),
              Object.getOwnPropertyNames(Object.keys).join(','),
              String(Object.hasOwn(Object.keys, 'name') === false),
              String(Object.keys.name === ''),
              String(mapDelete),
              String(mapSet),
              String(mapDesc === undefined),
              Object.getOwnPropertyNames(map).join(','),
              String(Object.hasOwn(map, 'size') === false),
              String(map.size === 1),
              String(reDelete),
              String(reSet),
              String(reDesc === undefined),
              String(!Object.getOwnPropertyNames(re).includes('source')),
              String(Object.hasOwn(re, 'source') === false),
              String(re.source === 'ab')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|false|true|name,prototype|true|true|false|true|length|true|true|true|false|true||true|true|true|false|true|true|true|true",
    )?;
    Ok(())
}

#[test]
fn function_prototype_descriptor_and_write_parity_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            function Box() {}
            const replacement = { mark: 1 };

            const deleteBefore = delete Box.prototype;
            const reflectBefore = Reflect.set(Box, 'prototype', replacement);
            const descAfterSet = Object.getOwnPropertyDescriptor(Box, 'prototype');
            const instanceBefore = Object.getPrototypeOf(new Box()) === replacement;

            Object.defineProperty(Box, 'prototype', { writable: false });
            const descAfterFreeze = Object.getOwnPropertyDescriptor(Box, 'prototype');
            const reflectAfterFreeze = Reflect.set(Box, 'prototype', { mark: 2 });
            Box.prototype = { mark: 3 };
            const deleteAfterFreeze = delete Box.prototype;
            const instanceAfter = Object.getPrototypeOf(new Box()) === replacement;

            document.getElementById('result').textContent = [
              String(deleteBefore === false),
              String(reflectBefore === true),
              String(descAfterSet.value === replacement),
              String(
                descAfterSet.writable === true &&
                descAfterSet.enumerable === false &&
                descAfterSet.configurable === false
              ),
              String(Object.keys(Box).length === 0),
              String(Object.getOwnPropertyNames(Box).join(',') === 'length,name,prototype'),
              String(instanceBefore === true),
              String(descAfterFreeze.value === replacement),
              String(
                descAfterFreeze.writable === false &&
                descAfterFreeze.enumerable === false &&
                descAfterFreeze.configurable === false
              ),
              String(reflectAfterFreeze === false),
              String(Box.prototype === replacement),
              String(deleteAfterFreeze === false),
              String(instanceAfter === true)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|true|true|true|true|true|true|true|true|true|true|true|true",
    )?;
    Ok(())
}

#[test]
fn regexp_prototype_accessor_and_own_surface_parity_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = /ab/gi;
            const ownDesc = Object.getOwnPropertyDescriptor(re, 'source');
            const protoSourceDesc = Object.getOwnPropertyDescriptor(RegExp.prototype, 'source');
            const protoFlagsDesc = Object.getOwnPropertyDescriptor(RegExp.prototype, 'flags');
            const protoSourceGetter = protoSourceDesc.get;
            const protoFlagsGetter = protoFlagsDesc.get;

            const deleteInherited = delete re.source;
            const reflectInherited = Reflect.set(re, 'source', 'nope');
            re.source = 'still-nope';

            Object.defineProperty(re, 'source', {
              value: 'own',
              enumerable: true,
              configurable: true
            });
            const ownAfterDefine = Object.getOwnPropertyDescriptor(re, 'source');
            const namesAfterDefine = Object.getOwnPropertyNames(re).join(',');
            const deleteOwn = delete re.source;

            document.getElementById('result').textContent = [
              String(Object.getOwnPropertyNames(re).join(',') === 'lastIndex'),
              String(Reflect.ownKeys(re).join(',') === 'lastIndex'),
              String(Object.hasOwn(re, 'source') === false),
              String(ownDesc === undefined),
              String(
                typeof protoSourceGetter === 'function' &&
                protoSourceDesc.enumerable === false &&
                protoSourceDesc.configurable === true &&
                protoSourceDesc.set === undefined
              ),
              String(
                typeof protoFlagsGetter === 'function' &&
                protoFlagsDesc.enumerable === false &&
                protoFlagsDesc.configurable === true &&
                protoFlagsDesc.set === undefined
              ),
              String(protoSourceGetter.call(re) === 'ab'),
              String(protoFlagsGetter.call(re) === 'gi'),
              String(protoSourceGetter.call(RegExp.prototype) === '(?:)'),
              String(protoFlagsGetter.call(RegExp.prototype) === ''),
              String(deleteInherited === true),
              String(reflectInherited === false),
              String(re.source === 'ab'),
              String(namesAfterDefine === 'lastIndex,source'),
              String(
                ownAfterDefine.value === 'own' &&
                ownAfterDefine.enumerable === true &&
                ownAfterDefine.configurable === true
              ),
              String(deleteOwn === true),
              String(Object.getOwnPropertyNames(re).join(',') === 'lastIndex'),
              String(re.source === 'ab'),
              String(RegExp.prototype.toString() === '/(?:)/')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true",
    )?;
    Ok(())
}

#[test]
fn text_codec_instances_expose_prototype_backed_surface_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const encoder = new TextEncoder();
            const decoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });

            const encoderProto = TextEncoder.prototype;
            const decoderProto = TextDecoder.prototype;
            const encoderDesc = Object.getOwnPropertyDescriptor(encoderProto, 'encoding');
            const decoderDesc = Object.getOwnPropertyDescriptor(decoderProto, 'encoding');

            document.getElementById('result').textContent = [
              String(Object.getOwnPropertyNames(encoder).length),
              String(Object.keys(encoder).length),
              String(Reflect.ownKeys(encoder).length),
              String(
                typeof encoderDesc.get === 'function' &&
                encoderDesc.set === undefined &&
                encoderDesc.enumerable === false &&
                encoderDesc.configurable === true
              ),
              String(typeof Object.getOwnPropertyDescriptor(encoderProto, 'encode').value === 'function'),
              String(typeof Object.getOwnPropertyDescriptor(encoderProto, 'encodeInto').value === 'function'),
              String(Object.getOwnPropertyNames(decoder).length),
              String(Object.keys(decoder).length),
              String(Reflect.ownKeys(decoder).length),
              String(
                typeof decoderDesc.get === 'function' &&
                decoderDesc.set === undefined &&
                decoderDesc.enumerable === false &&
                decoderDesc.configurable === true
              ),
              String(typeof Object.getOwnPropertyDescriptor(decoderProto, 'fatal').get === 'function'),
              String(typeof Object.getOwnPropertyDescriptor(decoderProto, 'ignoreBOM').get === 'function'),
              String(typeof Object.getOwnPropertyDescriptor(decoderProto, 'decode').value === 'function'),
              String(decoder.encoding === 'utf-8' && decoder.fatal === true && decoder.ignoreBOM === true)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "0|0|0|true|true|true|0|0|0|true|true|true|true|true",
    )?;
    Ok(())
}

#[test]
fn non_configurable_redefine_invariant_sweep_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const accessorBox = {};
            const getter = () => 1;
            Object.defineProperty(accessorBox, 'x', { get: getter });
            const accessorDesc = Object.getOwnPropertyDescriptor(accessorBox, 'x');
            let accessorErr = '';
            try {
              Object.defineProperty(accessorBox, 'x', { set(v) {} });
            } catch (err) {
              accessorErr = String(err);
            }
            const accessorDelete = delete accessorBox.x;

            const dataBox = {};
            Object.defineProperty(dataBox, 'y', { value: 3 });
            const dataDesc = Object.getOwnPropertyDescriptor(dataBox, 'y');
            const dataSet = Reflect.set(dataBox, 'y', 4);
            const dataDelete = delete dataBox.y;
            let dataErr = '';
            try {
              Object.defineProperty(dataBox, 'y', { value: 4 });
            } catch (err) {
              dataErr = String(err);
            }

            const arr = [9];
            Object.defineProperty(arr, '0', { writable: false });
            const arrDesc = Object.getOwnPropertyDescriptor(arr, '0');
            const arrSet = Reflect.set(arr, '0', 10);
            let arrErr = '';
            try {
              Object.defineProperty(arr, '0', { value: 10 });
            } catch (err) {
              arrErr = String(err);
            }
            const arrAfterDesc = Object.getOwnPropertyDescriptor(arr, '0');

            const list = [1, 2, 3];
            Object.defineProperty(list, 'length', { writable: false });
            const lengthDesc = Object.getOwnPropertyDescriptor(list, 'length');
            const lengthSet = Reflect.set(list, 'length', 1);
            let lengthAccessorErr = '';
            try {
              Object.defineProperty(list, 'length', { get() { return 1; } });
            } catch (err) {
              lengthAccessorErr = String(err);
            }
            let lengthValueErr = '';
            try {
              Object.defineProperty(list, 'length', { value: 1 });
            } catch (err) {
              lengthValueErr = String(err);
            }

            const re = /ab/g;
            Object.defineProperty(re, 'lastIndex', { writable: false });
            const reDesc = Object.getOwnPropertyDescriptor(re, 'lastIndex');
            const reSet = Reflect.set(re, 'lastIndex', 3);
            let reAccessorErr = '';
            try {
              Object.defineProperty(re, 'lastIndex', { get() { return 1; } });
            } catch (err) {
              reAccessorErr = String(err);
            }
            let reValueErr = '';
            try {
              Object.defineProperty(re, 'lastIndex', { value: 2 });
            } catch (err) {
              reValueErr = String(err);
            }

            const map = new Map();
            Object.defineProperty(map, 'size', { value: 9 });
            const mapDesc = Object.getOwnPropertyDescriptor(map, 'size');
            const mapSet = Reflect.set(map, 'size', 10);
            const mapDelete = delete map.size;
            let mapErr = '';
            try {
              Object.defineProperty(map, 'size', { value: 10 });
            } catch (err) {
              mapErr = String(err);
            }

            document.getElementById('result').textContent = [
              String(
                typeof accessorDesc.get === 'function' &&
                accessorDesc.set === undefined &&
                accessorDesc.enumerable === false &&
                accessorDesc.configurable === false
              ),
              String(accessorErr.includes('Cannot redefine property: x')),
              String(accessorDelete === false),
              String(Object.getOwnPropertyNames(accessorBox).join(',') === 'x'),
              String(
                dataDesc.value === 3 &&
                dataDesc.writable === false &&
                dataDesc.enumerable === false &&
                dataDesc.configurable === false
              ),
              String(dataSet === false && dataDelete === false),
              String(dataErr.includes('Cannot redefine property: y')),
              String(
                arrDesc.value === 9 &&
                arrDesc.writable === false &&
                arrDesc.enumerable === true &&
                arrDesc.configurable === true
              ),
              String(arrSet === false),
              String(arrErr === ''),
              String(
                arr[0] === 10 &&
                arrAfterDesc.value === 10 &&
                arrAfterDesc.writable === false &&
                arrAfterDesc.enumerable === true &&
                arrAfterDesc.configurable === true
              ),
              String(
                lengthDesc.value === 3 &&
                lengthDesc.writable === false &&
                lengthDesc.enumerable === false &&
                lengthDesc.configurable === false
              ),
              String(lengthSet === false),
              String(
                lengthAccessorErr.includes('Cannot redefine property: length') &&
                lengthValueErr.includes('Cannot redefine property: length')
              ),
              String(
                reDesc.value === 0 &&
                reDesc.writable === false &&
                reDesc.enumerable === false &&
                reDesc.configurable === false
              ),
              String(reSet === false),
              String(
                reAccessorErr.includes('Cannot redefine property: lastIndex') &&
                reValueErr.includes('Cannot redefine property: lastIndex')
              ),
              String(
                mapDesc.value === 9 &&
                mapDesc.writable === false &&
                mapDesc.enumerable === false &&
                mapDesc.configurable === false
              ),
              String(mapSet === false && mapDelete === false),
              String(mapErr.includes('Cannot redefine property: size'))
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true",
    )?;
    Ok(())
}

#[test]
fn define_property_descriptor_object_coercion_parity_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const mixedSteps = [];
            const mixedProto = {
              get enumerable() { mixedSteps.push('enumerable'); return true; },
              get configurable() { mixedSteps.push('configurable'); return true; },
              get writable() { mixedSteps.push('writable'); return true; }
            };
            const mixedDesc = {
              __proto__: mixedProto,
              get value() { mixedSteps.push('value'); return 1; },
              get get() { mixedSteps.push('get'); return undefined; },
              get set() { mixedSteps.push('set'); return undefined; }
            };
            let mixedErr = '';
            try {
              Object.defineProperty({}, 'mixed', mixedDesc);
            } catch (err) {
              mixedErr = String(err);
            }

            const fnSteps = [];
            function descriptorFn() {}
            Object.defineProperty(descriptorFn, 'enumerable', {
              get() { fnSteps.push('enumerable'); return true; }
            });
            Object.defineProperty(descriptorFn, 'configurable', {
              get() { fnSteps.push('configurable'); return true; }
            });
            Object.defineProperty(descriptorFn, 'value', {
              get() { fnSteps.push('value'); return 9; }
            });

            const target = {};
            const returned = Object.defineProperty(target, 'fromFn', descriptorFn);
            const fnDesc = Object.getOwnPropertyDescriptor(target, 'fromFn');

            document.getElementById('result').textContent = [
              mixedSteps.join(','),
              String(mixedErr.includes('Invalid property descriptor')),
              String(returned === target),
              fnSteps.join(','),
              String(
                fnDesc.value === 9 &&
                fnDesc.writable === false &&
                fnDesc.enumerable === true &&
                fnDesc.configurable === true
              )
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "enumerable,configurable,value,writable,get,set|true|true|enumerable,configurable,value|true",
    )?;
    Ok(())
}

#[test]
fn define_property_accessor_undefined_parity_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = {};
            const defined = Object.defineProperty(box, 'ghost', {
              get: undefined,
              set: undefined,
              enumerable: true,
              configurable: true
            });
            const desc = Object.getOwnPropertyDescriptor(box, 'ghost');
            const namesBefore = Object.getOwnPropertyNames(box).join(',');
            const keysBefore = Object.keys(box).join(',');
            const ownKeysBefore = Reflect.ownKeys(box).join(',');
            const readBefore = box.ghost;
            box.ghost = 1;
            const readAfterAssign = box.ghost;
            const reflectSet = Reflect.set(box, 'ghost', 2);
            const deleted = delete box.ghost;
            const afterDelete = Object.getOwnPropertyDescriptor(box, 'ghost');

            document.getElementById('result').textContent = [
              String(defined === box),
              String(
                desc.get === undefined &&
                desc.set === undefined &&
                desc.enumerable === true &&
                desc.configurable === true
              ),
              namesBefore,
              keysBefore,
              ownKeysBefore,
              String(readBefore === undefined && readAfterAssign === undefined),
              String(reflectSet === false),
              String(deleted === true),
              String(afterDelete === undefined)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true|true|ghost|ghost|ghost|true|true|true|true")?;
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
fn object_prototype_to_string_and_value_of_receiver_paths_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { a: 1 };
            const toStringFn = obj['toString'];
            const valueOfFn = obj['valueOf'];
            let nullValueOf = 'none';
            try {
              Object.prototype.valueOf.call(null);
            } catch (e) {
              nullValueOf = String(e);
            }

            document.getElementById('result').textContent = [
              obj.toString(),
              obj['toString'](),
              String(obj),
              String(toStringFn === Object.prototype.toString),
              String(valueOfFn === Object.prototype.valueOf),
              String(valueOfFn.call(obj) === obj),
              Object.prototype.toString.call(null),
              Object.prototype.toString.call(undefined),
              Object.prototype.toString.call(1),
              Object.prototype.toString.call('x'),
              String(Function.prototype.toString.call(toStringFn) === toStringFn.toString()),
              String(nullValueOf.includes('Object.valueOf called on null or undefined'))
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "[object Object]|[object Object]|[object Object]|true|true|true|[object Null]|[object Undefined]|[object Number]|[object String]|true|true",
    )?;
    Ok(())
}

#[test]
fn define_property_boxed_descriptor_wrappers_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const boolSteps = [];
            const boolDesc = new Boolean(false);
            Object.defineProperty(boolDesc, 'enumerable', {
              get() { boolSteps.push('enumerable'); return true; }
            });
            Object.defineProperty(boolDesc, 'configurable', {
              get() { boolSteps.push('configurable'); return true; }
            });
            Object.defineProperty(boolDesc, 'value', {
              get() { boolSteps.push('value'); return 11; }
            });

            const dataTarget = {};
            Object.defineProperty(dataTarget, 'boxed', boolDesc);
            const dataDesc = Object.getOwnPropertyDescriptor(dataTarget, 'boxed');

            let assigned = 'none';
            const numSteps = [];
            const BoxedNumber = Number;
            const numDesc = new BoxedNumber(0);
            Object.defineProperty(numDesc, 'enumerable', {
              get() { numSteps.push('enumerable'); return true; }
            });
            Object.defineProperty(numDesc, 'configurable', {
              get() { numSteps.push('configurable'); return true; }
            });
            Object.defineProperty(numDesc, 'get', {
              get() { numSteps.push('get'); return undefined; }
            });
            Object.defineProperty(numDesc, 'set', {
              get() {
                numSteps.push('set');
                return function(v) { assigned = String(v); };
              }
            });

            const accessorTarget = {};
            Object.defineProperty(accessorTarget, 'ghost', numDesc);
            accessorTarget.ghost = 5;
            const accessorDesc = Object.getOwnPropertyDescriptor(accessorTarget, 'ghost');
            const viaObject = Object(1);
            const boxedNumber = new Number(2);
            const boxedNumberDetails = [
              String(Object.getPrototypeOf(viaObject) === Number.prototype),
              String(viaObject.valueOf()),
              viaObject.toString(),
              String(boxedNumber.valueOf()),
              boxedNumber.toFixed(1)
            ].join(':');

            document.getElementById('result').textContent = [
              boolSteps.join(','),
              String(
                dataDesc.value === 11 &&
                dataDesc.enumerable === true &&
                dataDesc.configurable === true &&
                dataDesc.writable === false
              ),
              numSteps.join(','),
              String(
                accessorTarget.ghost === undefined &&
                accessorDesc.get === undefined &&
                typeof accessorDesc.set === 'function' &&
                assigned === '5'
              ),
              Object.keys(accessorTarget).join(','),
              boxedNumberDetails
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "enumerable,configurable,value|true|enumerable,configurable,get,set|true|ghost|true:1:1:2:2.0",
    )?;
    Ok(())
}

#[test]
fn object_constructor_boxes_numbers_with_number_wrapper_surface_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const viaObject = Object(1);
            const boxedNumber = new Number(2);

            document.getElementById('result').textContent = [
              String(Object.getPrototypeOf(viaObject) === Number.prototype),
              String(viaObject.valueOf()),
              viaObject.toString(),
              String(boxedNumber.valueOf()),
              boxedNumber.toFixed(1)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true|1|1|2|2.0")?;
    Ok(())
}

#[test]
fn string_wrapper_reflective_surface_and_copy_paths_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const wrapped = Object('ab');
            wrapped.extra = 'x';
            const zero = Object.getOwnPropertyDescriptor(wrapped, '0');
            const lengthDesc = Object.getOwnPropertyDescriptor(wrapped, 'length');
            const assigned = Object.assign({}, wrapped);
            const spread = { ...wrapped };

            document.getElementById('result').textContent = [
              Object.getOwnPropertyNames(wrapped).join(','),
              Reflect.ownKeys(wrapped).join(','),
              Object.keys(wrapped).join(','),
              Object.values(wrapped).join(','),
              Object.entries(wrapped).map(([key, value]) => key + ':' + value).join(','),
              [zero.value, String(zero.writable), String(zero.enumerable), String(zero.configurable)].join(':'),
              [String(lengthDesc.value), String(lengthDesc.writable), String(lengthDesc.enumerable), String(lengthDesc.configurable)].join(':'),
              [String(Object.hasOwn(wrapped, '0')), String(Object.hasOwn(wrapped, 'length')), String(Object.hasOwn(wrapped, 'extra'))].join(':'),
              Object.keys(assigned).join(','),
              Object.values(assigned).join(','),
              Object.keys(spread).join(','),
              Object.values(spread).join(',')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "0,1,length,extra|0,1,length,extra|0,1,extra|a,b,x|0:a,1:b,extra:x|a:false:true:false|2:false:false:false|true:true:true|0,1,extra|a,b,x|0,1,extra|a,b,x",
    )?;
    Ok(())
}

#[test]
fn boxed_primitive_wrapper_tags_and_prototype_introspection_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const boolBox = Object(false);
            const numBox = Object(7);
            const bigBox = Object(7n);
            const sym = Symbol('s');
            const symBox = Object(sym);

            document.getElementById('result').textContent = [
              String(Object.getPrototypeOf(boolBox) === Boolean.prototype),
              String(Object.getPrototypeOf(numBox) === Number.prototype),
              String(Object.getPrototypeOf(bigBox) === BigInt.prototype),
              String(Object.getPrototypeOf(symBox) === Symbol.prototype),
              Object.prototype.toString.call(boolBox),
              Object.prototype.toString.call(numBox),
              Object.prototype.toString.call(bigBox),
              Object.prototype.toString.call(symBox),
              String(boolBox.constructor === Boolean),
              String(numBox.constructor === Number),
              String(bigBox.constructor === BigInt),
              String(symBox.constructor === Symbol),
              String(boolBox.valueOf()),
              String(numBox.valueOf()),
              String(bigBox.valueOf()),
              String(symBox.valueOf() === sym),
              String(Object.getOwnPropertyNames(boolBox).length),
              String(Object.getOwnPropertyNames(numBox).length),
              String(Object.getOwnPropertyNames(bigBox).length),
              String(Object.getOwnPropertyNames(symBox).length)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|true|true|true|[object Boolean]|[object Number]|[object BigInt]|[object Symbol]|true|true|true|true|false|7|7|true|0|0|0|0",
    )?;
    Ok(())
}

#[test]
fn string_wrapper_descriptor_invariants_and_override_attempts_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const wrapped = Object('ab');
            const receiver = {};

            wrapped[0] = 'x';
            wrapped.length = 9;

            const reflectSameIndex = Reflect.set(wrapped, '0', 'x');
            const reflectSameLength = Reflect.set(wrapped, 'length', 9);
            const reflectOtherIndex = Reflect.set(wrapped, '0', 'x', receiver);
            const reflectOtherLength = Reflect.set(wrapped, 'length', 9, receiver);

            const deleteIndex = delete wrapped[0];
            const deleteLength = delete wrapped.length;

            let sameIndex = 'ok';
            try {
              Object.defineProperty(wrapped, '0', {
                value: 'a',
                writable: false,
                enumerable: true,
                configurable: false
              });
            } catch (error) {
              sameIndex = String(error.message || error);
            }

            let badIndex = 'ok';
            try {
              Object.defineProperty(wrapped, '0', { value: 'x' });
            } catch (error) {
              badIndex = String(error.message || error);
            }

            let sameLength = 'ok';
            try {
              Object.defineProperty(wrapped, 'length', { value: 2 });
            } catch (error) {
              sameLength = String(error.message || error);
            }

            let badLength = 'ok';
            try {
              Object.defineProperty(wrapped, 'length', { value: 3 });
            } catch (error) {
              badLength = String(error.message || error);
            }

            Object.defineProperty(wrapped, '2', {
              value: 'c',
              writable: true,
              enumerable: true,
              configurable: true
            });

            const zero = Object.getOwnPropertyDescriptor(wrapped, '0');
            const lengthDesc = Object.getOwnPropertyDescriptor(wrapped, 'length');
            const two = Object.getOwnPropertyDescriptor(wrapped, '2');

            document.getElementById('result').textContent = [
              wrapped[0],
              String(wrapped.length),
              String(reflectSameIndex),
              String(reflectSameLength),
              String(reflectOtherIndex),
              String(reflectOtherLength),
              String(deleteIndex),
              String(deleteLength),
              [zero.value, String(zero.writable), String(zero.enumerable), String(zero.configurable)].join(':'),
              [String(lengthDesc.value), String(lengthDesc.writable), String(lengthDesc.enumerable), String(lengthDesc.configurable)].join(':'),
              [two.value, String(two.writable), String(two.enumerable), String(two.configurable)].join(':'),
              sameIndex,
              badIndex,
              sameLength,
              badLength,
              String(Object.hasOwn(receiver, '0')),
              String(Object.hasOwn(receiver, 'length')),
              Object.getOwnPropertyNames(wrapped).join(','),
              Reflect.ownKeys(wrapped).join(','),
              Object.keys(wrapped).join(','),
              Object.values(wrapped).join(',')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "a|2|false|false|false|false|false|false|a:false:true:false|2:false:false:false|c:true:true:true|ok|Cannot redefine property: 0|ok|Cannot redefine property: length|false|false|0,1,2,length|0,1,2,length|0,1,2|a,b,c",
    )?;
    Ok(())
}

#[test]
fn wrapper_prototype_mutation_and_string_exotic_introspection_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const wrapped = Object('ab');
            const boolBox = Object(false);

            const hasProtoMethodBefore = 'propertyIsEnumerable' in wrapped;
            const ownEnum0 = wrapped.propertyIsEnumerable('0');
            const ownEnumLength = wrapped.propertyIsEnumerable('length');
            const ownHas0 = wrapped.hasOwnProperty('0');
            const ownHas2 = wrapped.hasOwnProperty('2');
            const stringProtoBefore = String.prototype.isPrototypeOf(wrapped);

            const proto = {
              2: 'c',
              tail() {
                return this[2] + this[0];
              }
            };
            const boolProto = { flag: 7 };
            const setProto = Object.setPrototypeOf;
            const returned = setProto(wrapped, proto);
            Object.setPrototypeOf(boolBox, boolProto);

            const nullWrapped = Object('z');
            Object.setPrototypeOf(nullWrapped, null);

            document.getElementById('result').textContent = [
              String(hasProtoMethodBefore),
              String(ownEnum0),
              String(ownEnumLength),
              String(ownHas0),
              String(ownHas2),
              String(stringProtoBefore),
              String(returned === wrapped),
              String(Object.getPrototypeOf(wrapped) === proto),
              wrapped[0],
              wrapped[1],
              wrapped[2],
              wrapped.tail(),
              String('2' in wrapped),
              String('tail' in wrapped),
              String(String.prototype.isPrototypeOf(wrapped)),
              String(proto.isPrototypeOf(wrapped)),
              String(Object.getPrototypeOf(boolBox) === boolProto),
              String(boolBox.flag),
              String('0' in nullWrapped),
              String('toString' in nullWrapped),
              Object.getOwnPropertyNames(nullWrapped).join(','),
              Reflect.ownKeys(nullWrapped).join(','),
              Object.keys(nullWrapped).join(','),
              String(Object.prototype.propertyIsEnumerable.call(nullWrapped, '0')),
              String(Object.prototype.propertyIsEnumerable.call(nullWrapped, 'length'))
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|true|false|true|false|true|true|true|a|b|c|ca|true|true|false|true|true|7|true|false|0,length|0,length|0|true|false",
    )?;
    Ok(())
}

#[test]
fn wrapper_primitive_prototype_methods_survive_custom_prototype_chains_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const num = new Number(12.5);
            const customProto = {
              tail() {
                return Number.prototype.valueOf.call(this) + 4;
              }
            };
            Object.setPrototypeOf(customProto, Number.prototype);
            const setResult = Object.setPrototypeOf(num, customProto);

            const fixed = num.toFixed;
            const precision = Object.getPrototypeOf(num).toPrecision;
            const exponential = Object.getPrototypeOf(customProto).toExponential;

            const bool = Object(true);
            const boolProto = {
              shout() {
                return Boolean.prototype.toString.call(this).toUpperCase();
              }
            };
            Object.setPrototypeOf(boolProto, Boolean.prototype);
            Object.setPrototypeOf(bool, boolProto);

            const big = Object(255n);
            const bigProto = {
              hex() {
                return BigInt.prototype.toString.call(this, 16);
              }
            };
            Object.setPrototypeOf(bigProto, BigInt.prototype);
            Object.setPrototypeOf(big, bigProto);

            const sym = Object(Symbol('s'));
            const symProto = {
              desc() {
                return Symbol.prototype.toString.call(this);
              }
            };
            Object.setPrototypeOf(symProto, Symbol.prototype);
            Object.setPrototypeOf(sym, symProto);

            document.getElementById('result').textContent = [
              String(setResult === num),
              String(Object.getPrototypeOf(num) === customProto),
              String(customProto.isPrototypeOf(num)),
              String(Number.prototype.isPrototypeOf(num)),
              fixed.call(num, 1),
              precision.call(num, 4),
              exponential.call(num, 2),
              String(num.tail()),
              bool.shout(),
              big.hex(),
              sym.desc(),
              String(fixed.name === 'toFixed'),
              String(fixed.length === 1)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|true|true|true|12.5|12.50|1.25e+1|16.5|TRUE|ff|Symbol(s)|true|true",
    )?;
    Ok(())
}

#[test]
fn object_static_prototype_mutation_covers_functions_primitives_and_regexp_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            function fn() {}
            const proto = {
              mark: 7,
              ping() {
                return 'pong';
              }
            };
            const re = /a/;
            const reProto = { tag: 'rx' };
            const setProto = Object.setPrototypeOf;
            const getProto = Object.getPrototypeOf;

            const fnBefore = getProto(fn) === Function.prototype;
            const setFn = setProto(fn, proto);
            const fnAfter = getProto(fn) === proto;
            const fnMark = fn.mark === 7;
            const fnPing = fn.ping();
            const protoCheck = proto.isPrototypeOf(fn);

            setProto(fn, null);
            const fnNull = getProto(fn) === null;

            const primitiveNumber = setProto(1, proto) === 1;
            const primitiveBig = setProto(2n, null) === 2n;

            const ctorProto = { brand: 'ctor' };
            const setCtor = setProto(Boolean, ctorProto);
            const ctorAfter = getProto(Boolean) === ctorProto;
            const ctorBrand = Boolean.brand === 'ctor';

            const setRe = setProto(re, reProto);
            const reAfter = getProto(re) === reProto;
            const reTag = re.tag === 'rx';

            document.getElementById('result').textContent = [
              String(fnBefore),
              String(setFn === fn),
              String(fnAfter),
              String(fnMark),
              fnPing,
              String(protoCheck),
              String(fnNull),
              String(primitiveNumber),
              String(primitiveBig),
              String(setCtor === Boolean),
              String(ctorAfter),
              String(ctorBrand),
              String(setRe === re),
              String(reAfter),
              String(reTag)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true|true|true|true|pong|true|true|true|true|true|true|true|true|true|true",
    )?;
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

#[test]
fn storage_member_chain_and_extra_arg_parity_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            localStorage.clear();

            let side = 'start';
            const localSet = [
              String(({ store: localStorage }).store.setItem('a', '1', side = 'local.setItem')),
              side,
              localStorage.getItem('a')
            ].join(',');

            side = 'start';
            const localGet = [
              ({ nested: { store: localStorage } }).nested.store.getItem('a', side = 'local.getItem'),
              side
            ].join(',');

            side = 'start';
            const localKey = [
              ({ nested: { store: localStorage } }).nested.store.key(0, side = 'local.key'),
              side
            ].join(',');

            side = 'start';
            ({ store: localStorage }).store.removeItem('a', side = 'local.removeItem');
            const localRemove = [
              String(localStorage.getItem('a')),
              side,
              localStorage.length
            ].join(',');

            localStorage.setItem('x', '1');
            localStorage.setItem('y', '2');
            side = 'start';
            const localClear = [
              String(({ nested: { store: localStorage } }).nested.store.clear(side = 'local.clear')),
              side,
              localStorage.length
            ].join(',');

            document.getElementById('result').textContent = [
              localSet,
              localGet,
              localKey,
              localRemove,
              localClear
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "undefined,local.setItem,1|1,local.getItem|a,local.key|null,local.removeItem,0|undefined,local.clear,0",
    )?;
    Ok(())
}

#[test]
fn storage_extracted_method_call_parity_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            localStorage.clear();

            let side = 'start';
            const setItem = localStorage.setItem;
            const localSet = [
              String(setItem.call(localStorage, 'a', '1', side = 'storage.set.call')),
              side,
              localStorage.getItem('a')
            ].join(',');

            side = 'start';
            const getItem = localStorage.getItem;
            const localGet = [
              getItem.call(localStorage, 'a', side = 'storage.get.call'),
              side
            ].join(',');

            side = 'start';
            const key = localStorage.key;
            const localKey = [
              key.call(localStorage, 0, side = 'storage.key.call'),
              side
            ].join(',');

            localStorage.setItem('b', '2');
            side = 'start';
            const clear = localStorage.clear;
            const localClear = [
              String(clear.call(localStorage, side = 'storage.clear.call')),
              side,
              localStorage.length
            ].join(',');

            document.getElementById('result').textContent = [
              localSet,
              localGet,
              localKey,
              localClear
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "undefined,storage.set.call,1|1,storage.get.call|a,storage.key.call|undefined,storage.clear.call,0",
    )?;
    Ok(())
}

#[test]
fn string_nodelist_and_boxed_prototype_property_paths_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <ul>
          <li>alpha</li>
          <li>beta</li>
        </ul>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const text = 'Hello';
            const textResult = [
              text['slice'].call(text, 1, 4),
              Array.from(text[Symbol.iterator].call(text)).join(','),
              text.constructor.prototype.slice.call(text, 2),
              Array.from(text.constructor.prototype[Symbol.iterator].call(text)).join(',')
            ].join(';');

            const items = document.querySelectorAll('li');
            const seen = [];
            items['forEach'].call(items, (node) => seen.push(node.textContent));
            const listResult = [
              items['item'].call(items, 1).textContent,
              seen.join(','),
              Array.from(items['keys'].call(items)).join(','),
              Array.from(items['entries'].call(items))
                .map((pair) => pair[0] + ':' + pair[1].textContent)
                .join(','),
              Array.from(items[Symbol.iterator].call(items))
                .map((node) => node.textContent)
                .join(',')
            ].join(';');

            const flag = false;
            const number = 255;
            const big = 255n;
            const sym = Symbol('id');
            const primitiveResult = [
              flag.constructor.prototype.toString.call(flag),
              String(flag.constructor.prototype.valueOf.call(flag)),
              number.constructor.prototype.toString.call(number, 16),
              String(number.constructor.prototype.valueOf.call(number)),
              big.constructor.prototype.toString.call(big, 16),
              String(big.constructor.prototype.valueOf.call(big)),
              sym.constructor.prototype.toString.call(sym),
              sym.constructor.prototype.valueOf.call(sym).description
            ].join(';');

            document.getElementById('result').textContent = [
              textResult,
              listResult,
              primitiveResult
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "ell;H,e,l,l,o;llo;H,e,l,l,o|beta;alpha,beta;0,1;0:alpha,1:beta;alpha,beta|false;false;ff;255;ff;255;Symbol(id);id",
    )?;
    Ok(())
}

#[test]
fn constructor_identity_and_string_raw_getter_breadth_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const number = 255;
            const big = 255n;
            const sym = Symbol('id');
            const ctorResult = [
              typeof Number,
              typeof BigInt,
              typeof Symbol,
              String(window.Number === Number),
              String(globalThis.BigInt === BigInt),
              String(self.Symbol === Symbol),
              String(number.constructor === Number),
              String(big.constructor === BigInt),
              String(sym.constructor === Symbol),
              String(number.constructor.prototype === Number.prototype),
              String(big.constructor.prototype === BigInt.prototype),
              String(Number.call(undefined, '12.5')),
              String(BigInt.call(undefined, '12'))
            ].join(';');

            const text = 'bananas';
            const stringResult = [
              String(text['includes'].call(text, 'ana')),
              String(text['startsWith'].call(text, 'na', 2)),
              String(text['endsWith'].call(text, 'nas')),
              text['substring'].call(text, 1, 4),
              text['split'].call(text, 'n').join(','),
              String(text.constructor.prototype.includes.call(text, 'ana')),
              text.constructor.prototype.substring.call(text, 2, 5),
              text.constructor.prototype.split.call(text, 'a', 3).join(',')
            ].join(';');

            document.getElementById('result').textContent = [
              ctorResult,
              stringResult
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "function;function;function;true;true;true;true;true;true;true;true;12.5;12|true;true;true;ana;ba,a,as;true;nan;b,n,n",
    )?;
    Ok(())
}

#[test]
fn object_backed_host_callable_name_length_and_source_text_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const hostFetch = window['fetch'];
            const checks = [
              [window.Event, 'Event', '1'],
              [window.EventTarget, 'EventTarget', '0'],
              [window.KeyboardEvent, 'KeyboardEvent', '1'],
              [window.WheelEvent, 'WheelEvent', '1'],
              [window.Document, 'Document', '0'],
              [window.Document.parseHTML, 'parseHTML', '1'],
              [window.Document.parseHTMLUnsafe, 'parseHTMLUnsafe', '1'],
              [hostFetch, 'fetch', '1'],
              [window.TextEncoder, 'TextEncoder', '0'],
              [window.TextDecoder, 'TextDecoder', '0']
            ];

            const result = checks.map(([value, name, length]) => [
              String(value.name === name),
              String(String(value.length) === length),
              String(value.toString().includes(name)),
              String(Function.prototype.toString.call(value) === value.toString()),
              String(String(value) === value.toString())
            ].join(':')).join('|');

            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn string_search_and_padding_raw_getter_metadata_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const proto = String.prototype;
            const checks = [
              [proto.charAt, 'charAt', '1'],
              [proto.at, 'at', '1'],
              [proto.search, 'search', '1'],
              [proto.match, 'match', '1'],
              [proto.matchAll, 'matchAll', '1'],
              [proto.replace, 'replace', '2'],
              [proto.replaceAll, 'replaceAll', '2'],
              [proto.localeCompare, 'localeCompare', '1'],
              [proto.trim, 'trim', '0'],
              [proto.toUpperCase, 'toUpperCase', '0'],
              [proto.isWellFormed, 'isWellFormed', '0'],
              [proto.toWellFormed, 'toWellFormed', '0'],
              [proto.indexOf, 'indexOf', '1'],
              [proto.lastIndexOf, 'lastIndexOf', '1'],
              [proto.padStart, 'padStart', '1'],
              [proto.padEnd, 'padEnd', '1'],
              [proto.repeat, 'repeat', '1']
            ];

            const result = checks.map(([value, name, length]) => [
              String(value.name === name),
              String(String(value.length) === length),
              String(value.toString().includes(name)),
              String(Function.prototype.toString.call(value) === value.toString()),
              String(value === proto[name]),
              String(value === String.prototype[name])
            ].join(':')).join('|');

            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true|true:true:true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn regexp_symbol_method_raw_getter_metadata_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const checks = [
              [Symbol.match, '[Symbol.match]', '1'],
              [Symbol.matchAll, '[Symbol.matchAll]', '1'],
              [Symbol.replace, '[Symbol.replace]', '2'],
              [Symbol.search, '[Symbol.search]', '1'],
              [Symbol.split, '[Symbol.split]', '2']
            ];

            const result = checks.map(([symbol, name, length]) => {
              const value = RegExp.prototype[symbol];
              return [
                String(value.name === name),
                String(String(value.length) === length),
                String(value.toString().includes(name)),
                String(Function.prototype.toString.call(value) === value.toString()),
                String(value === RegExp.prototype[symbol])
              ].join(':');
            }).join('|');

            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true|true:true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn stable_constructor_prototype_identity_and_symbol_bracket_access_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const holder = { String, Symbol, Int8Array };
            const text = 'ok';
            const sym = Symbol('id');
            const typed = new Int8Array([1, 2]);
            const result = [
              String(String['prototype'] === String['prototype']),
              String(text.constructor.prototype === String['prototype']),
              String(holder.String['prototype'] === String['prototype']),
              String(Symbol['prototype'] === Symbol['prototype']),
              String(sym.constructor.prototype === Symbol['prototype']),
              String(holder.Symbol['prototype'] === Symbol['prototype']),
              String(Int8Array['prototype'] === Int8Array['prototype']),
              String(typed.constructor.prototype === Int8Array['prototype']),
              String(holder.Int8Array['prototype'] === Int8Array['prototype']),
              typeof holder.Symbol['iterator'],
              String(holder.Symbol['iterator'] === Symbol.iterator),
              String(holder.Symbol['for']('token') === Symbol.for('token')),
              holder.Symbol['keyFor'](Symbol.for('token'))
            ].join(';');

            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true;true;true;true;true;true;true;true;true;symbol;true;true;token",
    )?;
    Ok(())
}

#[test]
fn function_constructor_name_and_callable_prototype_chain_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const dynamic = new Function('a', 'b', 'return a + b;');
          const sample = function named() {};
          const fnProto = Object.getPrototypeOf(sample);
          document.getElementById('result').textContent = [
            dynamic.name,
            String(dynamic.length),
            String(Object.getPrototypeOf(dynamic) === fnProto),
            dynamic.constructor.name,
            String(dynamic.constructor.prototype === fnProto),
            String(dynamic instanceof Object),
            String(Object.getPrototypeOf(Map) === fnProto),
            Map.constructor.name,
            String(Map instanceof Object),
            String(String.constructor.name)
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "anonymous|2|true|Function|true|true|true|Function|true|Function",
    )?;
    Ok(())
}

#[test]
fn object_like_prototype_targets_preserve_inherited_lookup_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const arrProto = [];
          arrProto.extra = 'array';
          const fnProto = function protoFn() {};
          fnProto.fnMarker = 'fn';
          Object.setPrototypeOf(arrProto, fnProto);

          const plain = {};
          Object.setPrototypeOf(plain, arrProto);

          const ctorTarget = {};
          Object.setPrototypeOf(ctorTarget, Int8Array);

          document.getElementById('result').textContent = [
            String(Object.getPrototypeOf(plain) === arrProto),
            String(Object.getPrototypeOf(arrProto) === fnProto),
            plain.extra,
            plain.fnMarker,
            String(typeof plain.call === 'function'),
            String(typeof plain.push === 'undefined'),
            String(Object.getPrototypeOf(ctorTarget) === Int8Array),
            String(typeof ctorTarget.of === 'function'),
            String(ctorTarget.BYTES_PER_ELEMENT === 1)
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true|true|array|fn|true|true|true|true|true")?;
    Ok(())
}

#[test]
fn collection_and_regexp_explicit_prototype_override_disables_builtin_fast_paths_work() -> Result<()>
{
    let html = r#"
        <p id='result'></p>
        <script>
          const customProto = {
            marker: 'override',
            get() { return 'map-custom'; },
            has() { return 'set-custom'; },
            exec() { return 'regexp-custom'; }
          };

          const map = new Map([['k', 1]]);
          const set = new Set([1]);
          const re = /ab/g;

          Object.setPrototypeOf(map, customProto);
          Object.setPrototypeOf(set, customProto);
          Object.setPrototypeOf(re, customProto);

          document.getElementById('result').textContent = [
            String(Object.getPrototypeOf(map) === customProto),
            map.marker,
            String(map.size === undefined),
            String(map.get() === 'map-custom'),
            String(Object.getPrototypeOf(set) === customProto),
            set.marker,
            String(set.size === undefined),
            String(set.has() === 'set-custom'),
            String(Object.getPrototypeOf(re) === customProto),
            re.marker,
            String(re.lastIndex === undefined),
            String(re.exec() === 'regexp-custom')
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true|override|true|true|true|override|true|true|true|override|true|true",
    )?;
    Ok(())
}

#[test]
fn non_ordinary_prototype_traversal_preserves_receiver_and_hides_instance_state_work() -> Result<()>
{
    let html = r#"
        <p id='result'></p>
        <script>
          const mapProto = new Map([['k', 1]]);
          mapProto.extra = 'map';
          const setProto = new Set([1]);
          setProto.extra = 'set';
          const regexpProto = /ab/gi;
          regexpProto.extra = 'regexp';
          const promiseProto = Promise.resolve(1);
          const blobProto = new Blob(['hi'], { type: 'text/plain' });
          const bufferProto = new ArrayBuffer(4);

          const mapTarget = {};
          Object.setPrototypeOf(mapTarget, mapProto);
          const setTarget = {};
          Object.setPrototypeOf(setTarget, setProto);
          const regexpTarget = {};
          Object.setPrototypeOf(regexpTarget, regexpProto);
          const promiseTarget = {};
          Object.setPrototypeOf(promiseTarget, promiseProto);
          const blobTarget = {};
          Object.setPrototypeOf(blobTarget, blobProto);
          const bufferTarget = {};
          Object.setPrototypeOf(bufferTarget, bufferProto);

          let regexpSource = '';
          let mapGetCall = '';
          let promiseThenCall = '';
          try {
            regexpSource = regexpTarget.source;
          } catch (error) {
            regexpSource = String(error);
          }
          try {
            mapTarget.get('k');
          } catch (error) {
            mapGetCall = String(error);
          }
          try {
            promiseTarget.then(() => {});
          } catch (error) {
            promiseThenCall = String(error);
          }

          document.getElementById('result').textContent = [
            mapTarget.extra,
            String(mapTarget.size === undefined),
            String(typeof mapTarget.get === 'function'),
            String(String(mapGetCall).includes('Map method called on incompatible receiver')),
            setTarget.extra,
            String(setTarget.size === undefined),
            String(typeof setTarget.has === 'function'),
            regexpTarget.extra,
            String(regexpTarget.lastIndex === undefined),
            String(String(regexpSource).includes('RegExp method called on incompatible receiver')),
            String(typeof promiseTarget.then === 'function'),
            String(promiseTarget.status === undefined),
            String(String(promiseThenCall).includes('Promise method called on incompatible receiver')),
            String(typeof blobTarget.slice === 'function'),
            String(blobTarget.size === undefined),
            String(typeof bufferTarget.slice === 'function'),
            String(bufferTarget.byteLength === undefined)
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "map|true|true|true|set|true|true|regexp|true|true|true|true|true|true|true|true|true",
    )?;
    Ok(())
}

#[test]
fn object_like_array_and_function_prototype_lookup_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const arrProto = [];
          arrProto.extra = 'array';
          const fnProto = function protoFn() {};
          fnProto.fnMarker = 'fn';
          Object.setPrototypeOf(arrProto, fnProto);

          const plain = {};
          Object.setPrototypeOf(plain, arrProto);

          document.getElementById('result').textContent = [
            String(Object.getPrototypeOf(plain) === arrProto),
            String(Object.getPrototypeOf(arrProto) === fnProto),
            plain.extra,
            plain.fnMarker
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true|true|array|fn")?;
    Ok(())
}

#[test]
fn object_like_function_prototype_surface_lookup_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const arrProto = [];
          const fnProto = function protoFn() {};
          Object.setPrototypeOf(arrProto, fnProto);

          const plain = {};
          Object.setPrototypeOf(plain, arrProto);

          document.getElementById('result').textContent = [
            String(typeof plain.call === 'function')
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn object_like_function_prototype_missing_method_lookup_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const arrProto = [];
          const fnProto = function protoFn() {};
          Object.setPrototypeOf(arrProto, fnProto);

          const plain = {};
          Object.setPrototypeOf(plain, arrProto);

          document.getElementById('result').textContent = [
            String(typeof plain.push === 'undefined')
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn object_like_constructor_prototype_lookup_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const target = {};
          Object.setPrototypeOf(target, Int8Array);

          document.getElementById('result').textContent = [
            String(Object.getPrototypeOf(target) === Int8Array),
            String(typeof target.of === 'function'),
            String(target.BYTES_PER_ELEMENT === 1)
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true|true|true")?;
    Ok(())
}

#[test]
fn host_event_method_descriptors_and_delete_shadowing_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const event = new Event('click', { cancelable: true });
          const eventDesc = Object.getOwnPropertyDescriptor(event, 'preventDefault');
          const eventEnumerable = eventDesc.enumerable;
          const eventKeysHasMethod = Object.keys(event).includes('preventDefault');
          eventDesc.value.call(event);
          const eventDeleted = delete event.preventDefault;
          const eventGone = event.preventDefault === undefined;
          Object.defineProperty(event, 'preventDefault', {
            value() { return 'shadow'; },
            configurable: true
          });
          const eventOverride = event.preventDefault();

          const keyboard = new KeyboardEvent('keydown', { ctrlKey: true });
          const keyDesc = Object.getOwnPropertyDescriptor(keyboard, 'getModifierState');
          const keyEnumerable = keyDesc.enumerable;
          const keyValue = keyDesc.value.call(keyboard, 'Control');
          const keyDeleted = delete keyboard.getModifierState;
          const keyGone = keyboard.getModifierState === undefined;

          const pointer = new PointerEvent('pointerdown');
          const pointerDesc = Object.getOwnPropertyDescriptor(pointer, 'getCoalescedEvents');
          const pointerEnumerable = pointerDesc.enumerable;
          const pointerValue = Array.isArray(pointerDesc.value.call(pointer));
          const pointerDeleted = delete pointer.getCoalescedEvents;
          const pointerGone = pointer.getCoalescedEvents === undefined;

          const navigate = new NavigateEvent('navigate', { canIntercept: true });
          const interceptDesc = Object.getOwnPropertyDescriptor(navigate, 'intercept');
          const scrollDesc = Object.getOwnPropertyDescriptor(navigate, 'scroll');
          const navigateEnumerable = interceptDesc.enumerable && scrollDesc.enumerable;
          const navigateDeleted = delete navigate.intercept;
          const navigateGone = navigate.intercept === undefined;
          const scrollDeleted = delete navigate.scroll;
          const scrollGone = navigate.scroll === undefined;

          document.getElementById('result').textContent = [
            eventDesc.value.name,
            eventDesc.value.length,
            eventEnumerable,
            eventKeysHasMethod,
            event.defaultPrevented,
            eventDeleted,
            eventGone,
            eventOverride,
            keyDesc.value.name,
            keyDesc.value.length,
            keyEnumerable,
            keyValue,
            keyDeleted,
            keyGone,
            pointerDesc.value.name,
            pointerDesc.value.length,
            pointerEnumerable,
            pointerValue,
            pointerDeleted,
            pointerGone,
            interceptDesc.value.name,
            interceptDesc.value.length,
            scrollDesc.value.name,
            scrollDesc.value.length,
            navigateEnumerable,
            navigateDeleted,
            navigateGone,
            scrollDeleted,
            scrollGone
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "preventDefault|0|false|false|true|true|true|shadow|getModifierState|1|false|true|true|true|getCoalescedEvents|0|false|true|true|true|intercept|1|scroll|0|false|true|true|true|true",
    )?;
    Ok(())
}
