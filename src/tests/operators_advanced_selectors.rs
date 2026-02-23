use super::*;

#[test]
fn selector_parse_supports_active() {
    let active = parse_selector_step("button:active").expect("parse should succeed");
    assert_eq!(active.pseudo_classes, vec![SelectorPseudoClass::Active]);
}

#[test]
fn selector_parse_supports_not() {
    let by_id = parse_selector_step("span:not(#x)").expect("parse should succeed");
    let by_class = parse_selector_step("span:not(.x)").expect("parse should succeed");
    let nested = parse_selector_step("span:not(:not(.x))").expect("parse should succeed");
    let with_attribute = parse_selector_step("li:not([data='a,b'])").expect("parse should succeed");
    if let SelectorPseudoClass::Not(inners) = &by_id.pseudo_classes[0] {
        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 1);
        assert_eq!(inners[0][0].step.id.as_deref(), Some("x"));
    } else {
        panic!("expected not pseudo");
    }
    if let SelectorPseudoClass::Not(inners) = &by_class.pseudo_classes[0] {
        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 1);
        assert_eq!(inners[0][0].step.tag.as_deref(), None);
        assert_eq!(inners[0][0].step.classes.as_slice(), &["x"]);
    } else {
        panic!("expected not pseudo");
    }
    if let SelectorPseudoClass::Not(inners) = &nested.pseudo_classes[0] {
        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 1);
        if let SelectorPseudoClass::Not(inner_inners) = &inners[0][0].step.pseudo_classes[0] {
            assert_eq!(inner_inners.len(), 1);
            assert_eq!(inner_inners[0][0].step.tag.as_deref(), None);
            assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["x"]);
            assert!(inner_inners[0][0].step.pseudo_classes.is_empty());
        } else {
            panic!("expected nested not pseudo");
        }
    } else {
        panic!("expected not pseudo");
    }
    if let SelectorPseudoClass::Not(inners) = &with_attribute.pseudo_classes[0] {
        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 1);
        let inner = &inners[0][0].step;
        assert_eq!(
            inner.attrs,
            vec![SelectorAttrCondition::Eq {
                key: "data".into(),
                value: "a,b".into()
            }]
        );
        assert!(inner.classes.is_empty());
        assert!(inner.id.is_none());
        assert!(inner.pseudo_classes.is_empty());
        assert!(!inner.universal);
    } else {
        panic!("expected not pseudo");
    }
}

#[test]
fn selector_parse_supports_where_is_and_has() {
    let where_step =
        parse_selector_step("span:where(.a, #b, :not(.skip))").expect("parse should succeed");
    let is_step =
        parse_selector_step("span:is(.a, #b, :not(.skip))").expect("parse should succeed");
    let has_step = parse_selector_step("section:has(.c, #d)").expect("parse should succeed");

    assert!(matches!(
        where_step.pseudo_classes[0],
        SelectorPseudoClass::Where(_)
    ));
    if let SelectorPseudoClass::Where(inners) = &where_step.pseudo_classes[0] {
        assert_eq!(inners.len(), 3);
        assert_eq!(inners[0].len(), 1);
        assert_eq!(inners[1].len(), 1);
        assert_eq!(inners[2].len(), 1);
    }

    assert!(matches!(
        is_step.pseudo_classes[0],
        SelectorPseudoClass::Is(_)
    ));
    assert!(matches!(
        has_step.pseudo_classes[0],
        SelectorPseudoClass::Has(_)
    ));
}

