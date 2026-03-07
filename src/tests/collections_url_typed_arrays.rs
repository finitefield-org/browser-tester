use super::*;

#[test]
fn string_trim_and_case_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const raw = '  AbC ';
            const trimmed = raw.trim();
            const trimmedStart = raw.trimStart();
            const trimmedEnd = raw.trimEnd();
            const upper = raw.toUpperCase();
            const lower = raw.toLowerCase();
            const literal = ' z '.trim();
            document.getElementById('result').textContent =
              '[' + trimmed + ']|[' + trimmedStart + ']|[' + trimmedEnd + ']|[' +
              upper + ']|[' + lower + ']|[' + literal + ']';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "[AbC]|[AbC ]|[  AbC]|[  ABC ]|[  abc ]|[z]")?;
    Ok(())
}

#[test]
fn string_includes_prefix_suffix_and_index_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = 'hello world';
            const includes1 = text.includes('lo');
            const includes2 = text.includes('lo', 4);
            const includes3 = 'abc'.includes('a', -2);
            const starts1 = text.startsWith('hello');
            const starts2 = text.startsWith('world', 6);
            const starts3 = 'abc'.startsWith('a');
            const ends1 = text.endsWith('world');
            const ends2 = text.endsWith('hello', 5);
            const index1 = text.indexOf('o');
            const index2 = text.indexOf('o', 5);
            const index3 = text.indexOf('x');
            const index4 = text.indexOf('', 2);
            document.getElementById('result').textContent =
              includes1 + ':' + includes2 + ':' + includes3 + ':' +
              starts1 + ':' + starts2 + ':' + starts3 + ':' +
              ends1 + ':' + ends2 + ':' +
              index1 + ':' + index2 + ':' + index3 + ':' + index4;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:false:true:true:true:true:true:true:4:7:-1:2",
    )?;
    Ok(())
}

#[test]
fn string_slice_substring_split_and_replace_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = '012345';
            const s1 = text.slice(1, 4);
            const s2 = text.slice(-2);
            const s3 = text.slice(4, 1);
            const sub1 = text.substring(1, 4);
            const sub2 = text.substring(4, 1);
            const sub3 = text.substring(-2, 2);
            const split1 = 'a,b,c'.split(',');
            const split2 = 'abc'.split('');
            const split3 = 'a,b,c'.split(',', 2);
            const split4 = 'abc'.split();
            const rep1 = 'foo foo'.replace('foo', 'bar');
            const rep2 = 'abc'.replace('', '-');
            document.getElementById('result').textContent =
              s1 + ':' + s2 + ':' + s3.length + ':' +
              sub1 + ':' + sub2 + ':' + sub3 + ':' +
              split1.join('-') + ':' + split2.join('|') + ':' + split3.join(':') + ':' +
              split4.length + ':' + rep1 + ':' + rep2;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "123:45:0:123:123:01:a-b-c:a|b|c:a:b:1:bar foo:-abc",
    )?;
    Ok(())
}

#[test]
fn string_constructor_and_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const string1 = "A string primitive";
            const string2 = String(1);
            const string3 = String(true);
            const string4 = new String("A String object");
            const types =
              typeof string1 + ':' + typeof string2 + ':' + typeof string3 + ':' + typeof string4;
            const ctor = string4.constructor === String;
            const value = string4.valueOf();
            const rendered = string4.toString();
            const fromChar = String.fromCharCode(65, 66, 67);
            const fromCode = String.fromCodePoint(0x1F600);
            const raw = String.raw({ raw: ['Hi\\n', '!'] }, 'Bob');
            const symbolText = String(Symbol('token'));
            document.getElementById('result').textContent =
              types + '|' + ctor + '|' + value + '|' + rendered + '|' +
              fromChar + '|' + (fromCode.length > 0) + '|' + raw + '|' + symbolText;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "string:string:string:object|true|A String object|A String object|ABC|true|Hi\\nBob!|Symbol(token)",
    )?;
    Ok(())
}

#[test]
fn string_from_char_code_examples_and_uint16_coercion_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sample = String.fromCharCode(189, 43, 190, 61);
            const sampleCodes =
              sample.charCodeAt(0) + ',' + sample.charCodeAt(1) + ',' +
              sample.charCodeAt(2) + ',' + sample.charCodeAt(3);

            const abc = String.fromCharCode(65, 66, 67);

            const emDash = String.fromCharCode(0x2014);
            const emDashFromLarge = String.fromCharCode(0x12014);
            const emDashFromDecimal = String.fromCharCode(8212);
            const truncationOk =
              emDash.charCodeAt(0) === 0x2014 &&
              emDashFromLarge === emDash &&
              emDashFromDecimal === emDash;

            const night = String.fromCharCode(0xD83C, 0xDF03);
            const surrogateOk =
              night.length === 2 &&
              night.charCodeAt(0) === 0xD83C &&
              night.charCodeAt(1) === 0xDF03;

            const inf = String.fromCharCode(Infinity).charCodeAt(0);
            const negInf = String.fromCharCode(-Infinity).charCodeAt(0);
            const nan = String.fromCharCode(NaN).charCodeAt(0);
            const undef = String.fromCharCode(undefined).charCodeAt(0);
            const emptyLength = String.fromCharCode().length;

            document.getElementById('result').textContent =
              sampleCodes + '|' + abc + '|' + truncationOk + '|' + surrogateOk + '|' +
              inf + ':' + negInf + ':' + nan + ':' + undef + ':' + emptyLength;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "189,43,190,61|ABC|true|true|0:0:0:0:0")?;
    Ok(())
}

#[test]
fn string_from_char_code_surrogate_code_unit_semantics_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const high = String.fromCharCode(0xD800);
            const pair = String.fromCharCode(0xD83D, 0xDE00);
            const fromCp = String.fromCodePoint(0x1F600);
            const a = high.length === 1;
            const b = high.charCodeAt(0) === 0xD800;
            const c = high.codePointAt(0) === 0xD800;
            const d = pair.length === 2;
            const e = pair.charCodeAt(0) === 0xD83D && pair.charCodeAt(1) === 0xDE00;
            const f = pair.codePointAt(0) === 0x1F600 && pair.codePointAt(1) === 0xDE00;
            const g = /\uD800/.test(high) && /\uD800/u.test(high);
            const h = /\u{D800}/u.test(high);
            const i = fromCp.length === 2;
            const j = fromCp.charCodeAt(0) === 0xD83D && fromCp.charCodeAt(1) === 0xDE00;
            const k = /\uD83D\uDE00/.test(fromCp);
            const l = /^\u{1F600}$/u.test(fromCp);
            const m = fromCp === pair;
            const n = /^.$/u.test(fromCp);
            const o = !/^.$/.test(fromCp) && /^..$/.test(fromCp);
            const p = /^[\u{1F600}]$/u.test(fromCp);
            const q = /[\uD83D]/.test(fromCp) && !/[\uD83D]/u.test(fromCp);
            const r = /\p{RGI_Emoji}/v.test(fromCp) && /[\p{RGI_Emoji}]/v.test(fromCp);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' + h + ':' +
              i + ':' + j + ':' + k + ':' + l + ':' + m + ':' + n + ':' + o + ':' + p + ':' + q + ':' + r;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn string_well_formed_methods_handle_lone_surrogates() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const high = String.fromCharCode(0xD800);
            const low = String.fromCharCode(0xDC00);
            const pair = String.fromCharCode(0xD83D, 0xDE00);
            const mixed = 'A' + high + 'B' + low + 'C';
            const fixed = mixed.toWellFormed();

            const a = !high.isWellFormed();
            const b = !low.isWellFormed();
            const c = pair.isWellFormed();
            const d = !mixed.isWellFormed();
            const e = fixed.charCodeAt(1) === 0xFFFD;
            const f = fixed.charCodeAt(3) === 0xFFFD;
            const g = fixed.charAt(0) === 'A' && fixed.charAt(2) === 'B' && fixed.charAt(4) === 'C';
            const h = pair.toWellFormed().isWellFormed();
            const i = String.fromCharCode(0xD800, 0xDC00).isWellFormed();
            const j = String.fromCharCode(0xDC00, 0xD800).toWellFormed().charCodeAt(0) === 0xFFFD &&
                      String.fromCharCode(0xDC00, 0xD800).toWellFormed().charCodeAt(1) === 0xFFFD;
            const k = /\uD83D\uDE00/.test(pair.toWellFormed());

            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' +
              f + ':' + g + ':' + h + ':' + i + ':' + j + ':' + k;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:true:true:true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn string_extended_instance_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = 'cat';
            const charAt = text.charAt(1);
            const charCodeAt = text.charCodeAt(1);
            const codePointAt = text.codePointAt(1);
            const at = text.at(-1);
            const concat = text.concat('s', '!');
            const lastIndex1 = 'bananas'.lastIndexOf('an');
            const lastIndex2 = 'bananas'.lastIndexOf('an', 3);
            const searchRegex = 'abc123'.search(/[0-9]+/);
            const searchString = 'abc'.search('d');
            const replaceAll = 'foo foo'.replaceAll('foo', 'bar');
            const replaceAllRegex = 'a1b2c3'.replaceAll(/[0-9]/g, '');
            const repeated = 'ha'.repeat(3);
            const paddedStart = '5'.padStart(3, '0');
            const paddedEnd = '5'.padEnd(3, '0');
            const localeUpper = 'abc'.toLocaleUpperCase();
            const localeLower = 'ABC'.toLocaleLowerCase();
            const wellFormed = 'ok'.isWellFormed();
            const toWellFormed = 'ok'.toWellFormed();
            document.getElementById('result').textContent =
              charAt + ':' + charCodeAt + ':' + codePointAt + ':' + at + ':' +
              concat + ':' + lastIndex1 + ':' + lastIndex2 + ':' +
              searchRegex + ':' + searchString + ':' +
              replaceAll + ':' + replaceAllRegex + ':' +
              repeated + ':' + paddedStart + ':' + paddedEnd + ':' +
              localeUpper + ':' + localeLower + ':' + wellFormed + ':' + toWellFormed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "a:97:97:t:cats!:3:1:3:-1:bar bar:abc:hahaha:005:500:ABC:abc:true:ok",
    )?;
    Ok(())
}

#[test]
fn string_locale_compare_and_character_access_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const de = 'ä'.localeCompare('z', 'de');
            const sv = 'ä'.localeCompare('z', 'sv');
            const word = 'cat';
            const charAt = word.charAt(1);
            const bracket = word[1];
            const less = 'a' < 'b';
            const eq = 'HELLO'.toLowerCase() === 'hello';
            document.getElementById('result').textContent =
              (de < 0) + ':' + (sv > 0) + ':' + charAt + ':' + bracket + ':' + less + ':' + eq;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:a:a:true:true")?;
    Ok(())
}

#[test]
fn string_method_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>'x'.trim(1);</script>",
            "trim does not take arguments",
        ),
        (
            "<script>'x'.toUpperCase(1);</script>",
            "toUpperCase does not take arguments",
        ),
        (
            "<script>'x'.includes();</script>",
            "String.includes requires one or two arguments",
        ),
        (
            "<script>'x'.startsWith();</script>",
            "startsWith requires one or two arguments",
        ),
        (
            "<script>'x'.endsWith();</script>",
            "endsWith requires one or two arguments",
        ),
        (
            "<script>'x'.slice(, 1);</script>",
            "String.slice start cannot be empty",
        ),
        (
            "<script>'x'.substring(, 1);</script>",
            "substring start cannot be empty",
        ),
        (
            "<script>'x'.split(, 1);</script>",
            "split separator cannot be empty expression",
        ),
        (
            "<script>'x'.replace('a');</script>",
            "replace requires exactly two arguments",
        ),
        (
            "<script>'x'.indexOf();</script>",
            "indexOf requires one or two arguments",
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
fn typed_array_constructors_and_properties_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const i8 = new Int8Array([257, -129, 1.9]);
            const u8c = new Uint8ClampedArray([300, -1, 1.5, 2.5, 0.5]);
            const bi = new BigInt64Array([1n, -1n]);
            document.getElementById('result').textContent =
              i8.length + ':' +
              i8[0] + ':' +
              i8[1] + ':' +
              i8[2] + ':' +
              u8c.join(',') + ':' +
              Int8Array.BYTES_PER_ELEMENT + ':' +
              i8.BYTES_PER_ELEMENT + ':' +
              typeof Int8Array + ':' +
              typeof TypedArray + ':' +
              bi[1];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:1:127:1:255,0,2,2,0:1:1:function:undefined:-1")?;
    Ok(())
}

