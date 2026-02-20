use super::*;

#[test]
fn intl_segmenter_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const segmenterFr = new Intl.Segmenter('fr', { granularity: 'word' });
            const string = 'Que ma joie demeure';
            const segments = segmenterFr.segment(string);
            const iteratorFactory = segments[Symbol.iterator];
            const iterator = iteratorFactory();
            const next = iterator.next;
            const first = next();
            const second = next();
            document.getElementById('result').textContent =
              first.value.segment + '|' + second.value.segment + '|' + first.done + ':' + second.done;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Que| |false:false")?;
    Ok(())
}

#[test]
fn intl_segmenter_methods_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const str = '吾輩は猫である。名前はたぬき。';
            const split = str.split(' ');
            const segmenter = new Intl.Segmenter('ja-JP', { granularity: 'word' });
            const segments = segmenter.segment(str);
            const first = segments[0];
            const second = segments[1];
            const count = segments['length'];
            const arr = Array.from(segments);
            const arrFirst = arr[0];
            const supported = Intl.Segmenter.supportedLocalesOf(['fr', 'ja-JP', 'de']);
            const ro = segmenter.resolvedOptions();
            const tag = Intl.Segmenter.prototype[Symbol.toStringTag];
            const ctor = segmenter.constructor === Intl.Segmenter;
            document.getElementById('result').textContent =
              split.length + '|' +
              first.segment + ':' + second.segment + ':' + first.isWordLike + ':' + count + ':' +
              arrFirst.segment + ':' + arr.length + '|' +
              supported.join(',') + '|' +
              ro.locale + ':' + ro.granularity + ':' + ro.localeMatcher + '|' +
              tag + '|' + ctor;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "1|吾輩:は:true:10:吾輩:10|fr,ja-JP|ja-JP:word:best fit|Intl.Segmenter|true",
    )?;
    Ok(())
}

#[test]
fn intl_display_names_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const regionNamesInEnglish = new Intl.DisplayNames(['en'], { type: 'region' });
            const regionNamesInTraditionalChinese = new Intl.DisplayNames(['zh-Hant'], {
              type: 'region',
            });
            document.getElementById('result').textContent =
              regionNamesInEnglish.of('US') + '|' + regionNamesInTraditionalChinese.of('US');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "United States|美國")?;
    Ok(())
}

#[test]
fn intl_display_names_of_examples_for_multiple_types_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const regionNamesEn = new Intl.DisplayNames(['en'], { type: 'region' });
            const regionNamesZh = new Intl.DisplayNames(['zh-Hant'], { type: 'region' });
            const languageNamesEn = new Intl.DisplayNames(['en'], { type: 'language' });
            const languageNamesZh = new Intl.DisplayNames(['zh-Hant'], { type: 'language' });
            const scriptNamesEn = new Intl.DisplayNames(['en'], { type: 'script' });
            const scriptNamesZh = new Intl.DisplayNames(['zh-Hant'], { type: 'script' });
            const currencyNamesEn = new Intl.DisplayNames(['en'], { type: 'currency' });
            const currencyNamesZh = new Intl.DisplayNames(['zh-Hant'], { type: 'currency' });

            document.getElementById('result').textContent =
              regionNamesEn.of('419') + ':' + regionNamesZh.of('MM') + '|' +
              languageNamesEn.of('fr-CA') + ':' + languageNamesZh.of('fr') + '|' +
              scriptNamesEn.of('Latn') + ':' + scriptNamesZh.of('Kana') + '|' +
              currencyNamesEn.of('TWD') + ':' + currencyNamesZh.of('USD');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Latin America:緬甸|Canadian French:法文|Latin:片假名|New Taiwan Dollar:美元",
    )?;
    Ok(())
}