#[test]
fn selector_parse_supports_attribute_operators() {
    let exists = parse_selector_step("[flag]").expect("parse should succeed");
    let eq = parse_selector_step("[data='value']").expect("parse should succeed");
    let starts_with = parse_selector_step("[data^='pre']").expect("parse should succeed");
    let ends_with = parse_selector_step("[data$='post']").expect("parse should succeed");
    let contains = parse_selector_step("[data*='med']").expect("parse should succeed");
    let includes = parse_selector_step("[tags~='one']").expect("parse should succeed");
    let dash = parse_selector_step("[lang|='en']").expect("parse should succeed");

    assert_eq!(
        exists.attrs,
        vec![SelectorAttrCondition::Exists { key: "flag".into() }]
    );
    assert_eq!(
        eq.attrs,
        vec![SelectorAttrCondition::Eq {
            key: "data".into(),
            value: "value".into()
        }]
    );
    assert_eq!(
        starts_with.attrs,
        vec![SelectorAttrCondition::StartsWith {
            key: "data".into(),
            value: "pre".into()
        }]
    );
    assert_eq!(
        ends_with.attrs,
        vec![SelectorAttrCondition::EndsWith {
            key: "data".into(),
            value: "post".into()
        }]
    );
    assert_eq!(
        contains.attrs,
        vec![SelectorAttrCondition::Contains {
            key: "data".into(),
            value: "med".into()
        }]
    );
    assert_eq!(
        includes.attrs,
        vec![SelectorAttrCondition::Includes {
            key: "tags".into(),
            value: "one".into()
        }]
    );
    assert_eq!(
        dash.attrs,
        vec![SelectorAttrCondition::DashMatch {
            key: "lang".into(),
            value: "en".into()
        }]
    );
    let empty = parse_selector_step("[data='']").expect("parse should succeed");
    let case_key = parse_selector_step("[DATA='v']").expect("parse should succeed");
    let unquoted_empty = parse_selector_step("[data=]").expect("parse should succeed");
    assert_eq!(
        empty.attrs,
        vec![SelectorAttrCondition::Eq {
            key: "data".into(),
            value: "".into()
        }]
    );
    assert_eq!(
        case_key.attrs,
        vec![SelectorAttrCondition::Eq {
            key: "data".into(),
            value: "v".into()
        }]
    );
    assert_eq!(
        unquoted_empty.attrs,
        vec![SelectorAttrCondition::Eq {
            key: "data".into(),
            value: "".into()
        }]
    );
}

#[test]
fn selector_parse_supports_not_with_multiple_selectors() {
    let multi =
        parse_selector_step("li:not(.a, #target, :not(.skip))").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &multi.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };
    assert_eq!(inners.len(), 3);
    assert_eq!(inners[0].len(), 1);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["a"]);

    assert_eq!(inners[1].len(), 1);
    assert_eq!(inners[1][0].step.id.as_deref(), Some("target"));

    assert_eq!(inners[2].len(), 1);
    assert_eq!(inners[2][0].step.pseudo_classes.len(), 1);
    let inner = &inners[2][0].step.pseudo_classes[0];
    assert!(matches!(inner, SelectorPseudoClass::Not(_)));
}

#[test]
fn selector_parse_supports_not_with_multiple_not_pseudos() {
    let parsed =
        parse_selector_step("li:not(:not(.foo), :not(.bar))").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 2);

    assert_eq!(inners[0].len(), 1);
    assert_eq!(inners[0][0].step.pseudo_classes.len(), 1);
    let first = &inners[0][0].step.pseudo_classes[0];
    if let SelectorPseudoClass::Not(inner_inners) = first {
        assert_eq!(inner_inners.len(), 1);
        assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["foo"]);
    } else {
        panic!("expected nested not pseudo in first arg");
    }

    assert_eq!(inners[1].len(), 1);
    assert_eq!(inners[1][0].step.pseudo_classes.len(), 1);
    let second = &inners[1][0].step.pseudo_classes[0];
    if let SelectorPseudoClass::Not(inner_inners) = second {
        assert_eq!(inner_inners.len(), 1);
        assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["bar"]);
    } else {
        panic!("expected nested not pseudo in second arg");
    }
}

#[test]
fn selector_parse_supports_not_with_complex_selector_list() {
    let parsed = parse_selector_step("span:not(.scope *, #skip-me, .area :not(.nested .leaf))")
        .expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 3);

    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert!(inners[0][0].combinator.is_none());
    assert_eq!(inners[0][1].step.tag.as_deref(), None);
    assert!(inners[0][1].step.universal);
    assert_eq!(
        inners[0][1].combinator,
        Some(SelectorCombinator::Descendant)
    );

    assert_eq!(inners[1].len(), 1);
    assert_eq!(inners[1][0].step.id.as_deref(), Some("skip-me"));
    assert!(inners[1][0].combinator.is_none());

    assert_eq!(inners[2].len(), 2);
    assert_eq!(inners[2][0].step.classes.as_slice(), &["area"]);
    assert_eq!(inners[2][1].step.pseudo_classes.len(), 1);
    let nested = &inners[2][1].step.pseudo_classes[0];
    if let SelectorPseudoClass::Not(nested_inners) = nested {
        assert_eq!(nested_inners.len(), 1);
        assert_eq!(nested_inners[0].len(), 2);
        assert_eq!(nested_inners[0][0].step.classes.as_slice(), &["nested"]);
        assert_eq!(nested_inners[0][1].step.classes.as_slice(), &["leaf"]);
        assert_eq!(
            nested_inners[0][1].combinator,
            Some(SelectorCombinator::Descendant)
        );
    } else {
        panic!("expected nested not pseudo");
    }
}