#[test]
fn typed_array_static_from_of_and_constructor_errors_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = Int16Array.of(1, 2, 3);
            const b = Int16Array.from(a);
            document.getElementById('result').textContent = a.join(',') + ':' + b.join(',');
          });
        </script>
        "#;
    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,2,3:1,2,3")?;

    let err = Harness::from_html("<script>Int8Array(2);</script>")
        .expect_err("calling typed array constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("must be called with new")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>new BigInt64Array(new Int8Array([1]));</script>")
        .expect_err("mixing bigint and number typed arrays should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("cannot mix BigInt and Number typed arrays"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn typed_array_resizable_array_buffer_view_behavior_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const buffer = new ArrayBuffer(8, { maxByteLength: 16 });
          const tracking = new Float32Array(buffer);
          const fixed = new Float32Array(buffer, 0, 2);

          document.getElementById('btn').addEventListener('click', () => {
            let out =
              tracking.byteLength + ':' + tracking.length + ':' +
              fixed.byteLength + ':' + fixed.length;

            buffer.resize(12);
            out = out + ':' +
              tracking.byteLength + ':' + tracking.length + ':' +
              fixed.byteLength + ':' + fixed.length;

            buffer.resize(7);
            out = out + ':' +
              tracking.byteLength + ':' + tracking.length + ':' +
              fixed.byteLength + ':' + fixed.length + ':' + fixed[0];

            buffer.resize(8);
            out = out + ':' + fixed.byteLength + ':' + fixed.length + ':' + fixed[0];
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "8:2:8:2:12:3:8:2:4:1:0:0:undefined:8:2:0")?;
    Ok(())
}

#[test]
fn typed_array_methods_set_subarray_copy_within_and_with_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const ta = new Uint8Array([1, 2, 3, 4]);
            ta.copyWithin(1, 2);
            const sub = ta.subarray(1, 3);
            ta.set([9, 8], 2);
            const withOne = ta.with(0, 7);
            const rev = ta.toReversed();
            const src = new Uint8Array([3, 1, 2]);
            const sorted = src.toSorted();
            document.getElementById('result').textContent =
              ta.join(',') + ':' +
              sub.join(',') + ':' +
              withOne.join(',') + ':' +
              rev.join(',') + ':' +
              sorted.join(',') + ':' +
              ta.at(-1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,3,9,8:3,9:7,3,9,8:8,9,3,1:1,2,3:8")?;
    Ok(())
}

#[test]
fn typed_array_abstract_constructor_and_freeze_errors_work() {
    let err = Harness::from_html("<script>new (Object.getPrototypeOf(Int8Array))();</script>")
        .expect_err("abstract TypedArray constructor should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Abstract class TypedArray not directly constructable"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err =
        Harness::from_html("<script>const i8 = Int8Array.of(1,2,3); Object.freeze(i8);</script>")
            .expect_err("freezing non-empty typed array should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Cannot freeze array buffer views with elements"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn typed_array_alignment_and_array_buffer_constructor_errors_work() {
    let cases = [
        (
            "<script>new Int32Array(new ArrayBuffer(3));</script>",
            "byte length of Int32Array should be a multiple of 4",
        ),
        (
            "<script>new Int32Array(new ArrayBuffer(4), 1);</script>",
            "start offset of Int32Array should be a multiple of 4",
        ),
        (
            "<script>ArrayBuffer(8);</script>",
            "ArrayBuffer constructor must be called with new",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains(expected), "expected '{expected}', got '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn map_set_get_size_delete_and_iteration_order_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const map = new Map();
            map.set('a', 1);
            map.set('b', 2);
            map.set('c', 3);
            map.set('a', 97);

            const deleted = map.delete('b');
            const missing = map.delete('missing');

            let forEachOut = '';
            map.forEach((value, key, self) => {
              forEachOut = forEachOut + key + '=' + value + ':' + (self === map) + ';';
            });

            let forOfOut = '';
            for (const pair of map) {
              forOfOut = forOfOut + pair[0] + '=' + pair[1] + ';';
            }

            const entries = map.entries();
            const keys = map.keys();
            const values = map.values();

            document.getElementById('result').textContent =
              map.get('a') + ':' +
              map.size + ':' +
              deleted + ':' +
              missing + ':' +
              forEachOut + ':' +
              forOfOut + ':' +
              entries.length + ':' +
              keys.join(',') + ':' +
              values.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "97:2:true:false:a=97:true;c=3:true;:a=97;c=3;:2:a,c:97,3",
    )?;
    Ok(())
}

#[test]
fn map_same_value_zero_and_wrong_property_assignment_behavior_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const map = new Map();
            const keyArr = [];

            map.set(NaN, 'not-a-number');
            map.set(0, 'zero');
            map.set(-0, 'minus-zero');
            map.set(keyArr, 'arr');

            const wrongMap = new Map();
            wrongMap['bla'] = 'blaa';
            wrongMap['bla2'] = 'blaaa2';

            document.getElementById('result').textContent =
              map.get(Number('foo')) + ':' +
              map.get(0) + ':' +
              map.has(-0) + ':' +
              map.get([]) + ':' +
              map.get(keyArr) + ':' +
              wrongMap.has('bla') + ':' +
              wrongMap.size + ':' +
              wrongMap.bla;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "not-a-number:minus-zero:true:undefined:arr:false:0:blaa",
    )?;
    Ok(())
}

#[test]
fn map_group_by_and_get_or_insert_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const grouped = Map.groupBy([1, 2, 3, 4, 5], function(value) { return value % 2; });
            const odd = grouped.get(1);
            const even = grouped.get(0);
            const map = new Map();
            const first = map.getOrInsert('count', 1);
            const second = map.getOrInsert('count', 9);
            const computed1 = map.getOrInsertComputed('lazy', function(key) { return key + '-value'; });
            const computed2 = map.getOrInsertComputed('lazy', function() { return 'ignored'; });

            document.getElementById('result').textContent =
              odd.join(',') + ':' +
              even.join(',') + ':' +
              first + ':' +
              second + ':' +
              computed1 + ':' +
              computed2 + ':' +
              map.size;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,3,5:2,4:1:1:lazy-value:lazy-value:2")?;
    Ok(())
}

#[test]
fn map_constructor_clone_and_error_cases_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const original = new Map([[1, 'one'], [2, 'two']]);
            const clone = new Map(original);
            clone.set(1, 'uno');
            const cleared = new Map([[1, 'x']]);
            cleared.clear();
            document.getElementById('result').textContent =
              original.get(1) + ':' + clone.get(1) + ':' + (original === clone) + ':' +
              clone.size + ':' + cleared.size;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "one:uno:false:2:0")?;

    let err = Harness::from_html("<script>Map();</script>")
        .expect_err("calling Map constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Map constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const value = []; value.delete('x');</script>")
        .expect_err("Map methods on non-map value should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("is not a Map")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn weak_map_basic_methods_and_key_rules_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const wm = new WeakMap();
            const obj = {};
            const fnKey = function() {};
            const unique = Symbol('u');

            wm.set(obj, 'obj');
            wm.set(fnKey, 'fn');
            wm.set(unique, 'sym');

            const gotObj = wm.get(obj);
            const hasFn = wm.has(fnKey);
            const deleted = wm.delete(fnKey);
            const hasFnAfter = wm.has(fnKey);

            const primitiveGet = wm.get(1);
            const primitiveHas = wm.has('x');
            const primitiveDelete = wm.delete(null);

            const ctor = wm.constructor === WeakMap;
            const text = String(wm);
            const clearType = typeof wm.clear;

            let regThrows = false;
            try {
              wm.set(Symbol.for('shared'), 1);
            } catch (e) {
              regThrows = true;
            }

            document.getElementById('result').textContent =
              gotObj + ':' +
              hasFn + ':' +
              deleted + ':' +
              hasFnAfter + ':' +
              primitiveGet + ':' +
              primitiveHas + ':' +
              primitiveDelete + ':' +
              ctor + ':' +
              text + ':' +
              clearType + ':' +
              regThrows;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "obj:true:true:false:undefined:false:false:true:[object WeakMap]:undefined:true",
    )?;
    Ok(())
}

#[test]
fn weak_map_get_or_insert_and_constructor_clone_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const keyA = {};
            const keyB = {};
            const keyC = {};
            const lazyKey = {};

            const fromIterable = new WeakMap([[keyA, 'a'], [keyB, 'b']]);
            const fromClone = new WeakMap(fromIterable);

            const first = fromClone.getOrInsert(keyA, 'aa');
            const inserted = fromClone.getOrInsert(keyC, 3);
            const computed1 = fromClone.getOrInsertComputed(keyC, function() { return 9; });
            const computed2 = fromClone.getOrInsertComputed(keyB, function(k) {
              return k === keyB ? 'bb' : 'x';
            });

            let calls = 0;
            const lazy1 = fromClone.getOrInsertComputed(lazyKey, function(k) {
              calls = calls + 1;
              return k === lazyKey ? 'lazy' : 'bad';
            });
            const lazy2 = fromClone.getOrInsertComputed(lazyKey, function() {
              calls = calls + 1;
              return 'ignored';
            });

            let invalidThrows = false;
            try {
              fromClone.getOrInsert(1, 'x');
            } catch (e) {
              invalidThrows = true;
            }

            document.getElementById('result').textContent =
              fromClone.get(keyA) + ':' +
              fromClone.get(keyB) + ':' +
              first + ':' +
              inserted + ':' +
              computed1 + ':' +
              computed2 + ':' +
              lazy1 + ':' +
              lazy2 + ':' +
              calls + ':' +
              invalidThrows + ':' +
              (fromClone === fromIterable);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:b:a:3:3:b:lazy:lazy:1:true:false")?;
    Ok(())
}

#[test]
fn weak_map_constructor_and_invalid_calls_error() {
    let err = Harness::from_html("<script>WeakMap();</script>")
        .expect_err("calling WeakMap constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("WeakMap constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>new WeakMap([[1, 'x']]);</script>")
        .expect_err("WeakMap keys must be objects or non-registered symbols");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Invalid value used as weak map key"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const wm = new WeakMap(); wm.clear();</script>")
        .expect_err("WeakMap.clear should not exist");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("WeakMap.clear is not a function")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn weak_set_basic_methods_and_key_rules_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const ws = new WeakSet();
            const obj = {};
            const fnKey = function() {};
            const unique = Symbol('u');
            const chainObj = {};

            const addReturn = ws.add(obj) === ws;
            ws.add(fnKey);
            ws.add(unique);
            ws.add(obj);

            const chainHas = (new WeakSet()).add(chainObj).has(chainObj);

            const hasObj = ws.has(obj);
            const hasFn = ws.has(fnKey);
            const deleted = ws.delete(fnKey);
            const hasFnAfter = ws.has(fnKey);

            const primitiveHas = ws.has(1);
            const primitiveDelete = ws.delete('x');

            const ctor = ws.constructor === WeakSet;
            const text = String(ws);
            const clearType = typeof ws.clear;

            let regThrows = false;
            try {
              ws.add(Symbol.for('shared'));
            } catch (e) {
              regThrows = true;
            }

            document.getElementById('result').textContent =
              addReturn + ':' +
              chainHas + ':' +
              hasObj + ':' +
              hasFn + ':' +
              deleted + ':' +
              hasFnAfter + ':' +
              primitiveHas + ':' +
              primitiveDelete + ':' +
              ctor + ':' +
              text + ':' +
              clearType + ':' +
              regThrows;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:false:false:false:true:[object WeakSet]:undefined:true",
    )?;
    Ok(())
}

#[test]
fn weak_set_constructor_clone_and_error_cases_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = {};
            const b = {};
            const c = {};
            const fromIterable = new WeakSet([a, b, a]);
            const clone = new WeakSet(fromIterable);
            const addRet = clone.add(c) === clone;

            let invalidThrows = false;
            try {
              clone.add(1);
            } catch (e) {
              invalidThrows = true;
            }

            document.getElementById('result').textContent =
              clone.has(a) + ':' +
              clone.has(b) + ':' +
              clone.has(c) + ':' +
              addRet + ':' +
              invalidThrows + ':' +
              (clone === fromIterable);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true:true:false")?;
    Ok(())
}