#[test]
fn intl_display_names_static_and_resolved_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const supported = Intl.DisplayNames.supportedLocalesOf(['zh-Hant', 'en', 'de']);
            const ro = new Intl.DisplayNames(['zh-Hant'], {
              type: 'language',
              style: 'short',
              fallback: 'none',
              languageDisplay: 'standard'
            }).resolvedOptions();
            const tag = Intl.DisplayNames.prototype[Symbol.toStringTag];
            const unknown = new Intl.DisplayNames(['en'], { type: 'region', fallback: 'none' }).of('ZZ');
            document.getElementById('result').textContent =
              supported.join(',') + '|' +
              ro.locale + ':' + ro.style + ':' + ro.fallback + ':' + ro.languageDisplay + '|' +
              tag + '|' + (unknown === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "zh-Hant,en|zh-Hant:short:none:standard|Intl.DisplayNames|true",
    )?;
    Ok(())
}

#[test]
fn intl_display_names_ja_and_he_dictionaries_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const supported = Intl.DisplayNames.supportedLocalesOf(['he', 'ja', 'de']);
            const regionJa = new Intl.DisplayNames(['ja'], { type: 'region' }).of('US');
            const regionHe = new Intl.DisplayNames(['he'], { type: 'region' }).of('US');
            const languageJa = new Intl.DisplayNames(['ja'], { type: 'language' }).of('fr-CA');
            const languageHe = new Intl.DisplayNames(['he'], { type: 'language' }).of('fr-CA');
            const scriptJa = new Intl.DisplayNames(['ja'], { type: 'script' }).of('Latn');
            const scriptHe = new Intl.DisplayNames(['he'], { type: 'script' }).of('Arab');
            const currencyJa = new Intl.DisplayNames(['ja'], { type: 'currency' }).of('USD');
            const currencyHe = new Intl.DisplayNames(['he'], { type: 'currency' }).of('EUR');
            document.getElementById('result').textContent =
              supported.join(',') + '|' +
              regionJa + ':' + regionHe + '|' +
              languageJa + ':' + languageHe + '|' +
              scriptJa + ':' + scriptHe + '|' +
              currencyJa + ':' + currencyHe;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "he,ja|アメリカ合衆国:ארצות הברית|カナダのフランス語:צרפתית קנדית|ラテン文字:ערבי|米ドル:אירו",
        )?;
    Ok(())
}

#[test]
fn intl_display_names_es_and_fr_dictionaries_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const supported = Intl.DisplayNames.supportedLocalesOf(['es', 'fr', 'de']);
            const regionEs = new Intl.DisplayNames(['es'], { type: 'region' }).of('US');
            const regionFr = new Intl.DisplayNames(['fr'], { type: 'region' }).of('US');
            const languageEs = new Intl.DisplayNames(['es'], { type: 'language' }).of('fr-CA');
            const languageFr = new Intl.DisplayNames(['fr'], { type: 'language' }).of('fr-CA');
            const scriptEs = new Intl.DisplayNames(['es'], { type: 'script' }).of('Arab');
            const scriptFr = new Intl.DisplayNames(['fr'], { type: 'script' }).of('Latn');
            const currencyEs = new Intl.DisplayNames(['es'], { type: 'currency' }).of('USD');
            const currencyFr = new Intl.DisplayNames(['fr'], { type: 'currency' }).of('TWD');
            document.getElementById('result').textContent =
              supported.join(',') + '|' +
              regionEs + ':' + regionFr + '|' +
              languageEs + ':' + languageFr + '|' +
              scriptEs + ':' + scriptFr + '|' +
              currencyEs + ':' + currencyFr;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "es,fr|Estados Unidos:États-Unis|francés canadiense:français canadien|árabe:latin|dólar estadounidense:nouveau dollar taïwanais",
        )?;
    Ok(())
}

#[test]
fn intl_collator_compare_returns_sign_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const c = new Intl.Collator();
            const a = c.compare('a', 'c');
            const b = c.compare('c', 'a');
            const d = c.compare('a', 'a');
            document.getElementById('result').textContent =
              (a < 0) + ':' + (b > 0) + ':' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:0")?;
    Ok(())
}