#[test]
fn selector_parse_supports_not_with_adjacent_selector() {
    let parsed = parse_selector_step("span:not(.scope + span)").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 1);
    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
    assert_eq!(
        inners[0][1].combinator,
        Some(SelectorCombinator::AdjacentSibling)
    );
}

#[test]
fn selector_parse_supports_not_with_selector_list_general_sibling_selector() {
    let parsed =
        parse_selector_step("span:not(.scope ~ span, #excluded-id)").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 2);
    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
    assert_eq!(
        inners[0][1].combinator,
        Some(SelectorCombinator::GeneralSibling)
    );

    assert_eq!(inners[1].len(), 1);
    assert_eq!(inners[1][0].step.id.as_deref(), Some("excluded-id"));
    assert!(inners[1][0].combinator.is_none());
}

#[test]
fn selector_parse_supports_not_with_general_sibling_selector() {
    let parsed = parse_selector_step("span:not(.scope ~ span)").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 1);
    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
    assert_eq!(
        inners[0][1].combinator,
        Some(SelectorCombinator::GeneralSibling)
    );
}

#[test]
fn selector_parse_supports_not_with_child_selector() {
    let parsed = parse_selector_step("span:not(.scope > span)").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 1);
    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
    assert_eq!(inners[0][1].combinator, Some(SelectorCombinator::Child));
}

#[test]
fn selector_parse_rejects_invalid_not_argument_forms() {
    assert!(parse_selector_step("span:not()").is_err());
    assert!(parse_selector_step("span:not(,)").is_err());
    assert!(parse_selector_step("span:not(.a,,#b)").is_err());
    assert!(parse_selector_step("span:not(.a,").is_err());
    assert!(parse_selector_step("span:not(.a,#b,)").is_err());
}

#[test]
fn selector_parse_rejects_unclosed_not_parenthesis() {
    assert!(parse_selector_step("span:not(.a, #b").is_err());
    assert!(parse_selector_step("span:not(:not(.a)").is_err());
}

#[test]
fn selector_runtime_rejects_invalid_not_selector() -> Result<()> {
    let html = "<div id='root'></div>";
    let h = Harness::from_html(html)?;

    let err = h
        .assert_exists("span:not()")
        .expect_err("invalid selector should be rejected");
    match err {
        Error::UnsupportedSelector(selector) => assert_eq!(selector, "span:not()"),
        other => panic!("expected unsupported selector error, got: {other:?}"),
    }

    let err = h
        .assert_exists("span:not(.a,)")
        .expect_err("invalid selector should be rejected");
    match err {
        Error::UnsupportedSelector(selector) => assert_eq!(selector, "span:not(.a,)"),
        other => panic!("expected unsupported selector error, got: {other:?}"),
    }

    Ok(())
}

#[test]
fn selector_parse_supports_nth_of_type() {
    let odd = parse_selector_step("li:nth-of-type(odd)").expect("parse should succeed");
    let expr = parse_selector_step("li:nth-of-type(2n)").expect("parse should succeed");
    let n = parse_selector_step("li:nth-of-type(n)").expect("parse should succeed");
    let exact = parse_selector_step("li:nth-of-type(3)").expect("parse should succeed");
    assert_eq!(
        odd.pseudo_classes,
        vec![SelectorPseudoClass::NthOfType(NthChildSelector::Odd)]
    );
    assert_eq!(
        expr.pseudo_classes,
        vec![SelectorPseudoClass::NthOfType(NthChildSelector::AnPlusB(
            2, 0
        ))]
    );
    assert_eq!(
        n.pseudo_classes,
        vec![SelectorPseudoClass::NthOfType(NthChildSelector::AnPlusB(
            1, 0
        ))]
    );
    assert_eq!(
        exact.pseudo_classes,
        vec![SelectorPseudoClass::NthOfType(NthChildSelector::Exact(3))]
    );
}