#[test]
fn weak_set_constructor_and_invalid_calls_error() {
    let err = Harness::from_html("<script>WeakSet();</script>")
        .expect_err("calling WeakSet constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("WeakSet constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>new WeakSet([1]);</script>")
        .expect_err("WeakSet values must be objects or non-registered symbols");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Invalid value used in weak set"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err =
        Harness::from_html("<script>const ws = new WeakSet(); ws.union(new Set([1]));</script>")
            .expect_err("WeakSet.union should not exist");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("WeakSet.union is not a function")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const ws = new WeakSet(); ws.get({});</script>")
        .expect_err("WeakSet.get should not exist");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("WeakSet.get is not a function")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn set_basic_methods_and_iteration_order_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const set = new Set();
            set.add(1);
            set.add(2);
            set.add(2);
            set.add(NaN);
            set.add(Number('foo'));
            set.delete(2);
            set.add(2);

            let order = '';
            for (const item of set) {
              order = order + item + ',';
            }

            const keys = set.keys();
            const values = set.values();
            const entries = set.entries();

            let forEachOut = '';
            set.forEach((value, key, self) => {
              forEachOut = forEachOut + value + '=' + key + ':' + (self === set) + ';';
            });

            document.getElementById('result').textContent =
              set.size + ':' +
              set.has(NaN) + ':' +
              set.has(2) + ':' +
              order + ':' +
              keys.join('|') + ':' +
              values.join('|') + ':' +
              entries.length + ':' +
              forEachOut;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "3:true:true:1,NaN,2,:1|NaN|2:1|NaN|2:3:1=1:true;NaN=NaN:true;2=2:true;",
    )?;
    Ok(())
}

#[test]
fn set_composition_methods_and_map_set_like_argument_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = new Set([1, 2, 3, 4]);
            const b = new Set([3, 4, 5]);
            const m = new Map([[2, 'two'], [7, 'seven']]);

            const union = a.union(b);
            const intersection = a.intersection(b);
            const difference = a.difference(b);
            const symmetric = a.symmetricDifference(b);
            const unionMap = a.union(m);

            const disjoint = a.isDisjointFrom(new Set([8, 9]));
            const subsetSet = new Set([1, 2]);
            const supersetSet = new Set([1, 4]);
            const subsetMapSet = new Set([2]);
            const subset = subsetSet.isSubsetOf(a);
            const superset = a.isSupersetOf(supersetSet);
            const subsetMap = subsetMapSet.isSubsetOf(m);

            const unionValues = union.values();
            const intersectionValues = intersection.values();
            const differenceValues = difference.values();
            const symmetricValues = symmetric.values();
            const unionMapValues = unionMap.values();

            document.getElementById('result').textContent =
              unionValues.join(',') + ':' +
              intersectionValues.join(',') + ':' +
              differenceValues.join(',') + ':' +
              symmetricValues.join(',') + ':' +
              unionMapValues.join(',') + ':' +
              disjoint + ':' + subset + ':' + superset + ':' + subsetMap;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "1,2,3,4,5:3,4:1,2:1,2,5:1,2,3,4,7:true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn set_constructor_iterable_and_property_assignment_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const map = new Map([[1, 'one'], [2, 'two']]);
            const fromArr = new Set([1, 1, 2, 3]);
            const fromMap = new Set(map);
            const fromMapValues = fromMap.values();

            const wrongSet = new Set();
            wrongSet['bla'] = 'x';

            const obj = {};
            fromArr.add(obj);
            fromArr.add({});

            document.getElementById('result').textContent =
              fromArr.size + ':' +
              fromArr.has(1) + ':' +
              fromArr.has(4) + ':' +
              fromMap.size + ':' +
              fromMapValues.join('|') + ':' +
              wrongSet.has('bla') + ':' +
              wrongSet.size + ':' +
              wrongSet.bla;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5:true:false:2:1,one|2,two:false:0:x")?;
    Ok(())
}

#[test]
fn set_constructor_and_composition_errors_work() {
    let err = Harness::from_html("<script>Set();</script>")
        .expect_err("calling Set constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Set constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const set = new Set([1]); set.union([1,2]);</script>")
        .expect_err("Set.union requires a set-like argument");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("set-like")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const arr = []; arr.union(new Set([1]));</script>")
        .expect_err("Set method target must be a Set");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("is not a Set")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn url_search_params_basic_methods_and_iteration_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const params = new URLSearchParams("q=URLUtils.searchParams&topic=api");

            let forOfOut = '';
            for (const pair of params) {
              forOfOut = forOfOut + pair[0] + '=' + pair[1] + ';';
            }

            const entries = params.entries();
            let entriesOut = '';
            for (const pair of entries) {
              entriesOut = entriesOut + pair[0] + '=' + pair[1] + ';';
            }

            const hasTopic = params.has('topic');
            const hasTopicFish = params.has('topic', 'fish');
            const topic = params.get('topic');
            const allTopic = params.getAll('topic');
            const missingIsNull = params.get('foo') === null;

            params.append('topic', 'webdev');
            const afterAppend = params.toString();

            params.set('topic', 'More webdev');
            const afterSet = params.toString();

            params.append('topic', 'fish');
            const hasFish = params.has('topic', 'fish');
            params.delete('topic', 'fish');
            const afterDeletePair = params.toString();

            params.delete('topic');
            const afterDelete = params.toString();

            document.getElementById('result').textContent =
              forOfOut + '|' +
              entriesOut + '|' +
              hasTopic + ':' + hasTopicFish + ':' + topic + ':' + allTopic.join(',') + ':' + missingIsNull + '|' +
              afterAppend + '|' +
              afterSet + '|' +
              hasFish + '|' +
              afterDeletePair + '|' +
              afterDelete + '|' +
              params.size;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "q=URLUtils.searchParams;topic=api;|q=URLUtils.searchParams;topic=api;|true:false:api:api:true|q=URLUtils.searchParams&topic=api&topic=webdev|q=URLUtils.searchParams&topic=More+webdev|true|q=URLUtils.searchParams&topic=More+webdev|q=URLUtils.searchParams|1",
    )?;
    Ok(())
}

#[test]
fn url_search_params_object_and_location_parsing_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const paramsObj = { foo: 'bar', baz: 'bar' };
            const fromObj = new URLSearchParams(paramsObj);

            location.href = 'https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams?foo=a';
            const fromLocation = new URLSearchParams(window.location.search);

            const full = new URLSearchParams('http://example.com/search?query=%40');
            const leading = new URLSearchParams('?query=value');
            const dup = new URLSearchParams('foo=bar&foo=baz');
            const dupAll = dup.getAll('foo');
            const emptyVal = new URLSearchParams('foo=&bar=baz');
            const noEquals = new URLSearchParams('foo&bar=baz');

            document.getElementById('result').textContent =
              fromObj.toString() + ':' +
              fromObj.has('foo') + ':' +
              fromObj.get('foo') + ':' +
              fromLocation.get('foo') + ':' +
              full.has('query') + ':' +
              full.has('http://example.com/search?query') + ':' +
              full.get('query') + ':' +
              full.get('http://example.com/search?query') + ':' +
              leading.has('query') + ':' +
              dup.get('foo') + ':' +
              dupAll.join(',') + ':' +
              emptyVal.get('foo') + ':' +
              noEquals.get('foo') + ':' +
              noEquals.toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "foo=bar&baz=bar:true:bar:a:false:true:null:@:true:bar:bar,baz:::foo=&bar=baz",
    )?;
    Ok(())
}

#[test]
fn url_search_params_percent_encoding_and_plus_behavior_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const params = new URLSearchParams('%24%25%26=%28%29%2B');
            const decoded = params.get('$%&');
            const encodedKeyMiss = params.get('%24%25%26') === null;
            params.append('$%&$#@+', '$#&*@#()+');
            const encoded = params.toString();

            const plusBrokenParams = new URLSearchParams('bin=E+AXQB+A');
            const plusBroken = plusBrokenParams.get('bin');
            const plusPreservedParams = new URLSearchParams();
            plusPreservedParams.append('bin', 'E+AXQB+A');
            const plusPreserved = plusPreservedParams.get('bin');
            const plusSerialized = plusPreservedParams.toString();

            const encodedKeyParams = new URLSearchParams();
            encodedKeyParams.append('%24%26', 'value');

            document.getElementById('result').textContent =
              decoded + ':' +
              encodedKeyMiss + ':' +
              encoded + ':' +
              plusBroken + ':' +
              plusPreserved + ':' +
              plusSerialized + ':' +
              encodedKeyParams.toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "()+:true:%24%25%26=%28%29%2B&%24%25%26%24%23%40%2B=%24%23%26*%40%23%28%29%2B:E AXQB A:E+AXQB+A:bin=E%2BAXQB%2BA:%2524%2526=value",
    )?;
    Ok(())
}

#[test]
fn url_search_params_constructor_requires_new() {
    let err = Harness::from_html("<script>URLSearchParams('a=1');</script>")
        .expect_err("calling URLSearchParams constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("URLSearchParams constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn url_constructor_properties_setters_and_methods_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://some.site/?id=123';

            const url = new URL('../cats', 'http://www.example.com/dogs');
            const initial =
              url.href + '|' +
              url.hostname + '|' +
              url.pathname + '|' +
              url.origin;

            url['hash'] = 'tabby';
            url['username'] = 'alice';
            url['password'] = 'secret';
            url['port'] = '8080';
            url['protocol'] = 'https:';
            url['pathname'] = 'démonstration.html';
            url['search'] = 'q=space value';

            const parsedLocation = new URL(window.location.href);
            const fromLocation = parsedLocation.searchParams.get('id');

            document.getElementById('result').textContent =
              initial + '|' +
              url.href + '|' +
              url.hash + '|' +
              url.search + '|' +
              url.searchParams.get('q') + '|' +
              url.toString() + '|' +
              url.toJSON() + '|' +
              (window.URL === URL) + '|' +
              (typeof URL) + '|' +
              fromLocation;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://www.example.com/cats|www.example.com|/cats|http://www.example.com|https://alice:secret@www.example.com:8080/d%C3%A9monstration.html?q=space%20value#tabby|#tabby|?q=space%20value|space value|https://alice:secret@www.example.com:8080/d%C3%A9monstration.html?q=space%20value#tabby|https://alice:secret@www.example.com:8080/d%C3%A9monstration.html?q=space%20value#tabby|true|function|123",
    )?;
    Ok(())
}

#[test]
fn url_protocol_switch_and_opaque_setter_noop_matrix_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const httpToFile = new URL('http://example.com/a');
            httpToFile.protocol = 'file:';

            const portBlocksFile = new URL('http://example.com:81/a');
            portBlocksFile.protocol = 'file:';

            const credentialBlocksFile = new URL('http://u:p@example.com/a');
            credentialBlocksFile.protocol = 'file:';

            const fileToHttp = new URL('file://server/share/file.txt');
            fileToHttp.protocol = 'http:';

            const localFile = new URL('file:///Users/me/test.txt');
            localFile.protocol = 'http:';

            const opaque = new URL('foo:abc?x=1#h');
            opaque.pathname = 'new/path';
            opaque.host = 'ignored.test:1234';
            opaque.hostname = 'ignored-2.test';
            opaque.port = '5678';
            opaque.search = 'z=2';
            opaque.hash = 'k';

            document.getElementById('result').textContent = [
              httpToFile.href,
              portBlocksFile.href,
              credentialBlocksFile.href,
              fileToHttp.href,
              localFile.href,
              [opaque.href, opaque.pathname, opaque.host, opaque.search, opaque.hash].join(',')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "file://example.com/a|http://example.com:81/a|http://u:p@example.com/a|http://server/share/file.txt|file:///Users/me/test.txt|foo:abc?z=2#k,abc,,?z=2,#k",
    )?;
    Ok(())
}

#[test]
fn url_file_host_setter_matrix_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const server = new URL('file://server/share/file.txt');
            const localhost = new URL('file://localhost/Users/me/test.txt');

            const initial = [
              [server.href, server.host, server.hostname, server.port].join(','),
              [localhost.href, localhost.host, localhost.hostname, localhost.port].join(',')
            ].join(';');

            server.port = '8080';
            localhost.port = '8080';
            const afterPort = [
              [server.href, server.host, server.hostname, server.port].join(','),
              [localhost.href, localhost.host, localhost.hostname, localhost.port].join(',')
            ].join(';');

            server.host = 'example.com';
            localhost.hostname = 'example.com';
            const afterHost = [
              [server.href, server.host, server.hostname, server.port].join(','),
              [localhost.href, localhost.host, localhost.hostname, localhost.port].join(',')
            ].join(';');

            server.host = 'localhost';
            localhost.host = 'localhost';
            const afterLocalhost = [
              [server.href, server.host, server.hostname, server.port].join(','),
              [localhost.href, localhost.host, localhost.hostname, localhost.port].join(',')
            ].join(';');

            server.host = 'localhost:8080';
            localhost.host = 'example.com:8080';
            const blockedPort = [
              [server.href, server.host, server.hostname, server.port].join(','),
              [localhost.href, localhost.host, localhost.hostname, localhost.port].join(',')
            ].join(';');

            document.getElementById('result').textContent =
              [initial, afterPort, afterHost, afterLocalhost, blockedPort].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "file://server/share/file.txt,server,server,;file:///Users/me/test.txt,,,|file://server/share/file.txt,server,server,;file:///Users/me/test.txt,,,|file://example.com/share/file.txt,example.com,example.com,;file://example.com/Users/me/test.txt,example.com,example.com,|file:///share/file.txt,,,;file:///Users/me/test.txt,,,|file:///share/file.txt,,,;file:///Users/me/test.txt,,,",
    )?;
    Ok(())
}