#[test]
fn intl_collator_demo_sort_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const deValues = ['Z', 'a', 'z', 'ä'];
            deValues.sort(new Intl.Collator('de').compare);
            const de = deValues.join(',');
            const svValues = ['Z', 'a', 'z', 'ä'];
            svValues.sort(new Intl.Collator('sv').compare);
            const sv = svValues.join(',');
            const upValues = ['Z', 'a', 'z', 'ä'];
            upValues.sort(new Intl.Collator('de', { caseFirst: 'upper' }).compare);
            const up = upValues.join(',');
            document.getElementById('result').textContent = de + '|' + sv + '|' + up;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a,ä,z,Z|a,z,Z,ä|a,ä,Z,z")?;
    Ok(())
}

#[test]
fn intl_collator_locales_sensitivity_and_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const deCmp = new Intl.Collator('de').compare('ä', 'z');
            const svCmp = new Intl.Collator('sv').compare('ä', 'z');
            const deBase = new Intl.Collator('de', { sensitivity: 'base' }).compare('ä', 'a');
            const svBase = new Intl.Collator('sv', { sensitivity: 'base' }).compare('ä', 'a');
            const supportedLocales = Intl.Collator.supportedLocalesOf(['de', 'sv', 'fr']);
            const supported = supportedLocales.join(',');
            const ro = new Intl.Collator('sv', {
              caseFirst: 'upper',
              sensitivity: 'base'
            }).resolvedOptions();
            const tag = Intl.Collator.prototype[Symbol.toStringTag];
            document.getElementById('result').textContent =
              (deCmp < 0) + ':' + (svCmp > 0) + ':' + deBase + ':' + (svBase > 0) + ':' +
              supported + ':' + ro.locale + ':' + ro.caseFirst + ':' + ro.sensitivity + ':' + tag;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:true:0:true:de,sv:sv:upper:base:Intl.Collator",
    )?;
    Ok(())
}

#[test]
fn bigint_literals_constructor_and_typeof_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const bigA = 9007199254740991n;
            const bigB = BigInt(9007199254740991);
            const bigC = BigInt("0x1fffffffffffff");
            const bigD = BigInt("0o377777777777777777");
            const bigE = BigInt("0b11111111111111111111111111111111111111111111111111111");
            const typeA = typeof bigA;
            const typeB = typeof bigB;
            const falsyBranch = 0n ? 't' : 'f';
            const truthyBranch = 12n ? 't' : 'f';
            const concat = 'x' + 1n;
            document.getElementById('result').textContent =
              bigA + ':' + bigB + ':' + bigC + ':' + bigD + ':' + bigE + ':' +
              typeA + ':' + typeB + ':' + falsyBranch + ':' + truthyBranch + ':' + concat;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "9007199254740991:9007199254740991:9007199254740991:9007199254740991:9007199254740991:bigint:bigint:f:t:x1",
        )?;
    Ok(())
}

#[test]
fn bigint_static_and_instance_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = BigInt.asIntN(8, 257n);
            const b = BigInt.asIntN(8, 255n);
            const c = BigInt.asUintN(8, -1n);
            const d = window.BigInt.asUintN(8, 257n);
            const e = (255n).toString(16);
            const f = (255n).toString();
            const g = (255n).toLocaleString();
            const h = (255n).valueOf() === 255n;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:-1:255:1:ff:255:255:true")?;
    Ok(())
}

#[test]
fn bigint_arithmetic_and_bitwise_operations_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const previousMaxSafe = BigInt(Number.MAX_SAFE_INTEGER);
            const maxPlusTwo = previousMaxSafe + 2n;
            const prod = previousMaxSafe * 2n;
            const diff = prod - 10n;
            const mod = prod % 10n;
            const pow = 2n ** 54n;
            const neg = pow * -1n;
            const div1 = 4n / 2n;
            const div2 = 5n / 2n;
            const bitAnd = 6n & 3n;
            const bitOr = 6n | 3n;
            const bitXor = 6n ^ 3n;
            const shl = 8n << 1n;
            const shr = 8n >> 1n;
            const shlNeg = 8n << -1n;
            document.getElementById('result').textContent =
              maxPlusTwo + ':' + diff + ':' + mod + ':' + pow + ':' + neg + ':' +
              div1 + ':' + div2 + ':' + bitAnd + ':' + bitOr + ':' + bitXor + ':' +
              shl + ':' + shr + ':' + shlNeg;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "9007199254740993:18014398509481972:2:18014398509481984:-18014398509481984:2:2:2:7:5:16:4:4",
        )?;
    Ok(())
}