#[test]
fn selector_parse_supports_nth_last_of_type() {
    let odd = parse_selector_step("li:nth-last-of-type(odd)").expect("parse should succeed");
    let even = parse_selector_step("li:nth-last-of-type(even)").expect("parse should succeed");
    let n = parse_selector_step("li:nth-last-of-type(n)").expect("parse should succeed");
    let exact = parse_selector_step("li:nth-last-of-type(2)").expect("parse should succeed");
    assert_eq!(
        odd.pseudo_classes,
        vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Odd)]
    );
    assert_eq!(
        even.pseudo_classes,
        vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Even)]
    );
    assert_eq!(
        n.pseudo_classes,
        vec![SelectorPseudoClass::NthLastOfType(
            NthChildSelector::AnPlusB(1, 0)
        )]
    );
    assert_eq!(
        exact.pseudo_classes,
        vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Exact(
            2
        ))]
    );
}

#[test]
fn selector_nth_last_child_odd_even_work() -> Result<()> {
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
            const odd = document.querySelector('li:nth-last-child(odd)').id;
            const even = document.querySelector('li:nth-last-child(even)').id;
            const second_last = document.querySelector('li:nth-last-child(2)').id;
            const total = document.querySelectorAll('li:nth-last-child(odd)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + second_last + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "two:one:five:3")?;
    Ok(())
}

#[test]
fn radio_group_exclusive_selection_works() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='r1' type='radio' name='plan'>
          <input id='r2' type='radio' name='plan'>
        </form>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#r1")?;
    h.assert_checked("#r1", true)?;
    h.assert_checked("#r2", false)?;
    h.click("#r2")?;
    h.assert_checked("#r1", false)?;
    h.assert_checked("#r2", true)?;
    Ok(())
}

#[test]
fn radio_checked_property_assignment_preserves_group_exclusivity() -> Result<()> {
    let html = r#"
        <form id='f1'>
          <input id='r1' type='radio' name='plan'>
          <input id='r2' type='radio' name='plan'>
        </form>
        <form id='f2'>
          <input id='r3' type='radio' name='plan'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('r1').checked = true;
            document.getElementById('r3').checked = true;
            document.getElementById('r2').checked = true;
            document.getElementById('result').textContent =
              document.getElementById('r1').checked + ':' +
              document.getElementById('r2').checked + ':' +
              document.getElementById('r3').checked;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true:true")?;
    Ok(())
}

#[test]
fn radio_group_defaults_are_normalized_on_parse_and_form_reset() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='r1' type='radio' name='plan' checked>
          <input id='r2' type='radio' name='plan' checked>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('r1').checked = true;
            document.getElementById('f').reset();
            document.getElementById('result').textContent =
              document.getElementById('r1').checked + ':' +
              document.getElementById('r2').checked;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.assert_checked("#r1", false)?;
    h.assert_checked("#r2", true)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true")?;
    Ok(())
}

#[test]
fn disabled_controls_ignore_user_actions() -> Result<()> {
    let html = r#"
        <input id='name' disabled value='init'>
        <input id='agree' type='checkbox' disabled checked>
        <p id='result'></p>
        <script>
          document.getElementById('name').addEventListener('input', () => {
            document.getElementById('result').textContent = 'name-input';
          });
          document.getElementById('agree').addEventListener('change', () => {
            document.getElementById('result').textContent = 'agree-change';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "next")?;
    h.assert_value("#name", "init")?;
    h.assert_text("#result", "")?;

    h.click("#agree")?;
    h.assert_checked("#agree", true)?;
    h.assert_text("#result", "")?;

    h.set_checked("#agree", false)?;
    h.assert_checked("#agree", true)?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn disabled_property_prevents_user_actions_and_can_be_cleared() -> Result<()> {
    let html = r#"
        <input id='name' value='init'>
        <input id='agree' type='checkbox' checked>
        <button id='disable'>disable</button>
        <button id='enable'>enable</button>
        <p id='result'></p>
        <script>
          document.getElementById('disable').addEventListener('click', () => {
            document.getElementById('name').disabled = true;
            document.getElementById('agree').disabled = true;
          });
          document.getElementById('enable').addEventListener('click', () => {
            document.getElementById('name').disabled = false;
            document.getElementById('agree').disabled = false;
          });
          document.getElementById('name').addEventListener('input', () => {
            document.getElementById('result').textContent = 'name-input';
          });
          document.getElementById('agree').addEventListener('change', () => {
            document.getElementById('result').textContent = 'agree-change';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#disable")?;

    h.type_text("#name", "next")?;
    h.assert_value("#name", "init")?;
    h.click("#agree")?;
    h.assert_checked("#agree", true)?;
    h.assert_text("#result", "")?;

    h.click("#enable")?;
    h.type_text("#name", "next")?;
    h.set_checked("#agree", false)?;
    h.assert_value("#name", "next")?;
    h.assert_checked("#agree", false)?;
    Ok(())
}

#[test]
fn assignment_and_remainder_expressions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 20;
            n += 5;
            n -= 3;
            n *= 2;
            n /= 4;
            n %= 6;
            const eq = (10 % 3) == 1;
            const neq = (10 % 3) != 2;
            document.getElementById('result').textContent =
              n + ':' + (eq ? 'eq' : 'neq') + ':' + (neq ? 'neq' : 'eq');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5:eq:neq")?;
    Ok(())
}

#[test]
fn loose_equality_and_inequality_follow_js_coercion_rules() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 0 == false;
            const b = 1 == true;
            const c = '' == 0;
            const d = ' \t ' == 0;
            const e = '1' == 1;
            const f = null == undefined;
            const g = null == 0;
            const h = undefined == 0;
            const i = [1] == 1;
            const j = [] == '';
            const k = ({ a: 1 }) == '[object Object]';
            const l = '1' != 1;
            const m = '2' != 1;
            const n = 0 === false;
            const o = 0 !== false;
            const p = NaN == NaN;
            const q = NaN != NaN;
            const arr = [1];
            const r = arr == arr;
            const s = arr != arr;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' + h + ':' +
              i + ':' + j + ':' + k + ':' + l + ':' + m + ':' + n + ':' + o + ':' + p + ':' +
              q + ':' + r + ':' + s;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "true:true:true:true:true:true:false:false:true:true:true:false:true:false:true:false:true:true:false",
        )?;
    Ok(())
}

#[test]
fn unary_plus_works_as_numeric_expression() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = '12';
            const value = +text;
            const direct = +'-3.5';
            const paren = +('+7');
            document.getElementById('result').textContent =
              value + ':' + direct + ':' + paren;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12:-3.5:7")?;
    Ok(())
}