#[test]
fn url_file_idna_host_and_method_extra_args_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ctor = [
              new URL('file://\u00E9xample.com/path'),
              new URL('file://%C3%A9xample.com/path'),
              new URL('file://example\u3002com./path'),
              new URL('file://\u05D0.com/path'),
              new URL('file://localhost/path')
            ].map((url) => [url.href, url.host, url.hostname, url.origin].join(',')).join(';');

            const invalid = [
              'file://xn--/path',
              'file://a\u200Db.com/path'
            ].map((value) => {
              try {
                new URL(value);
                return 'false';
              } catch (err) {
                return String(err).includes('Invalid URL');
              }
            }).join(',');

            const setter = (() => {
              const url = new URL('file://server/share/file.txt');
              url.host = '\u00E9xample.com';
              const a = [url.href, url.host, url.hostname].join(',');
              url.hostname = 'example\u3002com.';
              const b = [url.href, url.host, url.hostname].join(',');
              url.host = '\u05D0.com';
              const c = [url.href, url.host, url.hostname].join(',');
              url.hostname = 'xn--';
              const d = [url.href, url.host, url.hostname].join(',');
              url.host = 'a\u200Db.com';
              const e = [url.href, url.host, url.hostname].join(',');
              url.host = 'localhost';
              const f = [url.href, url.host, url.hostname].join(',');
              return [a, b, c, d, e, f].join(';');
            })();

            const url = new URL('https://example.com/?b=2&a=1&a=3');
            let side = 'start';
            const urlMethods = [
              url.toString(side = 'url.toString'),
              side,
              url.toJSON(side = 'url.toJSON'),
              side
            ].join(',');

            const params = url.searchParams;
            const entries = [
              Array.from(params.entries(side = 'params.entries'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');
            const keys = [
              Array.from(params.keys(side = 'params.keys')).join(';'),
              side
            ].join(',');
            const values = [
              Array.from(params.values(side = 'params.values')).join(';'),
              side
            ].join(',');
            const stringified = [
              params.toString(side = 'params.toString'),
              side
            ].join(',');

            document.getElementById('result').textContent = [
              ctor,
              invalid,
              setter,
              urlMethods,
              entries,
              keys,
              values,
              stringified
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "file://xn--xample-9ua.com/path,xn--xample-9ua.com,xn--xample-9ua.com,null;file://xn--xample-9ua.com/path,xn--xample-9ua.com,xn--xample-9ua.com,null;file://example.com./path,example.com.,example.com.,null;file://xn--4db.com/path,xn--4db.com,xn--4db.com,null;file:///path,,,null|true,true|file://xn--xample-9ua.com/share/file.txt,xn--xample-9ua.com,xn--xample-9ua.com;file://example.com./share/file.txt,example.com.,example.com.;file://xn--4db.com/share/file.txt,xn--4db.com,xn--4db.com;file://xn--4db.com/share/file.txt,xn--4db.com,xn--4db.com;file://xn--4db.com/share/file.txt,xn--4db.com,xn--4db.com;file:///share/file.txt,,|https://example.com/?b=2&a=1&a=3,url.toString,https://example.com/?b=2&a=1&a=3,url.toJSON|b:2;a:1;a:3,params.entries|b;a;a,params.keys|2;1;3,params.values|b=2&a=1&a=3,params.toString",
    )?;
    Ok(())
}

#[test]
fn url_search_params_live_sync_with_url_search_and_href_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const url = new URL('https://example.com/?a=b ~');
            const before = url.href;

            const params = url.searchParams;
            const appended = params.append('topic', 'web dev');
            const afterAppend = url.href;

            url['search'] = '?x=1';
            const afterSearch = url.href;

            const topicCleared = url.searchParams.get('topic') === null;
            const xValue = url.searchParams.get('x');

            document.getElementById('result').textContent =
              before + '|' +
              afterAppend + '|' +
              afterSearch + '|' +
              topicCleared + ':' + xValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/?a=b%20~|https://example.com/?a=b+%7E&topic=web+dev|https://example.com/?x=1|true:1",
    )?;
    Ok(())
}

#[test]
fn url_search_params_malformed_percent_and_host_code_point_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const replacement = String.fromCharCode(0xFFFD);

            const params = new URLSearchParams('?%zz=1&a=%zz&b=%E0%A4&c=%C3%28&d=%&e=%2');
            const paramsState = [
              params.get('%zz'),
              params.get('a'),
              params.get('b'),
              params.get('c'),
              params.get('d'),
              params.get('e'),
              params.toString()
            ].join(',');

            const url = new URL('https://\uFF21example.com/?a=%zz&b=%E0%A4&c=%C3%28');
            const fromCtor = [
              url.href,
              url.search,
              url.searchParams.get('a'),
              url.searchParams.get('b'),
              url.searchParams.get('c'),
              url.searchParams.toString()
            ].join(',');

            url.search = '?a=%zz&b=%E0%A4&c=%C3%28';
            const afterSearch = [
              url.href,
              url.search,
              url.searchParams.get('a'),
              url.searchParams.get('b'),
              url.searchParams.get('c'),
              url.searchParams.toString()
            ].join(',');

            url.searchParams.append('%zz', 'x y');
            const afterAppend = [
              url.href,
              url.search,
              url.searchParams.get('%zz'),
              url.searchParams.toString()
            ].join(',');

            const hostCtor = (() => {
              const rawFullwidth = new URL('https://\uFF21example.com/root');
              const percentFullwidth = new URL('https://%EF%BC%A1example.com/root');
              const invalid = [
                'https://\u00E9xample.com/',
                'https://%C3%A9xample.com/',
                'https://%00example.com/'
              ].map((value) => {
                const canParse = URL.canParse(value);
                const parsed = URL.parse(value) === null;
                const constructed = (() => {
                  try {
                    new URL(value);
                    return 'false';
                  } catch (err) {
                    return String(err).includes('Invalid URL');
                  }
                })();
                return [canParse, parsed, constructed].join(':');
              }).join(';');

              const setter = (() => {
                const host = new URL('https://base.test/root');
                host.hostname = '\uFF21example.com';
                const afterFullwidth = [host.href, host.host].join(',');
                host.hostname = '\u00E9xample.com';
                const afterUnicode = [host.href, host.host].join(',');
                host.hostname = '%00example.com';
                const afterControl = [host.href, host.host].join(',');
                return [afterFullwidth, afterUnicode, afterControl].join(';');
              })();

              return [
                [rawFullwidth.href, rawFullwidth.host].join(','),
                [percentFullwidth.href, percentFullwidth.host].join(','),
                invalid,
                setter
              ].join('|');
            })();

            document.getElementById('result').textContent = [
              paramsState,
              fromCtor,
              afterSearch,
              afterAppend,
              hostCtor,
              replacement
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1,%zz,\u{FFFD},\u{FFFD}(,%,%2,%25zz=1&a=%25zz&b=%EF%BF%BD&c=%EF%BF%BD%28&d=%25&e=%252|https://aexample.com/?a=%zz&b=%E0%A4&c=%C3%28,?a=%zz&b=%E0%A4&c=%C3%28,%zz,\u{FFFD},\u{FFFD}(,a=%25zz&b=%EF%BF%BD&c=%EF%BF%BD%28|https://aexample.com/?a=%zz&b=%E0%A4&c=%C3%28,?a=%zz&b=%E0%A4&c=%C3%28,%zz,\u{FFFD},\u{FFFD}(,a=%25zz&b=%EF%BF%BD&c=%EF%BF%BD%28|https://aexample.com/?a=%25zz&b=%EF%BF%BD&c=%EF%BF%BD%28&%25zz=x+y,?a=%25zz&b=%EF%BF%BD&c=%EF%BF%BD%28&%25zz=x+y,x y,a=%25zz&b=%EF%BF%BD&c=%EF%BF%BD%28&%25zz=x+y|https://aexample.com/root,aexample.com|https://aexample.com/root,aexample.com|true:false:false;true:false:false;false:true:true|https://aexample.com/root,aexample.com;https://xn--xample-9ua.com/root,xn--xample-9ua.com;https://xn--xample-9ua.com/root,xn--xample-9ua.com|\u{FFFD}",
    )?;
    Ok(())
}

#[test]
fn url_idna_host_and_search_params_duplicate_live_mutation_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const replacement = String.fromCharCode(0xFFFD);

            const idna = [
              new URL('https://\u00E9xample.com/root'),
              new URL('https://e\u0301xample.com/root'),
              new URL('https://%C3%A9xample.com/root'),
              new URL('https://example\u3002com/root'),
              new URL('https://%E3%80%82example.com/root')
            ].map((url) => [url.href, url.host].join(',')).join(';');

            const setter = (() => {
              const url = new URL('https://base.test/root');
              url.hostname = '\u00E9xample.com';
              const afterUnicode = [url.href, url.host].join(',');
              url.hostname = 'example\u3002com';
              const afterDot = [url.href, url.host].join(',');
              url.hostname = '%00example.com';
              const afterInvalid = [url.href, url.host].join(',');
              return [afterUnicode, afterDot, afterInvalid].join(';');
            })();

            const url = new URL('https://\u00E9xample.com/?b=%E0%A4&a=%zz&a=1');
            const params = url.searchParams;
            const ctor = [
              url.href,
              url.host,
              params.get('a'),
              params.get('b'),
              params.toString()
            ].join(',');

            params.sort();
            const afterSort = [
              url.href,
              params.get('a'),
              params.get('b'),
              params.toString()
            ].join(',');

            params.set('a', '%zz');
            const afterSet = [
              url.href,
              params.get('a'),
              params.get('b'),
              params.toString()
            ].join(',');

            url.search = '?m=%zz&m=%E0%A4&n=1';
            const afterSearch = [
              url.href,
              params.get('m'),
              params.get('n'),
              params.toString()
            ].join(',');

            params.delete('m', replacement);
            const afterDelete = [
              url.href,
              params.get('m'),
              params.get('n'),
              params.toString()
            ].join(',');

            url.href = 'https://example\u3002com/?q=%zz';
            params.append('r', '1 2');
            const afterHref = [
              url.href,
              url.host,
              params.get('q'),
              params.get('r'),
              params.toString()
            ].join(',');

            document.getElementById('result').textContent = [
              idna,
              setter,
              ctor,
              afterSort,
              afterSet,
              afterSearch,
              afterDelete,
              afterHref
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://xn--xample-9ua.com/root,xn--xample-9ua.com;https://xn--xample-9ua.com/root,xn--xample-9ua.com;https://xn--xample-9ua.com/root,xn--xample-9ua.com;https://example.com/root,example.com;https://.example.com/root,.example.com|https://xn--xample-9ua.com/root,xn--xample-9ua.com;https://example.com/root,example.com;https://example.com/root,example.com|https://xn--xample-9ua.com/?b=%E0%A4&a=%zz&a=1,xn--xample-9ua.com,%zz,\u{FFFD},b=%EF%BF%BD&a=%25zz&a=1|https://xn--xample-9ua.com/?a=%25zz&a=1&b=%EF%BF%BD,%zz,\u{FFFD},a=%25zz&a=1&b=%EF%BF%BD|https://xn--xample-9ua.com/?a=%25zz&b=%EF%BF%BD,%zz,\u{FFFD},a=%25zz&b=%EF%BF%BD|https://xn--xample-9ua.com/?m=%zz&m=%E0%A4&n=1,%zz,1,m=%25zz&m=%EF%BF%BD&n=1|https://xn--xample-9ua.com/?m=%25zz&n=1,%zz,1,m=%25zz&n=1|https://example.com/?q=%25zz&r=1+2,example.com,%zz,1 2,q=%25zz&r=1+2",
    )?;
    Ok(())
}