#[test]
fn bigint_comparisons_and_increment_decrement_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 1n;
            ++n;
            n++;
            --n;
            n--;
            const a = 0n == 0;
            const b = 0n === 0;
            const c = 1n < 2;
            const d = 2n > 1;
            const e = 2n > 2;
            const f = 2n >= 2;
            document.getElementById('result').textContent =
              n + ':' + a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' +
              (typeof n) + ':' + (0n ? 't' : 'f') + ':' + (12n ? 't' : 'f');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:true:false:true:true:false:true:bigint:f:t")?;
    Ok(())
}

#[test]
fn bigint_mixed_type_and_unsupported_operations_report_errors() -> Result<()> {
    let html = r#"
        <button id='mix'>mix</button>
        <button id='ushift'>ushift</button>
        <button id='unary'>unary</button>
        <script>
          document.getElementById('mix').addEventListener('click', () => {
            const v = 1n + 1;
          });
          document.getElementById('ushift').addEventListener('click', () => {
            const v = 1n >>> 0n;
          });
          document.getElementById('unary').addEventListener('click', () => {
            const v = +1n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let mix_err = h
        .click("#mix")
        .expect_err("mixed BigInt/Number arithmetic should fail");
    match mix_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("cannot mix BigInt and other types in addition"))
        }
        other => panic!("unexpected mixed operation error: {other:?}"),
    }

    let us_err = h
        .click("#ushift")
        .expect_err("unsigned right shift for BigInt should fail");
    match us_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("BigInt values do not support unsigned right shift"))
        }
        other => panic!("unexpected unsigned shift error: {other:?}"),
    }

    let unary_err = h
        .click("#unary")
        .expect_err("unary plus for BigInt should fail");
    match unary_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("unary plus is not supported for BigInt values"))
        }
        other => panic!("unexpected unary plus error: {other:?}"),
    }

    Ok(())
}

#[test]
fn bigint_constructor_and_json_stringify_errors_are_reported() -> Result<()> {
    let html = r#"
        <button id='ctor'>ctor</button>
        <button id='newctor'>newctor</button>
        <button id='json'>json</button>
        <script>
          document.getElementById('ctor').addEventListener('click', () => {
            BigInt('1.5');
          });
          document.getElementById('newctor').addEventListener('click', () => {
            new BigInt(1);
          });
          document.getElementById('json').addEventListener('click', () => {
            JSON.stringify({ a: 1n });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let ctor_err = h
        .click("#ctor")
        .expect_err("BigInt constructor should reject decimal string");
    match ctor_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("cannot convert 1.5 to a BigInt")),
        other => panic!("unexpected BigInt conversion error: {other:?}"),
    }

    let new_ctor_err = h.click("#newctor").expect_err("new BigInt should fail");
    match new_ctor_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("BigInt is not a constructor")),
        other => panic!("unexpected new BigInt error: {other:?}"),
    }

    let json_err = h
        .click("#json")
        .expect_err("JSON.stringify with BigInt should fail");
    match json_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("JSON.stringify does not support BigInt values"))
        }
        other => panic!("unexpected JSON.stringify BigInt error: {other:?}"),
    }

    Ok(())
}

#[test]
fn decimal_numeric_literals_work_in_comparisons_and_assignment() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 0.5;
            const b = 1.0;
            if (a < b && a === 0.5 && b >= 1)
              document.getElementById('result').textContent = a;
            else
              document.getElementById('result').textContent = 'ng';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0.5")?;
    Ok(())
}

#[test]
fn multiplication_and_division_work_for_numbers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 6 * 7;
            const b = 5 / 2;
            document.getElementById('result').textContent = a + ':' + b;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "42:2.5")?;
    Ok(())
}

#[test]
fn subtraction_and_unary_minus_work_for_numbers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 10 - 3;
            const b = -2;
            const c = 1 - -2;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "7:-2:3")?;
    Ok(())
}