#[test]
fn bitwise_expression_supports_binary_operations() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const bit_and = 5 & 3;
            const bit_or = 5 | 2;
            const bit_xor = 5 ^ 1;
            const left = 1 + 2 << 2;
            const masked = 5 + 2 & 4;
            const shift = 8 >>> 1;
            const signed_shift = -8 >> 1;
            const unsigned_shift = (-1) >>> 1;
            const inv = ~1;
            document.getElementById('result').textContent =
              bit_and + ':' + bit_or + ':' + bit_xor + ':' + left + ':' + masked + ':' +
              shift + ':' + signed_shift + ':' + unsigned_shift + ':' + inv;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:7:4:12:4:4:-4:2147483647:-2")?;
    Ok(())
}

#[test]
fn bitwise_compound_assignment_operators_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 6;
            n &= 3;
            n |= 4;
            n ^= 1;
            n <<= 1;
            n >>= 1;
            n >>>= 1;
            document.getElementById('result').textContent = n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn exponentiation_expression_and_compound_assignment_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const value = 2 ** 3 ** 2;
            const with_mul = 2 * 3 ** 2;
            const grouped = (2 + 2) ** 3;
            let n = 2;
            n **= 3;
            document.getElementById('result').textContent =
              value + ':' + with_mul + ':' + grouped + ':' + n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "512:18:64:8")?;
    Ok(())
}

#[test]
fn update_statements_change_identifier_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 1;
            ++n;
            n++;
            --n;
            n--;
            document.getElementById('result').textContent = n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn typeof_operator_works_for_known_and_undefined_identifiers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const known = 1;
            const a = typeof known;
            const b = typeof unknownName;
            const c = typeof false;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "number:undefined:boolean")?;
    Ok(())
}

#[test]
fn undefined_void_delete_and_special_literals_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const known = 1;
            const is_void = void known;
            const a = typeof undefined;
            const b = typeof is_void;
            const c = typeof NaN;
            const d = typeof Infinity;
            const e = is_void === undefined;
            const f = delete known;
            const g = delete missing;
            const h = NaN === NaN;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "undefined:undefined:number:number:true:false:true:false",
    )?;
    Ok(())
}

#[test]
fn await_operator_supports_values_and_fulfilled_promises() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const direct = await 7;
            const promised = await Promise.resolve('ok');
            document.getElementById('result').textContent = direct + ':' + promised;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "7:ok")?;
    Ok(())
}