#[test]
fn url_idna_invalid_labels_and_overlap_dispatch_extra_args_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const invalid = [
              'https://xn--/',
              'https://xn--a/',
              'https://xn--a-.com/',
              'https://\u200C.com/',
              'https://a\u200Db.com/'
            ].map((value) => {
              try {
                new URL(value);
                return 'false';
              } catch (err) {
                return String(err).includes('Invalid URL');
              }
            }).join(',');

            const valid = [
              new URL('https://-a.com/'),
              new URL('https://a-.com/'),
              new URL('https://example.com./'),
              new URL('https://example\u3002com./'),
              new URL('https://\u05D0.com/'),
              new URL('https://a\u3002b\uFF0Ec\uFF61d/')
            ].map((url) => [url.href, url.host].join(',')).join(';');

            const setter = (() => {
              const url = new URL('https://base.test/root');
              url.hostname = 'example.com.';
              const trailingDot = [url.href, url.host].join(',');
              url.hostname = 'example\u3002com.';
              const dotVariant = [url.href, url.host].join(',');
              url.hostname = '\u05D0.com';
              const bidi = [url.href, url.host].join(',');
              url.hostname = 'xn--';
              const afterInvalidLabel = [url.href, url.host].join(',');
              url.hostname = 'a\u200Db.com';
              const afterJoiner = [url.href, url.host].join(',');
              return [trailingDot, dotVariant, bidi, afterInvalidLabel, afterJoiner].join(';');
            })();

            const url = new URL('https://example.com/?b=2&a=1');
            let side = 'start';
            url.searchParams.sort(side = 'array-sort');
            const afterArraySort = [url.href, side].join(',');

            const params = new URLSearchParams('a=1&a=2');
            const has = params.has('a', '2', side = side + '/has');
            params.delete('a', '2', side = side + '/delete');
            params.set('b', '3', side = side + '/set');
            params.append('c', '4', side = side + '/append');
            params.sort(side = side + '/sort');
            const afterParams = [has, params.toString(), side].join(',');

            const key = {};
            const map = new Map([['a', 1]]);
            const set = new Set(['x']);
            const weakMap = new WeakMap([[key, 'v']]);
            const weakSet = new WeakSet([key]);
            side = 'map-start';
            const overlap = [
              map.has('a', side = 'map-has'),
              map.delete('a', side = side + '/map-delete'),
              set.has('x', side = side + '/set-has'),
              set.delete('x', side = side + '/set-delete'),
              weakMap.has(key, side = side + '/weakmap-has'),
              weakMap.delete(key, side = side + '/weakmap-delete'),
              weakSet.has(key, side = side + '/weakset-has'),
              weakSet.delete(key, side = side + '/weakset-delete'),
              side
            ].join(',');

            document.getElementById('result').textContent = [
              invalid,
              valid,
              setter,
              afterArraySort,
              afterParams,
              overlap
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true,true,true,true,true|https://-a.com/,-a.com;https://a-.com/,a-.com;https://example.com./,example.com.;https://example.com./,example.com.;https://xn--4db.com/,xn--4db.com;https://a.b.c.d/,a.b.c.d|https://example.com./root,example.com.;https://example.com./root,example.com.;https://xn--4db.com/root,xn--4db.com;https://xn--4db.com/root,xn--4db.com;https://xn--4db.com/root,xn--4db.com|https://example.com/?a=1&b=2,array-sort|true,a=1&b=3&c=4,array-sort/has/delete/set/append/sort|true,true,true,true,true,true,true,true,map-has/map-delete/set-has/set-delete/weakmap-has/weakmap-delete/weakset-has/weakset-delete",
    )?;
    Ok(())
}