#[test]
fn addition_supports_numeric_and_string_left_fold() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 1 + 2;
            const b = 1 + 2 + 'x';
            const c = 1 + '2' + 3;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:3x:123")?;
    Ok(())
}

#[test]
fn timer_delay_accepts_arithmetic_expression() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 5 * 2);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(9)?;
    h.assert_text("#result", "")?;
    h.advance_time(1)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn timer_delay_accepts_addition_expression() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 5 + 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(10)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn timer_delay_accepts_subtraction_expression() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 15 - 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(10)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn timer_arrow_expression_callback_executes() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(
              () => setTimeout(() => {
                document.getElementById('result').textContent = 'ok';
              }, 0),
              5
            );
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn math_random_seed_reset_repeats_sequence() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.random() + ':' + Math.random();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_random_seed(7);
    h.click("#btn")?;
    let first = h.dump_dom("#result")?;

    h.set_random_seed(7);
    h.click("#btn")?;
    let second = h.dump_dom("#result")?;

    assert_eq!(first, second);
    Ok(())
}

#[test]
fn clear_timeout_cancels_task_and_set_timeout_returns_ids() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const first = setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            const second = setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
            clearTimeout(first);
            result.textContent = first + ':' + second + ':';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:2:")?;
    h.flush()?;
    h.assert_text("#result", "1:2:B")?;
    Ok(())
}

#[test]
fn clear_timeout_unknown_id_is_ignored() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            clearTimeout(999);
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.flush()?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn script_extractor_ignores_script_like_strings() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const marker = "</script>";
          const htmlLike = "<script>not real</script>";
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = marker + '|' + htmlLike;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "</script>|<script>not real</script>")?;
    Ok(())
}