#[test]
fn async_function_declaration_and_expression_return_promises() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function resolveNow(value) {
            return Promise.resolve(value);
          }

          async function asyncDecl() {
            const first = await resolveNow('A');
            return first + 'B';
          }

          const asyncExpr = async function(value = 'C') {
            const second = await Promise.resolve(value);
            return second + 'D';
          };

          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const p1 = asyncDecl();
            const p2 = asyncExpr();
            result.textContent = typeof p1;
            Promise.all([p1, p2]).then((values) => {
              result.textContent = result.textContent + ':' + values[0] + ':' + values[1];
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "object:AB:CD")?;
    Ok(())
}

#[test]
fn async_function_returned_promise_reference_differs_from_returned_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const p = Promise.resolve(1);

          async function asyncReturn() {
            return p;
          }

          function basicReturn() {
            return Promise.resolve(p);
          }

          document.getElementById('btn').addEventListener('click', () => {
            const sameBasic = p === basicReturn();
            const sameAsync = p === asyncReturn();
            document.getElementById('result').textContent = sameBasic + ':' + sameAsync;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false")?;
    Ok(())
}

#[test]
fn async_function_errors_reject_promise_instead_of_throwing_synchronously() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          async function explode() {
            missingFunction();
            return 'never';
          }

          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const promise = explode();
            result.textContent = 'called';
            promise.catch(() => {
              result.textContent = result.textContent + ':caught';
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "called:caught")?;
    Ok(())
}

#[test]
fn async_function_declaration_is_hoisted_within_scope() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const promise = declaredLater();
            result.textContent = typeof promise;
            promise.then((value) => {
              result.textContent = result.textContent + ':' + value;
            });

            async function declaredLater() {
              return 'ready';
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "object:ready")?;
    Ok(())
}

#[test]
fn async_function_without_await_runs_body_before_returning() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          async function noAwait() {
            document.getElementById('result').textContent += 'A';
            return 1;
          }

          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'B';
            const promise = noAwait();
            document.getElementById('result').textContent += 'C';
            promise.then(() => {
              document.getElementById('result').textContent += ':done';
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "BAC:done")?;
    Ok(())
}

#[test]
fn line_break_between_async_and_function_is_parsed_with_asi() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const async = 'marker';
            async
            function declaredWithLineBreak() {
              return 'ok';
            }

            const value = declaredWithLineBreak();
            document.getElementById('result').textContent =
              typeof value + ':' + value + ':' + async;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "string:ok:marker")?;
    Ok(())
}

#[test]
fn nullish_coalescing_operator_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = null ?? 'x';
            const b = undefined ?? 'y';
            const c = false ?? 'z';
            const d = 0 ?? 9;
            const e = '' ?? 'fallback';
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "x:y:false:0:")?;
    Ok(())
}

#[test]
fn logical_assignment_operators_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let a = 0;
            let b = 2;
            let c = null;
            let d = 'keep';
            let e = 0;
            let f = 'set';

            a ||= 5;
            b &&= 7;
            c ??= 9;
            d ||= 'alt';
            e &&= 4;
            f ??= 'x';

            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5:7:9:keep:0:set")?;
    Ok(())
}

#[test]
fn destructuring_assignment_for_array_and_object_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let first = 0;
            let second = 2;
            let third = 0;
            let a = '';
            let b = '';

            [first, , third] = [10, 20, 30];
            { a, b } = { a: 'A', b: 'B', c: 'C' };

            document.getElementById('result').textContent =
              first + ':' + second + ':' + third + ':' + a + ':' + b;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "10:2:30:A:B")?;
    Ok(())
}

#[test]
fn destructuring_declaration_for_array_and_object_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const [first, , third] = [10, 20, 30];
            const { a, b: renamed } = { a: 'A', b: 'B', c: 'C' };
            document.getElementById('result').textContent =
              first + ':' + third + ':' + a + ':' + renamed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "10:30:A:B")?;
    Ok(())
}

#[test]
fn yield_and_yield_star_operators_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = yield 3;
            const b = yield* (2 + 3);
            document.getElementById('result').textContent = a + ':' + b;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:5")?;
    Ok(())
}

#[test]
fn spread_syntax_for_array_and_object_literals_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const base = [2, 3];
            const arr = [1, ...base, 4];
            const obj1 = { a: 1, b: 2 };
            const obj2 = { ...obj1, b: 9, c: 3 };
            document.getElementById('result').textContent =
              arr.join(',') + '|' + obj2.a + ':' + obj2.b + ':' + obj2.c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,2,3,4|1:9:3")?;
    Ok(())
}

#[test]
fn comma_operator_returns_last_value_in_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const x = (1, 2, 3);
            const y = (alert('first'), alert('second'), 'ok');
            document.getElementById('result').textContent = x + ':' + y;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:ok")?;
    assert_eq!(
        h.take_alert_messages(),
        vec!["first".to_string(), "second".to_string()]
    );
    Ok(())
}