#[test]
fn collection_member_chain_and_extra_arg_parity_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const probe = (label, fn) => {
              try {
                return fn();
              } catch (error) {
                return `ERR:${label}:${error.message}`;
              }
            };
            let side = 'start';

            const directMap = new Map([['a', 1]]);
            const directMapEntries = [
              Array.from(directMap.entries(side = 'direct.map.entries'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');
            side = 'start';
            const directMapSet = [
              directMap.set('b', 2, side = 'direct.map.set') === directMap,
              side,
              directMap.get('b')
            ].join(',');
            side = 'start';
            const directMapClearTarget = new Map([['c', 3]]);
            const directMapClear = [
              String(directMapClearTarget.clear(side = 'direct.map.clear')),
              side,
              directMapClearTarget.size
            ].join(',');

            side = 'start';
            const directSet = new Set(['x']);
            const directSetEntries = [
              Array.from(directSet.entries(side = 'direct.set.entries'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');
            side = 'start';
            const directSetAdd = [
              directSet.add('y', side = 'direct.set.add') === directSet,
              side,
              Array.from(directSet.values()).join(';')
            ].join(',');

            const weakKey = {};
            side = 'start';
            const directWeakMap = new WeakMap([[weakKey, 'v']]);
            const directWeakMapSet = [
              directWeakMap.set(weakKey, 'w', side = 'direct.weakMap.set') === directWeakMap,
              side,
              directWeakMap.get(weakKey)
            ].join(',');
            side = 'start';
            const directWeakSet = new WeakSet();
            const directWeakSetAdd = [
              directWeakSet.add(weakKey, side = 'direct.weakSet.add') === directWeakSet,
              side,
              directWeakSet.has(weakKey)
            ].join(',');

            const holder = {
              nested: { map: new Map([['m', 4]]) },
              setHolder: { nested: { set: new Set(['n']) } },
              params: new URL('https://example.com/?q=1&q=2&r=3').searchParams
            };

            side = 'start';
            const chainMapEntries = [
              Array.from(holder.nested.map.entries(side = 'chain.map.entries'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');
            side = 'start';
            const chainMapClear = [
              String(holder.nested.map.clear(side = 'chain.map.clear')),
              side,
              holder.nested.map.size
            ].join(',');

            side = 'start';
            const chainSetEntries = [
              Array.from(holder.setHolder.nested.set.entries(side = 'chain.set.entries'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');
            side = 'start';
            const chainSetClear = [
              String(holder.setHolder.nested.set.clear(side = 'chain.set.clear')),
              side,
              holder.setHolder.nested.set.size
            ].join(',');

            side = 'start';
            const chainParamsEntries = [
              Array.from(holder.params.entries(side = 'chain.params.entries'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');
            side = 'start';
            const chainParamsKeys = [
              Array.from(holder.params.keys(side = 'chain.params.keys')).join(';'),
              side
            ].join(',');
            side = 'start';
            const chainParamsValues = [
              Array.from(holder.params.values(side = 'chain.params.values')).join(';'),
              side
            ].join(',');

            document.getElementById('result').textContent = [
              directMapEntries,
              directMapSet,
              directMapClear,
              directSetEntries,
              directSetAdd,
              directWeakMapSet,
              directWeakSetAdd,
              chainMapEntries,
              chainMapClear,
              chainSetEntries,
              chainSetClear,
              chainParamsEntries,
              chainParamsKeys,
              chainParamsValues
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "a:1,direct.map.entries|true,direct.map.set,2|undefined,direct.map.clear,0|x:x,direct.set.entries|true,direct.set.add,x;y|true,direct.weakMap.set,w|true,direct.weakSet.add,true|m:4,chain.map.entries|undefined,chain.map.clear,0|n:n,chain.set.entries|undefined,chain.set.clear,0|q:1;q:2;r:3,chain.params.entries|q;q;r,chain.params.keys|1;2;3,chain.params.values",
    )?;
    Ok(())
}

#[test]
fn collection_extracted_method_call_and_prototype_parity_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let side = 'start';

            const map = new Map([['a', 1]]);
            const mapGet = [
              map.get.call(map, 'a', side = 'map.get.call'),
              side
            ].join(',');
            side = 'start';
            const mapProtoEntries = [
              Array.from(map.constructor.prototype.entries.call(map, side = 'map.proto.entries'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');

            side = 'start';
            const set = new Set(['x']);
            const setProtoAdd = [
              set.constructor.prototype.add.call(set, 'y', side = 'set.proto.add') === set,
              side,
              Array.from(set.values()).join(';')
            ].join(',');

            const weakKey = {};
            side = 'start';
            const weakMap = new WeakMap([[weakKey, 'v']]);
            const weakMapProtoGet = [
              weakMap.constructor.prototype.get.call(weakMap, weakKey, side = 'weakMap.proto.get'),
              side
            ].join(',');

            side = 'start';
            const weakSet = new WeakSet([weakKey]);
            const weakSetProtoHas = [
              weakSet.constructor.prototype.has.call(weakSet, weakKey, side = 'weakSet.proto.has'),
              side
            ].join(',');

            const url = new URL('https://example.com/path?b=2&a=1&a=3#hash');
            side = 'start';
            const urlToStringFn = url.constructor.prototype.toString;
            const urlToString = [
              urlToStringFn.call(url, side = 'url.toString.call'),
              side
            ].join(',');
            side = 'start';
            const urlProtoToJson = [
              url.constructor.prototype.toJSON.call(url, side = 'url.proto.toJSON'),
              side
            ].join(',');

            const params = url.searchParams;
            side = 'start';
            const paramsToStringFn = params['toString'];
            const paramsToString = [
              paramsToStringFn.call(params, side = 'params.toString.call'),
              side
            ].join(',');
            side = 'start';
            const paramsEntries = [
              Array.from(params.entries.call(params, side = 'params.entries.call'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');

            document.getElementById('result').textContent = [
              mapGet,
              mapProtoEntries,
              setProtoAdd,
              weakMapProtoGet,
              weakSetProtoHas,
              urlToString,
              urlProtoToJson,
              paramsToString,
              paramsEntries
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1,map.get.call|a:1,map.proto.entries|true,set.proto.add,x;y|v,weakMap.proto.get|true,weakSet.proto.has|https://example.com/path?b=2&a=1&a=3#hash,url.toString.call|https://example.com/path?b=2&a=1&a=3#hash,url.proto.toJSON|b=2&a=1&a=3,params.toString.call|b:2;a:1;a:3,params.entries.call",
    )?;
    Ok(())
}

#[test]
fn raw_url_location_getter_and_collection_bracket_parity_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let side = 'start';

            const url = new URL('https://example.com/path?x=1#h');
            const urlToString = url['toString'];
            const urlToJson = url['toJSON'];
            const urlBracket = [
              urlToString.call(url, side = 'url.bracket.toString'),
              side,
              String(url['length'] === url.href.length),
              String(url[8] === url.href[8]),
              urlToJson.call(url)
            ].join(',');

            side = 'start';
            const locationToString = location['toString'];
            const locationBracket = [
              String(locationToString.call(location, side = 'location.bracket.toString') === location.href),
              side,
              String(location['length'] === location.href.length),
              String(location[0] === location.href[0])
            ].join(',');

            const map = new Map([['a', 1]]);
            side = 'start';
            const mapEntries = map['entries'];
            const mapBracket = [
              Array.from(mapEntries.call(map, side = 'map.bracket.entries'))
                .map((pair) => pair.join(':'))
                .join(';'),
              side
            ].join(',');

            let bad = 'none';
            try {
              locationToString.call(url);
            } catch (e) {
              bad = String(e);
            }

            document.getElementById('result').textContent = [
              urlBracket,
              locationBracket,
              mapBracket,
              bad
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/path?x=1#h,url.bracket.toString,true,true,https://example.com/path?x=1#h|true,location.bracket.toString,true,true|a:1,map.bracket.entries|Location method called on incompatible receiver",
    )?;
    Ok(())
}

#[test]
fn url_file_invalid_authority_and_serialization_matrix_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const invalid = [
              'file://server:8080/share/file.txt',
              'file://u:p@server/share/file.txt',
              'file://u@server/share/file.txt',
              'file://localhost:8080/Users/me/test.txt'
            ];
            const canParse = invalid.map((value) => URL.canParse(value)).join(',');
            const parsed = invalid.map((value) => URL.parse(value) === null).join(',');
            const constructed = invalid.map((value) => {
              try {
                new URL(value);
                return 'false';
              } catch (err) {
                return String(err).includes('Invalid URL');
              }
            }).join(',');

            const hrefSetter = (() => {
              const url = new URL('file://server/share/file.txt');
              try {
                url.href = 'file://server:8080/share/file.txt';
                return 'false';
              } catch (err) {
                return String(err).includes('Invalid URL');
              }
            })();

            const localhost = new URL('FiLe://LOCALHOST/Users/Me/Test.txt');
            const server = new URL('FiLe://SeRVer/Share/File.txt');

            document.getElementById('result').textContent = [
              canParse,
              parsed,
              constructed,
              hrefSetter,
              [localhost.href, localhost.origin, localhost.host, localhost.hostname].join(','),
              [server.href, server.origin, server.host, server.hostname].join(',')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false,false,false,false|true,true,true,true|true,true,true,true|true|file:///Users/Me/Test.txt,null,,|file://server/Share/File.txt,null,server,server",
    )?;
    Ok(())
}

#[test]
fn url_generic_invalid_authority_and_setter_port_matrix_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const invalid = [
              'http://example.com:abc/',
              'https://example.com:65536/',
              'http://[::1/'
            ];
            const canParse = invalid.map((value) => URL.canParse(value)).join(',');
            const parsed = invalid.map((value) => URL.parse(value) === null).join(',');
            const constructed = invalid.map((value) => {
              try {
                new URL(value);
                return 'false';
              } catch (err) {
                return String(err).includes('Invalid URL');
              }
            }).join(',');

            const leading = new URL('https://Example.COM:080/a');

            const hostInvalidPort = new URL('https://base.test:8080/root/page');
            hostInvalidPort.host = 'Example.com:abc';

            const hostIpv6KeepPort = new URL('https://base.test:8080/root/page');
            hostIpv6KeepPort.host = '[::1]:abc';

            const hostInvalidIpv6 = new URL('https://base.test:8080/root/page');
            hostInvalidIpv6.host = '[::1';

            const hostnameInvalid = new URL('https://base.test:8080/root/page');
            hostnameInvalid.hostname = 'example.com:123';

            const portCanonical = new URL('https://base.test:8080/root/page');
            portCanonical.port = '09090';

            const portInvalid = new URL('https://base.test:8080/root/page');
            portInvalid.port = '99999';

            document.getElementById('result').textContent = [
              leading.href,
              canParse,
              parsed,
              constructed,
              hostInvalidPort.href,
              hostIpv6KeepPort.href,
              hostInvalidIpv6.href,
              hostnameInvalid.href,
              portCanonical.href,
              portInvalid.href
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com:80/a|false,false,false|true,true,true|true,true,true|https://example.com:8080/root/page|https://[::1]:8080/root/page|https://base.test:8080/root/page|https://base.test:8080/root/page|https://base.test:9090/root/page|https://base.test:8080/root/page",
    )?;
    Ok(())
}

#[test]
fn url_special_host_edge_matrix_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const constructed = [
              new URL('http:foo').href,
              new URL('https:/Example.COM:080/docs').href,
              new URL('http:\\Example.COM\\docs\\a?x=1#h').href,
              new URL('http://example.com:').href
            ].join(',');

            const invalid = [
              'http://',
              'http:?x',
              'http://?x'
            ].map((value) => {
              const canParse = URL.canParse(value);
              const parsed = URL.parse(value) === null;
              const constructed = (() => {
                try {
                  new URL(value);
                  return 'false';
                } catch (err) {
                  return String(err).includes('Invalid URL');
                }
              })();
              return [canParse, parsed, constructed].join(':');
            }).join(',');

            const hrefSetter = (() => {
              const url = new URL('https://base.test/root');
              url.href = 'https:Example.COM:080/next';
              return url.href;
            })();

            const backslashSetter = (() => {
              const url = new URL('https://base.test/root');
              url.href = 'http:\\Example.COM\\p';
              return url.href;
            })();

            document.getElementById('result').textContent = [
              constructed,
              invalid,
              hrefSetter,
              backslashSetter
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://foo/,https://example.com:80/docs,http://example.com/docs/a?x=1#h,http://example.com/|false:true:true,false:true:true,false:true:true|https://example.com:80/next|http://example.com/p",
    )?;
    Ok(())
}

#[test]
fn url_credential_and_delimiter_encoding_matrix_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ctor = new URL('https://a@b:p@q:r@example.com/p q?x y#z z');

            const special = new URL('https://u:p@example.com/base');
            special.username = 'a@b';
            special.password = 'p@q:r';
            special.pathname = '\\docs\\a b';
            special.search = "a'b";
            special.hash = 'x`y';

            const custom = new URL('foo://example.com/base');
            custom.pathname = '\\docs\\a b';
            custom.search = "a'b";
            custom.hash = 'x`y';

            const file = new URL('file:///Users/me/base');
            file.username = 'a@b';
            file.password = 'p@q:r';
            file.pathname = '\\docs\\a b';
            file.search = "a'b";
            file.hash = 'x`y';

            const mail = new URL('mailto:test@example.com?x=1#h');
            mail.username = 'a@b';
            mail.password = 'p@q:r';
            mail.pathname = 'ignored';
            mail.search = "a'b";
            mail.hash = 'x`y';

            document.getElementById('result').textContent = [
              [ctor.href, ctor.username, ctor.password, ctor.pathname, ctor.search, ctor.hash].join(','),
              [special.href, special.username, special.password, special.pathname, special.search, special.hash].join(','),
              [custom.href, custom.username, custom.password, custom.pathname, custom.search, custom.hash].join(','),
              [file.href, file.username, file.password, file.pathname, file.search, file.hash].join(','),
              [mail.href, mail.username, mail.password, mail.pathname, mail.search, mail.hash].join(',')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://a%40b:p%40q%3Ar@example.com/p%20q?x%20y#z%20z,a%40b,p%40q%3Ar,/p%20q,?x%20y,#z%20z|https://a%40b:p%40q%3Ar@example.com/docs/a%20b?a%27b#x%60y,a%40b,p%40q%3Ar,/docs/a%20b,?a%27b,#x%60y|foo://example.com/\\docs\\a%20b?a'b#x%60y,,,/\\docs\\a%20b,?a'b,#x%60y|file:///docs/a%20b?a%27b#x%60y,,,/docs/a%20b,?a%27b,#x%60y|mailto:test@example.com?a'b#x%60y,,,test@example.com,?a'b,#x%60y",
    )?;
    Ok(())
}

#[test]
fn url_authority_and_percent_residual_matrix_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ctor = new URL('https://user%zz:pa%2fss@ExA%41mple.ORG/%2f%zz?x=%2f%zz#y=%2f%zz');

            const invalid = [
              'https://exa%mple.org/',
              'https://exa%25mple.org/',
              'https://exa%2fmple.org/'
            ];
            const canParse = invalid.map((value) => URL.canParse(value)).join(',');
            const parsed = invalid.map((value) => URL.parse(value) === null).join(',');
            const constructed = invalid.map((value) => {
              try {
                new URL(value);
                return 'false';
              } catch (err) {
                return String(err).includes('Invalid URL');
              }
            }).join(',');

            const hrefSetter = (() => {
              const url = new URL('https://base.test/root');
              url.href = 'https://%41example.com/%2f%zz?x=%2f%zz#y=%2f%zz';
              const valid = [url.href, url.host, url.pathname, url.search, url.hash].join(',');
              try {
                url.href = 'https://exa%mple.org/';
                return [valid, 'false', url.href].join(';');
              } catch (err) {
                return [
                  valid,
                  String(err).includes('Invalid URL'),
                  url.href
                ].join(';');
              }
            })();

            const hostSetters = (() => {
              const url = new URL('https://base.test:8080/root');
              url.host = 'ExA%41mple.ORG:0099';
              const afterHost = [url.href, url.host, url.hostname].join(',');
              url.host = 'exa%mple.org:77';
              const afterBadHost = [url.href, url.host, url.hostname].join(',');
              url.hostname = '%41lt.EXAMPLE.com';
              const afterHostname = [url.href, url.host, url.hostname].join(',');
              url.hostname = 'exa%mple.org';
              const afterBadHostname = [url.href, url.host, url.hostname].join(',');
              return [
                afterHost,
                afterBadHost,
                afterHostname,
                afterBadHostname
              ].join(';');
            })();

            const userinfoEdges = [
              new URL('https://a@@example.com/'),
              new URL('https://:pass@example.com/'),
              new URL('https://user:@example.com/')
            ].map((url) => [url.href, url.username, url.password].join(',')).join(';');

            const setterPreserve = (() => {
              const url = new URL('https://example.com/base');
              url.username = 'a%zz';
              url.password = 'b%2f';
              url.pathname = '%2f%zz';
              url.search = '%2f%zz';
              url.hash = '%2f%zz';
              return [
                url.href,
                url.username,
                url.password,
                url.pathname,
                url.search,
                url.hash
              ].join(',');
            })();

            const custom = (() => {
              const url = new URL('foo://example.com/base');
              url.pathname = '%2f%zz';
              url.search = '%2f%zz';
              url.hash = '%2f%zz';
              return [url.href, url.pathname, url.search, url.hash].join(',');
            })();

            const opaque = new URL('mailto:test@example.com%zz?x=%2f%zz#y=%2f%zz');

            document.getElementById('result').textContent = [
              [ctor.href, ctor.username, ctor.password, ctor.pathname, ctor.search, ctor.hash].join(','),
              canParse,
              parsed,
              constructed,
              hrefSetter,
              hostSetters,
              userinfoEdges,
              setterPreserve,
              custom,
              [opaque.href, opaque.pathname, opaque.search, opaque.hash].join(',')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://user%zz:pa%2fss@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,user%zz,pa%2fss,/%2f%zz,?x=%2f%zz,#y=%2f%zz|false,false,false|true,true,true|true,true,true|https://aexample.com/%2f%zz?x=%2f%zz#y=%2f%zz,aexample.com,/%2f%zz,?x=%2f%zz,#y=%2f%zz;true;https://aexample.com/%2f%zz?x=%2f%zz#y=%2f%zz|https://exaample.org:99/root,exaample.org:99,exaample.org;https://exaample.org:99/root,exaample.org:99,exaample.org;https://alt.example.com:99/root,alt.example.com:99,alt.example.com;https://alt.example.com:99/root,alt.example.com:99,alt.example.com|https://a%40@example.com/,a%40,;https://:pass@example.com/,,pass;https://user@example.com/,user,|https://a%zz:b%2f@example.com/%2f%zz?%2f%zz#%2f%zz,a%zz,b%2f,/%2f%zz,?%2f%zz,#%2f%zz|foo://example.com/%2f%zz?%2f%zz#%2f%zz,/%2f%zz,?%2f%zz,#%2f%zz|mailto:test@example.com%zz?x=%2f%zz#y=%2f%zz,test@example.com%zz,?x=%2f%zz,#y=%2f%zz",
    )?;
    Ok(())
}

#[test]
fn url_static_methods_and_blob_object_urls_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const blob = new Blob(['abc'], { type: 'text/plain' });

            const objectUrl1 = URL.createObjectURL(blob);
            URL.revokeObjectURL(objectUrl1);
            const objectUrl2 = window.URL.createObjectURL(blob);
            URL.revokeObjectURL('blob:bt-999');

            const canRel = URL.canParse('../cats', 'http://www.example.com/dogs');
            const canBad = URL.canParse('/cats');

            const parsed = URL.parse('../cats', 'http://www.example.com/dogs');
            const parsedHref = parsed === null ? 'null' : parsed.href;
            const parsedBad = URL.parse('/cats') === null;

            const C = URL;
            const viaAlias = C.canParse('https://example.com/path');

            document.getElementById('result').textContent =
              objectUrl1 + '|' +
              objectUrl2 + '|' +
              canRel + ':' + canBad + '|' +
              parsedHref + ':' + parsedBad + ':' + viaAlias;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "blob:bt-1|blob:bt-2|true:false|http://www.example.com/cats:true:true",
    )?;
    Ok(())
}

#[test]
fn url_constructor_requires_new() {
    let err = Harness::from_html("<script>URL('https://example.com');</script>")
        .expect_err("calling URL constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("URL constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn url_constructor_without_base_rejects_relative_urls() {
    let err = Harness::from_html("<script>new URL('/cats');</script>")
        .expect_err("relative URL without base should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Invalid URL")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn url_href_setter_rejects_relative_urls() {
    let err = Harness::from_html(
        "<script>const u = new URL('https://example.com/a'); u['href'] = '/cats';</script>",
    )
    .expect_err("setting URL.href to a relative URL should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Invalid URL")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn blob_constructor_properties_and_text_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { hello: 'world' };
            const blob = new Blob([JSON.stringify(obj)], { type: 'Application/JSON' });
            const promise = blob.text();
            promise.then((text) => {
              document.getElementById('result').textContent =
                blob.size + ':' +
                blob.type + ':' +
                text + ':' +
                (blob.constructor === Blob);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "17:application/json:{\"hello\":\"world\"}:true")?;
    Ok(())
}

#[test]
fn blob_array_buffer_bytes_slice_and_stream_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const source = new Uint8Array([65, 66, 67, 68]);
            const blob = new Blob([source.buffer], { type: 'text/plain' });
            const sliced = blob.slice(1, 3);
            const p1 = blob.arrayBuffer();
            const p2 = blob.bytes();
            const p3 = sliced.text();

            Promise.all([p1, p2, p3]).then((values) => {
              const fromAb = new Uint8Array(values[0]).join(',');
              const fromBytes = values[1].join(',');
              const streamObj = blob.stream();
              document.getElementById('result').textContent =
                fromAb + '|' +
                fromBytes + '|' +
                values[2] + '|' +
                (typeof streamObj) + ':' +
                (streamObj ? 'y' : 'n');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "65,66,67,68|65,66,67,68|BC|object:y")?;
    Ok(())
}

#[test]
fn blob_constructor_and_method_errors_work() {
    let err = Harness::from_html("<script>Blob(['x']);</script>")
        .expect_err("calling Blob constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Blob constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const b = new Blob(['x']); b.text('oops');</script>")
        .expect_err("Blob.text should reject extra arguments");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Blob.text does not take arguments")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn array_buffer_properties_slice_and_is_view_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const buffer = new ArrayBuffer(8, { maxByteLength: 16 });
          const view = new Uint8Array(buffer);
          view.set([10, 20, 30, 40]);

          document.getElementById('btn').addEventListener('click', () => {
            const sliced = buffer.slice(1, 3);
            const slicedView = new Uint8Array(sliced);
            document.getElementById('result').textContent =
              buffer.byteLength + ':' +
              buffer.resizable + ':' +
              buffer.maxByteLength + ':' +
              buffer.detached + ':' +
              ArrayBuffer.isView(view) + ':' +
              ArrayBuffer.isView(buffer) + ':' +
              sliced.byteLength + ':' +
              sliced.resizable + ':' +
              sliced.maxByteLength + ':' +
              (buffer.constructor === ArrayBuffer) + ':' +
              slicedView.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "8:true:16:false:true:false:2:false:2:true:20,30")?;
    Ok(())
}

#[test]
fn array_buffer_transfer_and_transfer_to_fixed_length_detach_source() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const buffer = new ArrayBuffer(6, { maxByteLength: 12 });
            const view = new Uint8Array(buffer);
            view.set([1, 2, 3, 4, 5, 6]);

            const moved = buffer.transfer();
            const fixed = moved.transferToFixedLength();
            const fixedView = new Uint8Array(fixed);

            document.getElementById('result').textContent =
              buffer.detached + ':' +
              buffer.byteLength + ':' +
              view.byteLength + ':' +
              moved.detached + ':' +
              moved.byteLength + ':' +
              moved.resizable + ':' +
              moved.maxByteLength + ':' +
              fixed.detached + ':' +
              fixed.byteLength + ':' +
              fixed.resizable + ':' +
              fixed.maxByteLength + ':' +
              fixedView.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:0:0:true:0:false:0:false:6:false:6:1,2,3,4,5,6",
    )?;
    Ok(())
}

#[test]
fn array_buffer_detached_behavior_errors_work() {
    let err = Harness::from_html(
            "<script>const b = new ArrayBuffer(4, { maxByteLength: 8 }); b.transfer(); b.resize(2);</script>",
        )
        .expect_err("resize on detached ArrayBuffer should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("detached ArrayBuffer")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html(
        "<script>const b = new ArrayBuffer(4); b.transfer(); b.slice(0, 1);</script>",
    )
    .expect_err("slice on detached ArrayBuffer should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("detached ArrayBuffer")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html(
            "<script>const b = new ArrayBuffer(4); const ta = new Uint8Array(b); b.transfer(); ta.fill(1);</script>",
        )
        .expect_err("typed array methods on detached backing buffer should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("detached ArrayBuffer")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn array_buffer_is_view_arity_and_transfer_arity_errors_work() {
    let err = Harness::from_html("<script>ArrayBuffer.isView();</script>")
        .expect_err("ArrayBuffer.isView without args should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("ArrayBuffer.isView requires exactly one argument"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const b = new ArrayBuffer(4); b.transfer(1);</script>")
        .expect_err("ArrayBuffer.transfer with args should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("ArrayBuffer.transfer does not take arguments"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn object_prototype_has_own_property_call_expression_works() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const payload = { sku: 'A-1', qty: 12 };
            const hasQty = Object.prototype.hasOwnProperty.call(payload, 'qty');
            const hasNote = Object.prototype.hasOwnProperty.call(payload, 'note');
            document.getElementById('result').textContent = hasQty + ':' + hasNote;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false")?;
    Ok(())
}

#[test]
fn set_checked_handles_input_handler_that_rerenders_same_subtree() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <script>
          const state = { checked: false };
          const root = document.getElementById('root');

          function render() {
            root.innerHTML =
              '<label><input id="agree" type="checkbox" ' +
              (state.checked ? 'checked' : '') +
              '></label>';
          }

          root.addEventListener('input', (event) => {
            const target = event.target;
            if (!(target instanceof HTMLInputElement)) return;
            state.checked = target.checked;
            render();
          });

          render();
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_checked("#agree", true)?;
    h.assert_checked("#agree", true)?;
    h.set_checked("#agree", false)?;
    h.assert_checked("#agree", false)?;
    Ok(())
}

#[test]
fn click_summary_toggles_details_open_state() -> Result<()> {
    let html = r#"
        <details id='panel' open>
          <summary id='toggle'>Panel</summary>
          <p>body</p>
        </details>
        <button id='read'>read</button>
        <p id='result'></p>
        <script>
          document.getElementById('read').addEventListener('click', () => {
            const open = document.getElementById('panel').open;
            document.getElementById('result').textContent = open ? 'open' : 'closed';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#read")?;
    h.assert_text("#result", "open")?;
    h.click("#toggle")?;
    h.click("#read")?;
    h.assert_text("#result", "closed")?;
    h.click("#toggle")?;
    h.click("#read")?;
    h.assert_text("#result", "open")?;
    Ok(())
}

#[test]
fn function_reference_chain_inside_event_callback_uses_same_scope_declarations() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <div id='result'></div>
        <script>
          (() => {
            const el = document.getElementById('btn');

            function render() {
              document.getElementById('result').textContent = compute();
            }

            function onStateUpdated() {
              render();
            }

            function compute() {
              return 'ok';
            }

            el.addEventListener('click', () => {
              onStateUpdated();
            });
          })();
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn callback_can_resolve_later_function_declaration_hide_toast() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <button id='toast-action'>action</button>
        <div id='toast' class='show'></div>
        <div id='result'></div>
        <script>
          (() => {
            const el = {
              btn: document.getElementById('btn'),
              toast: document.getElementById('toast'),
              toastAction: document.getElementById('toast-action'),
            };

            function showToast(action) {
              el.toast.classList.add('show');
              el.toastAction.onclick = () => {
                hideToast();
                action();
              };
            }

            function hideToast() {
              el.toast.classList.remove('show');
            }

            el.btn.addEventListener('click', () => {
              showToast(() => {
                document.getElementById('result').textContent = el.toast.className + ':ok';
              });
            });
          })();
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.click("#toast-action")?;
    h.assert_text("#result", ":ok")?;
    Ok(())
}

#[test]
fn event_listener_callback_can_resolve_later_function_declaration() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <div id='result'></div>
        <script>
          (() => {
            const btn = document.getElementById('btn');
            btn.addEventListener('click', () => {
              hideToast();
            });

            function hideToast() {
              document.getElementById('result').textContent = 'ok';
            }
          })();
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn function_calls_share_outer_scope_variable_updates() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          (() => {
            let computed = [];
            const state = [1, 2, 3];

            function computeAll() {
              computed = state.map((value) => value * 2);
            }

            function render() {
              document.getElementById('result').textContent = String(computed.length);
            }

            function renderAll() {
              computeAll();
              render();
            }

            document.getElementById('run').addEventListener('click', () => {
              renderAll();
            });
          })();
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn listeners_registered_in_same_scope_share_captured_bindings() -> Result<()> {
    let html = r#"
        <button id='load'>load</button>
        <button id='copy'>copy</button>
        <p id='result'></p>
        <script>
          (() => {
            let computedRows = [];

            function renderAll() {
              computedRows = [1, 2];
            }

            function buildResultSummary() {
              return String(computedRows.length);
            }

            document.getElementById('load').addEventListener('click', () => {
              renderAll();
            });

            document.getElementById('copy').addEventListener('click', () => {
              document.getElementById('result').textContent = buildResultSummary();
            });
          })();
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#load")?;
    h.click("#copy")?;
    h.assert_text("#result", "2")?;
    Ok(())
}

#[test]
fn const_arrow_assignment_with_template_literal_body_parses() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          (() => {
            const safeNumber = (raw, fallback) => {
              const parsed = Number(String(raw || "").replace(/,/g, "").replace(/[^0-9.-]/g, ""));
              return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
            };
            const yen = (v) => `¥${Math.max(0, Math.round(v)).toLocaleString("ja-JP")}`;
            const riskBand = (score) => {
              if (score >= 60) return "veryHigh";
              if (score >= 40) return "high";
              if (score >= 20) return "medium";
              return "low";
            };
            document.getElementById('result').textContent =
              safeNumber("1,234", 0) + ":" + yen(1250) + ":" + riskBand(45);
          })();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1234:¥1,250:high")?;
    Ok(())
}

#[test]
fn template_literal_with_math_call_and_member_call_parses() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const formatted = `¥${Math.max(0, Math.round(1250)).toLocaleString("ja-JP")}`;
          document.getElementById('result').textContent = formatted;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "¥1,250")?;
    Ok(())
}

#[test]
fn nested_object_member_logical_expression_parses() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj = { a: { b: {}, c: {} } };
          const value = (obj.a && (obj.a.b || obj.a.c));
          document.getElementById('result').textContent = value ? 'ok' : 'ng';
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn nested_object_literal_parses_without_recursion_overflow() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj = { a: { b: {}, c: {} } };
          document.getElementById('result').textContent =
            obj && obj.a && obj.a.b && obj.a.c ? 'ok' : 'ng';
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn deeply_nested_binary_expression_evaluates_on_small_thread_stack() -> Result<()> {
    let mut expression = String::from("Date.now()");
    for _ in 0..12_000 {
        expression.push_str(" + 1");
    }

    let html = format!(
        r#"
        <p id='result'></p>
        <script>
          const value = {expression};
          document.getElementById('result').textContent = value > 0 ? 'ok' : 'ng';
        </script>
        "#
    );

    let handle = std::thread::Builder::new()
        .name("small-stack-runtime-eval".to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || -> std::result::Result<(), String> {
            let h = Harness::from_html(&html).map_err(|err| err.to_string())?;
            h.assert_text("#result", "ok")
                .map_err(|err| err.to_string())?;
            Ok(())
        })
        .map_err(|err| Error::ScriptRuntime(err.to_string()))?;

    handle
        .join()
        .map_err(|_| Error::ScriptRuntime("small-stack runtime thread panicked".into()))?
        .map_err(Error::ScriptRuntime)?;

    Ok(())
}

#[test]
fn arrow_function_with_object_destructuring_parameter_parses() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const render = ({ growthFlag, arrearsFlag, uncertain }) => {
            document.getElementById('result').textContent =
              String(growthFlag) + ':' + String(arrearsFlag) + ':' + String(uncertain);
          };
          render({ growthFlag: true, arrearsFlag: false, uncertain: 2 });
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true:false:2")?;
    Ok(())
}

#[test]
fn arrow_function_with_object_destructuring_after_prior_call_and_object_updates_parses()
-> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          (() => {
            const buildRenderer = () => {
              const latest = { actions: [] };
              const render = ({ growthFlag, arrearsFlag, uncertain }) => {
                latest.actions.forEach((text, idx) => {
                  const input = document.createElement("input");
                  input.id = "action-" + idx;
                  input.addEventListener("change", () => {
                    if (input.checked) {
                      document.getElementById('result').textContent = text;
                    }
                  });
                });
                return String(growthFlag) + ":" + String(arrearsFlag) + ":" + String(uncertain);
              };
              return render;
            };
            const render = buildRenderer();
            document.getElementById('result').textContent =
              render({ growthFlag: false, arrearsFlag: false, uncertain: false });
          })();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "false:false:false")?;
    Ok(())
}

#[test]
fn array_for_each_concise_callback_with_member_call_body_parses() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const btn = {};
          const el = { mobileTabButtons: [] };
          el.mobileTabButtons.forEach((b) => b.classList.toggle("active", b === btn));
          document.getElementById('result').textContent = 'ok';
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn array_typed_array_and_collection_iterator_property_paths_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const arr = [1, 2, 3];
            const arrResult = [
              arr['map'].call(arr, (value) => value * 3).join(','),
              arr['slice'].call(arr, 1).join(','),
              Array.from(arr[Symbol.iterator].call(arr)).join(',')
            ].join(';');

            const typed = new Uint8Array([4, 5, 6]);
            const typedResult = [
              typed['slice'].call(typed, 1).join(','),
              Array.from(typed[Symbol.iterator].call(typed)).join(','),
              Array.from(typed['entries'].call(typed))
                .map((pair) => pair.join(':'))
                .join(',')
            ].join(';');

            const map = new Map([['a', 1], ['b', 2]]);
            const set = new Set(['x', 'y']);
            const params = new URLSearchParams('q=1&q=2&r=3');
            const mapIterator = map[Symbol.iterator].call(map);
            const setIterator = set[Symbol.iterator].call(set);
            const paramsIterator = params[Symbol.iterator].call(params);
            const mapSelf = mapIterator[Symbol.iterator].call(mapIterator).next().value.join(':');
            const setSelf = setIterator[Symbol.iterator].call(setIterator).next().value;
            const paramsSelf =
              paramsIterator[Symbol.iterator].call(paramsIterator).next().value.join(':');
            const collectionResult = [
              Array.from(mapIterator).map((pair) => pair.join(':')).join(','),
              String(mapSelf),
              Array.from(setIterator).join(','),
              String(setSelf),
              Array.from(paramsIterator).map((pair) => pair.join(':')).join(','),
              String(paramsSelf)
            ].join(';');

            document.getElementById('result').textContent = [
              arrResult,
              typedResult,
              collectionResult
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "3,6,9;2,3;1,2,3|5,6;4,5,6;0:4,1:5,2:6|a:1,b:2;a:1;x,y;x;q:1,q:2,r:3;q:1",
    )?;
    Ok(())
}

#[test]
fn typed_array_raw_getter_breadth_and_constructor_prototype_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const base = new Uint8Array([10, 20, 30, 40]);
            const copied = new Uint8Array([1, 2, 3, 4]);
            const typedResult = [
              String(base['at'].call(base, -1)),
              Array.from(copied['copyWithin'].call(copied, 1, 2)).join(','),
              Array.from(base['subarray'].call(base, 1, 3)).join(','),
              Array.from(base['with'].call(base, -1, 99)).join(','),
              String(base.constructor.prototype.at.call(base, 0)),
              Array.from(base.constructor.prototype.subarray.call(base, 2)).join(','),
              Array.from(base.constructor.prototype.with.call(base, 1, 77)).join(',')
            ].join(';');

            document.getElementById('result').textContent = typedResult;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "40;1,3,4,4;20,30;10,20,30,99;10;30,40;10,77,30,40",
    )?;
    Ok(())
}

#[test]
fn constructor_static_bracket_and_property_path_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const holder = { Number, BigInt, String, Symbol, Int8Array };
            const result = [
              String(holder.Number['parseInt']('101', 2)),
              String(holder.Number['isInteger'](7)),
              String(holder.Number['MAX_SAFE_INTEGER'] === Number.MAX_SAFE_INTEGER),
              String(holder.BigInt['asIntN'](8, 257n)),
              holder.String['fromCharCode'](65, 66),
              holder.String['fromCodePoint'](0x1F63A),
              String(holder.Symbol['iterator'] === Symbol.iterator),
              Array.from(holder.Int8Array['of'](1, 2, 3)).join(','),
              Array.from(holder.Int8Array['from']([4, 5])).join(','),
              String(holder.Int8Array['BYTES_PER_ELEMENT'])
            ].join(';');

            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "5;true;true;1;AB;😺;true;1,2,3;4,5;1")?;
    Ok(())
}

#[test]
fn constructor_static_identity_and_new_callee_alias_paths_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const holder = { String, Symbol, Int8Array };
            const key = 'Int8Array';
            const viaNew = new holder[key]([9, 10]);
            const result = [
              String(holder.String['fromCharCode'] === holder.String.fromCharCode),
              String(holder.String['fromCodePoint'] === holder.String.fromCodePoint),
              String(holder.String['raw'] === holder.String.raw),
              String(holder.Symbol['for'] === holder.Symbol.for),
              String(holder.Symbol['keyFor'] === holder.Symbol.keyFor),
              String(holder.Int8Array['of'] === holder.Int8Array.of),
              String(holder.Int8Array['from'] === holder.Int8Array.from),
              Array.from(viaNew).join(','),
              Array.from(globalThis[key]['of'](3, 4)).join(',')
            ].join(';');

            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true;true;true;true;true;true;true;9,10;3,4")?;
    Ok(())
}

#[test]
fn constructor_function_surface_and_global_bindings_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const params = new globalThis['URLSearchParams']('a=1&b=2');
            const map = new globalThis['Map']([['k', 'v']]);
            const url = new globalThis['URL']('/path?x=1', 'https://example.com/base/');
            const blob = new globalThis['Blob'](['ab'], { type: 'text/plain' });
            const buffer = new globalThis['ArrayBuffer'](3);
            let callError = '';
            try {
              Map.call(null);
            } catch (err) {
              callError = String(err && err.message ? err.message : err);
            }
            const result = [
              String(globalThis.Map === Map),
              String(globalThis.URLSearchParams === URLSearchParams),
              String(globalThis.ArrayBuffer === ArrayBuffer),
              String(globalThis.Blob === Blob),
              String(Map.call === Map.call),
              String(Map.call === Number.call),
              String(Map.apply === Number.apply),
              String(Map.bind === Number.bind),
              String(Map['toString'] === Number['toString']),
              Map.name,
              String(Map.length),
              URL.name,
              String(URL.length),
              URLSearchParams.name,
              String(URLSearchParams.length),
              ArrayBuffer.name,
              String(ArrayBuffer.length),
              String(Map.prototype.constructor === Map),
              String(URL.prototype.constructor === URL),
              String(URLSearchParams.prototype.constructor === URLSearchParams),
              String(ArrayBuffer.prototype.constructor === ArrayBuffer),
              String(Blob.prototype.constructor === Blob),
              params.toString(),
              String(map.get('k')),
              url.href,
              String(blob.size) + ':' + blob.type,
              String(buffer.byteLength),
              callError
            ].join(';');

            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true;true;true;true;true;true;true;true;true;Map;0;URL;1;URLSearchParams;0;ArrayBuffer;1;true;true;true;true;true;a=1&b=2;v;https://example.com/path?x=1;2:text/plain;3;Map constructor must be called with new",
    )?;
    Ok(())
}

#[test]
fn constructor_raw_static_and_prototype_property_paths_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const BlobCtor = globalThis['Blob'];
            const ArrayBufferCtor = globalThis['ArrayBuffer'];
            const PromiseCtor = globalThis['Promise'];
            const RegExpCtor = globalThis['RegExp'];
            const blob = new BlobCtor(['abcd'], { type: 'text/plain' });
            const buffer = new ArrayBufferCtor(4);
            const view = new Uint8Array(buffer);
            view.set([1, 2, 3, 4]);
            const re = RegExpCtor('a.', 'g');
            const protoInfo = [
              String(BlobCtor.prototype === BlobCtor.prototype),
              String(ArrayBufferCtor.prototype === ArrayBufferCtor.prototype),
              String(PromiseCtor.prototype === PromiseCtor.prototype),
              String(RegExpCtor.prototype === RegExpCtor.prototype),
              String(PromiseCtor['resolve'] === PromiseCtor.resolve),
              String(ArrayBufferCtor['isView'] === ArrayBufferCtor.isView),
              String(RegExpCtor['escape'] === RegExpCtor.escape)
            ].join(':');
            const bag = PromiseCtor['withResolvers']();

            PromiseCtor.prototype['then'].call(
              BlobCtor.prototype['text'].call(
                BlobCtor.prototype['slice'].call(blob, 1, 3)
              ),
              (text) => {
                PromiseCtor.prototype['then'].call(
                  PromiseCtor['resolve']('ok'),
                  (ok) => {
                    const match = RegExpCtor.prototype['exec'].call(re, 'baac')[0];
                    const sliced = ArrayBufferCtor.prototype['slice'].call(buffer, 1, 3);
                    const slicedView = new Uint8Array(sliced);
                    bag.resolve([
                      protoInfo,
                      text,
                      ok,
                      String(ArrayBufferCtor['isView'](view)),
                      slicedView.join(','),
                      match,
                      RegExpCtor['escape']('a+b')
                    ].join(';'));
                  }
                );
              }
            );

            PromiseCtor.prototype['then'].call(bag.promise, (value) => {
              document.getElementById('result').textContent = value;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:true:true;bc;ok;true;2,3;aa;\\x61\\+b",
    )?;
    Ok(())
}

#[test]
fn native_variant_backed_constructor_source_text_is_stable_across_paths_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          function describe(name) {
            const ctor = globalThis[name];
            const viaMethod = ctor.toString();
            const viaCall = Function.prototype.toString.call(ctor);
            const viaString = String(ctor);
            const viaNewString = new String(ctor).valueOf();
            const viaAlias = globalThis[name].toString();
            const viaBracket = globalThis[name]['toString']();
            return [
              String(viaMethod.includes('[native code]')),
              String(viaMethod.includes(name)),
              String(viaMethod === viaCall),
              String(viaMethod === viaString),
              String(viaMethod === viaNewString),
              String(viaMethod === viaAlias),
              String(viaMethod === viaBracket)
            ].join(':');
          }

          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = [
              describe('Map'),
              describe('URL'),
              describe('URLSearchParams'),
              describe('ArrayBuffer'),
              describe('Promise'),
              describe('RegExp'),
              describe('Blob')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:true:true|true:true:true:true:true:true:true|true:true:true:true:true:true:true|true:true:true:true:true:true:true|true:true:true:true:true:true:true|true:true:true:true:true:true:true|true:true:true:true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn builtin_instanceof_and_object_get_prototype_of_parity_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const map = new Map([['k', 1]]);
            const set = new Set(['v']);
            const params = new URLSearchParams('a=1');
            const url = new URL('/next', 'https://example.com/base/');
            const buffer = new ArrayBuffer(2);
            const blob = new Blob(['ok']);
            const promise = Promise.resolve('done');
            const re = /a/g;
            const view = Int8Array.of(1, 2);
            const typedArrayCtor = Object.getPrototypeOf(Int8Array);
            const check = (label, fn) => {
              try {
                return label + ':' + fn();
              } catch (err) {
                return label + ':ERR:' + String(err && err.message ? err.message : err);
              }
            };
            const result = [
              check('map', () => [
                String(Object.getPrototypeOf(map) === Map.prototype),
                String(map instanceof Map),
                String(map instanceof Object)
              ].join(':')),
              check('set', () => [
                String(Object.getPrototypeOf(set) === Set.prototype),
                String(set instanceof Set),
                String(set instanceof Object)
              ].join(':')),
              check('params', () => [
                String(params.constructor === URLSearchParams),
                String(Object.getPrototypeOf(params) === URLSearchParams.prototype),
                String(params instanceof URLSearchParams)
              ].join(':')),
              check('url', () => [
                String(url.constructor === URL),
                String(Object.getPrototypeOf(url) === URL.prototype),
                String(url instanceof URL)
              ].join(':')),
              check('buffer', () => [
                String(Object.getPrototypeOf(buffer) === ArrayBuffer.prototype),
                String(buffer instanceof ArrayBuffer),
                String(buffer instanceof Object)
              ].join(':')),
              check('blob', () => [
                String(Object.getPrototypeOf(blob) === Blob.prototype),
                String(blob instanceof Blob),
                String(blob instanceof Object)
              ].join(':')),
              check('promise', () => [
                String(Object.getPrototypeOf(promise) === Promise.prototype),
                String(promise instanceof Promise),
                String(promise instanceof Object)
              ].join(':')),
              check('regexp', () => [
                String(Object.getPrototypeOf(re) === RegExp.prototype),
                String(re instanceof RegExp),
                String(re instanceof Object)
              ].join(':')),
              check('typed', () => [
                String(Object.getPrototypeOf(view) === Int8Array.prototype),
                String(Object.getPrototypeOf(Object.getPrototypeOf(view)).constructor === typedArrayCtor),
                String(view instanceof Int8Array),
                String(view instanceof Object)
              ].join(':'))
            ].join('|');

            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "map:true:true:true|set:true:true:true|params:true:true:true|url:true:true:true|buffer:true:true:true|blob:true:true:true|promise:true:true:true|regexp:true:true:true|typed:true:true:true:true",
    )?;
    Ok(())
}