#[test]
fn script_extractor_handles_regex_literals_with_quotes_for_end_tag_scan() -> Result<()> {
    let html = r##"
        <script>
          const sanitizer = /["]/g;
        </script>
        <p id="result"></p>
        "##;

    let parsed = parse_html(html)?;
    assert_eq!(parsed.scripts.len(), 1);
    assert!(parsed.scripts[0].contains(r#"/["]/g"#));
    Ok(())
}

#[test]
fn script_extractor_falls_back_to_raw_end_tag_scan_when_js_lexer_gets_stuck() -> Result<()> {
    let html = r###"
        <script>
          const map = {
            '"': """,
            "'": "'",
          };
          return String(value ?? "").replace(/[&<>"']/g, (ch) => map[ch] || ch);
        </script>
        <p id="result"></p>
        "###;

    let parsed = parse_html(html)?;
    assert_eq!(parsed.scripts.len(), 1);
    assert!(parsed.scripts[0].contains(r#"replace(/[&<>"']/g"#));
    Ok(())
}

#[test]
fn script_extractor_handles_nested_template_literals_for_end_tag_scan() -> Result<()> {
    let html = r##"
        <script>
          const markup = `${true ? `<span class="chip">${"ok"}</span>` : ""}`;
        </script>
        <p id="result"></p>
        "##;

    let parsed = parse_html(html)?;
    assert_eq!(parsed.scripts.len(), 1);
    assert!(parsed.scripts[0].contains("class=\"chip\""));
    Ok(())
}

#[test]
fn optional_chaining_member_get_after_call_and_null_target_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const plans = [{ key: "none", selectable: false }, { key: "up", selectable: true }];
          document.getElementById('btn').addEventListener('click', () => {
            const selected = plans.find((plan) => plan.selectable)?.key || "";
            const fromNull = null?.key || "missing";
            document.getElementById('result').textContent = selected + ":" + fromNull;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "up:missing")?;
    Ok(())
}

#[test]
fn textarea_select_member_call_is_not_treated_as_intl_plural_rules_select() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const tmp = document.createElement('textarea');
            tmp.value = 'x';
            tmp.select();
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn member_get_after_index_on_plain_object_chain_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const state = { rows: [{ selectedPlan: 'round_up' }] };
            const index = 0;
            const staticValue = state.rows[0].selectedPlan;
            const dynamicValue = state.rows[index].selectedPlan;
            document.getElementById('result').textContent = staticValue + ':' + dynamicValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "round_up:round_up")?;
    Ok(())
}

#[test]
fn doctype_declaration_is_ignored_during_html_parse() -> Result<()> {
    let html = r#"
        <!DOCTYPE html>
        <p id="result">ok</p>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn set_interval_repeats_and_clear_interval_stops_requeue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'I';
              if (result.textContent === '1:III') clearInterval(id);
            }, 0);
            result.textContent = id + ':';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:")?;
    h.flush()?;
    h.assert_text("#result", "1:III")?;
    h.flush()?;
    h.assert_text("#result", "1:III")?;
    Ok(())
}

#[test]
fn clear_timeout_can_cancel_interval_id() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'X';
            }, 0);
            clearTimeout(id);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn flush_step_limit_error_contains_timer_diagnostics() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    let err = h
        .flush()
        .expect_err("flush should fail on uncleared interval");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("flush exceeded max task steps"));
            assert!(msg.contains("limit=10000"));
            assert!(msg.contains("pending_tasks="));
            assert!(msg.contains("next_task=id=1"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn timer_step_limit_can_be_configured() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_timer_step_limit(3)?;
    h.click("#btn")?;
    let err = h
        .flush()
        .expect_err("flush should fail with configured small step limit");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("limit=3"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn timer_step_limit_rejects_zero() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    let err = h
        .set_timer_step_limit(0)
        .expect_err("zero step limit should be rejected");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("set_timer_step_limit requires at least 1 step"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn advance_time_step_limit_error_contains_due_limit() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_timer_step_limit(2)?;
    h.click("#btn")?;
    let err = h
        .advance_time(7)
        .expect_err("advance_time should fail with configured small step limit");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("limit=2"));
            assert!(msg.contains("now_ms=7"));
            assert!(msg.contains("due_limit=7"));
            assert!(msg.contains("next_task=id=1"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn assertion_failure_contains_dom_snippet() -> Result<()> {
    let html = r#"
        <p id='result'>NG</p>
        "#;
    let h = Harness::from_html(html)?;

    let err = match h.assert_text("#result", "OK") {
        Ok(()) => panic!("assert_text should fail"),
        Err(err) => err,
    };

    match err {
        Error::AssertionFailed {
            selector,
            expected,
            actual,
            dom_snippet,
        } => {
            assert_eq!(selector, "#result");
            assert_eq!(expected, "OK");
            assert_eq!(actual, "NG");
            assert!(dom_snippet.contains("<p"));
            assert!(dom_snippet.contains("NG"));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn remove_and_has_attribute_work() -> Result<()> {
    let html = r#"
        <div id='box' data-x='1' class='a'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            const before = box.hasAttribute('data-x');
            box.removeAttribute('data-x');
            const after = box.hasAttribute('data-x');
            box.removeAttribute('class');
            document.getElementById('result').textContent =
              before + ':' + after + ':' + box.className + ':' + box.getAttribute('data-x');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false::")?;
    Ok(())
}

#[test]
fn remove_id_attribute_updates_id_selector_index() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.removeAttribute('id');
            document.getElementById('result').textContent =
              document.querySelectorAll('#box').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0")?;
    Ok(())
}

#[test]
fn create_element_append_and_remove_child_work() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const node = document.createElement('span');
            node.id = 'tmp';
            node.textContent = 'X';

            document.getElementById('result').textContent =
              document.querySelectorAll('#tmp').length + ':';
            root.appendChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              document.querySelectorAll('#tmp').length + ':' +
              document.querySelector('#root>#tmp').textContent;
            root.removeChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' +
              document.querySelectorAll('#tmp').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:1:X:0")?;
    Ok(())
}

#[test]
fn insert_before_inserts_new_node_before_reference() -> Result<()> {
    let html = r#"
        <div id='root'><span id='a'>A</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.createElement('span');
            b.id = 'b';
            b.textContent = 'B';
            root.insertBefore(b, document.getElementById('c'));
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#b').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC:B")?;
    Ok(())
}

#[test]
fn insert_before_reorders_existing_child() -> Result<()> {
    let html = r#"
        <div id='root'><span id='a'>A</span><span id='b'>B</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            root.insertBefore(
              document.getElementById('c'),
              document.getElementById('a')
            );
            document.getElementById('result').textContent = root.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "CAB")?;
    Ok(())
}

#[test]
fn append_alias_adds_child_to_end() -> Result<()> {
    let html = r#"
        <div id='root'><span>A</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.createElement('span');
            b.id = 'b';
            b.textContent = 'B';
            root.append(b);
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#b').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB:B")?;
    Ok(())
}

#[test]
fn prepend_adds_child_to_start() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            root.prepend(a);
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#a').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC:A")?;
    Ok(())
}

#[test]
fn before_and_after_insert_relative_to_target() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            const c = document.createElement('span');
            c.id = 'c';
            c.textContent = 'C';
            b.before(a);
            b.after(c);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelector('#root>#a').textContent + ':' +
              document.querySelector('#root>#c').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC:A:C")?;
    Ok(())
}

#[test]
fn replace_with_replaces_node_and_updates_id_index() -> Result<()> {
    let html = r#"
        <div id='root'><span id='old'>O</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const old = document.getElementById('old');
            const neo = document.createElement('span');
            neo.id = 'new';
            neo.textContent = 'N';
            old.replaceWith(neo);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#old').length + ':' +
              document.querySelectorAll('#new').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "N:0:1")?;
    Ok(())
}

#[test]
fn insert_adjacent_element_positions_work() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            const c = document.createElement('span');
            c.id = 'c';
            c.textContent = 'C';
            const d = document.createElement('span');
            d.id = 'd';
            d.textContent = 'D';
            const e = document.createElement('span');
            e.id = 'e';
            e.textContent = 'E';
            b.insertAdjacentElement('beforebegin', a);
            b.insertAdjacentElement('afterbegin', d);
            b.insertAdjacentElement('beforeend', e);
            b.insertAdjacentElement('afterend', c);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#a').length + ':' +
              document.querySelectorAll('#c').length + ':' +
              document.querySelector('#b>#d').textContent + ':' +
              document.querySelector('#b>#e').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ADBEC:1:1:D:E")?;
    Ok(())
}

#[test]
fn insert_adjacent_text_positions_and_expression_work() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <input id='v' value='Y'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            b.insertAdjacentText('beforebegin', 'A');
            b.insertAdjacentText('afterbegin', 'X');
            b.insertAdjacentText('beforeend', document.getElementById('v').value);
            b.insertAdjacentText('afterend', 'C');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' + b.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AXBYC:XBY")?;
    Ok(())
}

#[test]
fn insert_adjacent_html_positions_and_order_work() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            b.insertAdjacentHTML('beforebegin', '<i id="y1">Y</i><i id="y2">Z</i>');
            b.insertAdjacentHTML('afterbegin', 'X<span id="x1">X</span>');
            b.insertAdjacentHTML('beforeend', '<span id="x2">W</span><span id="x3">Q</span>');
            b.insertAdjacentHTML('afterend', 'T<em id="t">T</em>');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#y1').length + ':' +
              document.querySelectorAll('#y2').length + ':' +
              document.querySelectorAll('#x1').length + ':' +
              document.querySelectorAll('#x2').length + ':' +
              document.querySelectorAll('#x3').length + ':' +
              document.querySelectorAll('#t').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "YZXXBWQTT:1:1:1:1:1:1")?;
    Ok(())
}

#[test]
fn insert_adjacent_html_position_expression_works() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            let head = 'beforebegin';
            let inner = 'afterbegin';
            let tail = 'AFTEREND';
            b.insertAdjacentHTML(head, '<i id="head">H</i>');
            b.insertAdjacentHTML(inner, '<i id="mid">M</i>');
            b.insertAdjacentHTML(tail, '<i id="tail">T</i>');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#head').length + ':' +
              document.querySelectorAll('#mid').length + ':' +
              document.querySelectorAll('#tail').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "HMBT:1:1:1")?;
    Ok(())
}