#[test]
fn in_operator_works_with_query_selector_all_indexes() -> Result<()> {
    let html = r#"
        <div id='a'>A</div>
        <div id='b'>B</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nodes = document.querySelectorAll('#a, #b');
            const a = 0 in nodes;
            const b = 1 in nodes;
            const c = 2 in nodes;
            const d = '1' in nodes;
            document.getElementById('result').textContent = a + ':' + b + ':' + c + ':' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false:true")?;
    Ok(())
}

#[test]
fn instanceof_operator_works_with_node_membership_and_identity() -> Result<()> {
    let html = r#"
        <div id='a'>A</div>
        <div id='b'>B</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a_node = document.getElementById('a');
            const b_node = document.getElementById('b');
            const a_only = document.querySelectorAll('#a');
            const same = a_node instanceof a_node;
            const member = a_node instanceof a_only;
            const other = b_node instanceof a_only;
            document.getElementById('result').textContent = same + ':' + member + ':' + other;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn instanceof_html_element_constructors_work_for_dom_nodes() -> Result<()> {
    let html = r#"
        <input id='name' value='A'>
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const input = document.getElementById('name');
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              (input instanceof HTMLInputElement) + ':' +
              (box instanceof HTMLInputElement) + ':' +
              (input instanceof HTMLElement) + ':' +
              (box instanceof HTMLElement) + ':' +
              (window.HTMLInputElement === HTMLInputElement);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true:true:true")?;
    Ok(())
}

#[test]
fn array_find_index_uses_array_runtime_not_typed_array_runtime() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const normalized = ['sku', 'name', 'moq'];
            const found = normalized.findIndex((field) => field === 'name');
            const missing = normalized.findIndex((field) => field === 'none');
            document.getElementById('result').textContent = found + ':' + missing;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:-1")?;
    Ok(())
}

#[test]
fn object_property_access_is_case_sensitive() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const page = {
              plans: {
                minexcess: { title: 'Min' },
              },
            };
            document.getElementById('result').textContent =
              String(page.plans.minExcess === undefined) + ':' +
              page.plans.minexcess.title;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:Min")?;
    Ok(())
}

