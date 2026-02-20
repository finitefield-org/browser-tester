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
    h.assert_text("#result", "1234:¥1250:high")?;
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
    h.assert_text("#result", "¥1250")?;
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