#[test]
fn instanceof_html_input_element_works_for_input_event_target() -> Result<()> {
    let html = r#"
        <input id='name' value=''>
        <p id='result'></p>
        <script>
          document.getElementById('name').addEventListener('input', (event) => {
            const target = event.target;
            if (!(target instanceof HTMLInputElement)) {
              document.getElementById('result').textContent = 'ng';
              return;
            }
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "A")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn input_event_handler_updates_row_via_closest_and_dataset() -> Result<()> {
    let html = r#"
        <table>
          <tbody id='rows-tbody'>
            <tr data-row-id='r1'>
              <td><input id='moq' data-field='moq' value=''></td>
            </tr>
          </tbody>
        </table>
        <p id='result'></p>
        <script>
          const state = { rows: [{ id: 'r1', moq: '' }] };
          const tbody = document.getElementById('rows-tbody');

          tbody.addEventListener('input', (event) => {
            const target = event.target;
            if (!(target instanceof HTMLInputElement)) return;
            const rowEl = target.closest('tr[data-row-id]');
            if (!rowEl) {
              document.getElementById('result').textContent = 'no-row';
              return;
            }
            const rowID = rowEl.getAttribute('data-row-id');
            const field = target.dataset.field;
            const row = state.rows.find((item) => item.id === rowID);
            if (!row) {
              document.getElementById('result').textContent = 'no-match';
              return;
            }
            row[field] = target.value;
            document.getElementById('result').textContent = row.moq;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#moq", "100")?;
    h.assert_text("#result", "100")?;
    Ok(())
}

#[test]
fn input_event_handler_keeps_dataset_camel_case_keys() -> Result<()> {
    let html = r#"
        <table>
          <tbody id='rows-tbody'>
            <tr data-row-id='r1'>
              <td><input id='case' data-field='casePack' value=''></td>
              <td><input id='desired' data-field='desiredQty' value=''></td>
            </tr>
          </tbody>
        </table>
        <p id='result'></p>
        <script>
          const state = { rows: [{ id: 'r1', casePack: '', desiredQty: '' }] };
          const tbody = document.getElementById('rows-tbody');

          function paint() {
            const row = state.rows[0];
            document.getElementById('result').textContent = row.casePack + ':' + row.desiredQty;
          }

          tbody.addEventListener('input', (event) => {
            const target = event.target;
            if (!(target instanceof HTMLInputElement)) return;
            const rowEl = target.closest('tr[data-row-id]');
            if (!rowEl) return;
            const rowID = rowEl.getAttribute('data-row-id');
            const field = target.dataset.field;
            const row = state.rows.find((item) => item.id === rowID);
            if (!row || !field) return;
            row[field] = target.value;
            paint();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#case", "24")?;
    h.type_text("#desired", "100")?;
    h.assert_text("#result", "24:100")?;
    Ok(())
}

#[test]
fn type_text_handles_input_handler_that_rerenders_same_subtree() -> Result<()> {
    let html = r#"
        <table>
          <tbody id='rows-tbody'></tbody>
        </table>
        <script>
          const state = { rows: [{ id: 'r1', moq: '' }] };
          const tbody = document.getElementById('rows-tbody');

          function escapeHtml(value) {
            const map = {
              "&": "\u0026amp;",
              "<": "\u0026lt;",
              ">": "\u0026gt;",
              '"': "\u0026quot;",
              "'": "\u0026#39;",
            };
            return String(value === null || value === undefined ? "" : value).replace(/[&<>"']/g, (ch) => map[ch] || ch);
          }

          function renderRowsTable() {
            const html = state.rows
              .map((row) => `<tr data-row-id="${row.id}"><td><input data-field="moq" value="${escapeHtml(row.moq)}" /></td></tr>`)
              .join("");
            tbody.innerHTML = html;
          }

          tbody.addEventListener("input", (event) => {
            const target = event.target;
            if (!(target instanceof HTMLInputElement)) return;

            const row = state.rows.find((item) => item.id === "r1");
            if (!row) return;

            row.moq = target.value;
            renderRowsTable();
          });

          renderRowsTable();
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#rows-tbody tr input[data-field='moq']", "100")?;
    h.assert_value("#rows-tbody tr input[data-field='moq']", "100")?;
    Ok(())
}

#[test]
fn array_map_on_object_path_keeps_elements() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const state = { rows: [{ id: 'r1', moq: '100' }] };
            const computed = state.rows.map((row) => ({ id: row.id, moq: row.moq }));
            document.getElementById('result').textContent = computed.length + ':' + computed[0].moq;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:100")?;
    Ok(())
}

#[test]
fn state_rows_initialized_with_empty_row_and_computed_rows() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let uid = 0;
            function nextRowID() {
              uid += 1;
              return `row-${uid}`;
            }
            function emptyRow() {
              return {
                id: nextRowID(),
                sku: "",
                moq: "",
              };
            }

            const state = { rows: [] };
            let computedRows = [];
            if (!state.rows.length) {
              state.rows = [emptyRow()];
            }
            computedRows = state.rows.map((row) => row);
            const first = computedRows.find((row) => true);
            document.getElementById('result').textContent =
              state.rows.length + ':' + computedRows.length + ':' + first.id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:1:row-1")?;
    Ok(())
}

#[test]
fn compute_rows_map_and_find_from_object_path_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const state = { rows: [{ id: 'r1', moq: '100', casePack: '24', desiredQty: '100' }] };

            function computeRow(row) {
              return {
                row,
                status: "ok",
                message: "",
                plans: [{ key: "min_excess", selectable: true }],
              };
            }

            const computedRows = state.rows.map((row) => computeRow(row));
            const first = computedRows.find((item) => item.status === "ok");
            document.getElementById('result').textContent =
              computedRows.length + ':' + (first ? first.status : 'none');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:ok")?;
    Ok(())
}

#[test]
fn computed_rows_global_is_visible_in_later_click_event() -> Result<()> {
    let html = r#"
        <input id='moq' />
        <button id='copy'>copy</button>
        <p id='result'></p>
        <script>
          let computedRows = [];

          function computeAll() {
            computedRows = [{ status: 'ok' }];
          }

          function renderAll() {
            computeAll();
          }

          document.getElementById('moq').addEventListener('input', () => {
            renderAll();
          });

          document.getElementById('copy').addEventListener('click', () => {
            let status = '';
            computedRows.forEach((item) => {
              if (!status) status = item.status;
            });
            document.getElementById('result').textContent =
              String(computedRows.length) + ':' + status;
          });

          renderAll();
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#moq", "100")?;
    h.click("#copy")?;
    h.assert_text("#result", "1:ok")?;
    Ok(())
}
