use super::*;

#[test]
fn element_client_width_includes_padding_and_excludes_border() -> Result<()> {
    let html = r#"
        <div id='box' style='width: 100px; padding: 20px; padding-right: 10px; border: 5px solid black;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent = String(box.clientWidth);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "130")?;
    Ok(())
}

#[test]
fn element_client_width_is_zero_for_inline_or_no_layout_box_elements() -> Result<()> {
    let html = r#"
        <span id='inline' style='display:inline; width: 100px; padding: 20px;'></span>
        <div id='hidden' style='display:none; width: 100px; padding: 20px;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const inline = document.getElementById('inline');
            const hidden = document.getElementById('hidden');
            const detached = document.createElement('div');
            detached.style.width = '40px';
            detached.style.padding = '10px';
            document.getElementById('result').textContent = [
              inline.clientWidth,
              hidden.clientWidth,
              detached.clientWidth
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0:0:0")?;
    Ok(())
}

#[test]
fn document_element_client_width_uses_window_inner_width() -> Result<()> {
    let html = r#"
        <html>
          <body>
            <button id='run'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                window.innerWidth = 845.9;
                document.getElementById('result').textContent =
                  String(document.documentElement.clientWidth);
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "845")?;
    Ok(())
}

#[test]
fn layout_derived_properties_shadow_define_property_delete_and_fast_path_parity_work() -> Result<()>
{
    let html = r#"
        <div
          id='box'
          style='width: 100px; height: 80px; padding-left: 7px; padding-right: 15px; padding-top: 10px; padding-bottom: 5px; border-left: 4px solid black; border-top: 6px solid black;'
        ></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');

            Object.defineProperty(box, 'clientWidth', { value: 'cw', configurable: true });
            Object.defineProperty(box, 'clientHeight', { get() { return 'ch'; }, configurable: true });
            Object.defineProperty(box, 'clientLeft', { value: 'cl', configurable: true });
            Object.defineProperty(box, 'clientTop', { value: 'ct', configurable: true });
            Object.defineProperty(box, 'currentCSSZoom', { value: 'zoom', configurable: true });
            Object.defineProperty(box, 'offsetWidth', { value: 'ow', configurable: true });
            Object.defineProperty(box, 'offsetHeight', { value: 'oh', configurable: true });
            Object.defineProperty(box, 'offsetLeft', { value: 'ol', configurable: true });
            Object.defineProperty(box, 'offsetTop', { value: 'ot', configurable: true });
            Object.defineProperty(box, 'scrollWidth', { value: 'sw', configurable: true });
            Object.defineProperty(box, 'scrollHeight', { value: 'sh', configurable: true });
            Object.defineProperty(box, 'scrollLeft', { value: 'sl', configurable: true });
            Object.defineProperty(box, 'scrollTop', { value: 'st', configurable: true });
            Object.defineProperty(box, 'scrollLeftMax', { value: 'slm', configurable: true });
            Object.defineProperty(box, 'scrollTopMax', { value: 'stm', configurable: true });

            const shadow = [
              box.clientWidth,
              box.clientHeight,
              box.clientLeft,
              box.clientTop,
              box.currentCSSZoom,
              box.offsetWidth,
              box.offsetHeight,
              box.offsetLeft,
              box.offsetTop,
              box.scrollWidth,
              box.scrollHeight,
              box.scrollLeft,
              box.scrollTop,
              box.scrollLeftMax,
              box.scrollTopMax
            ].join(':');

            delete box.clientWidth;
            delete box.clientHeight;
            delete box.clientLeft;
            delete box.clientTop;
            delete box.currentCSSZoom;
            delete box.offsetWidth;
            delete box.offsetHeight;
            delete box.offsetLeft;
            delete box.offsetTop;
            delete box.scrollWidth;
            delete box.scrollHeight;
            delete box.scrollLeft;
            delete box.scrollTop;
            delete box.scrollLeftMax;
            delete box.scrollTopMax;

            const restored = [
              box.clientWidth,
              box.clientHeight,
              box.clientLeft,
              box.clientTop,
              box.currentCSSZoom,
              box.offsetWidth,
              box.offsetHeight,
              box.offsetLeft,
              box.offsetTop,
              box.scrollWidth,
              box.scrollHeight,
              box.scrollLeft,
              box.scrollTop,
              box.scrollLeftMax,
              box.scrollTopMax
            ].join(':');

            document.getElementById('result').textContent = [shadow, restored].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "cw:ch:cl:ct:zoom:ow:oh:ol:ot:sw:sh:sl:st:slm:stm|122:95:4:6:1:0:0:0:0:0:0:0:0:0:0",
    )?;
    Ok(())
}

#[test]
fn reduced_layout_derived_metrics_stay_off_instance_copy_surface_until_shadowed_work() -> Result<()>
{
    let html = r#"
        <div id='box' style='width: 100px; height: 80px; padding-left: 7px; padding-right: 15px; border-left: 4px solid black;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');

            const before = [
              String(Object.getOwnPropertyDescriptor(box, 'clientWidth') === undefined),
              String(Object.getOwnPropertyDescriptor(box, 'offsetWidth') === undefined),
              String(Object.keys(Object.assign({}, box)).includes('clientWidth')),
              String(Object.keys({ ...box }).includes('offsetWidth'))
            ].join(':');

            Object.defineProperty(box, 'clientWidth', {
              value: 'cw',
              enumerable: true,
              configurable: true
            });

            const shadowed = [
              box.clientWidth,
              String(Object.keys(Object.assign({}, box)).includes('clientWidth')),
              String(Object.keys({ ...box }).includes('clientWidth'))
            ].join(':');

            delete box.clientWidth;

            const restored = [
              String(Object.getOwnPropertyDescriptor(box, 'clientWidth') === undefined),
              String(Object.keys(Object.assign({}, box)).includes('clientWidth')),
              String(Object.keys({ ...box }).includes('clientWidth')),
              String(box.clientWidth)
            ].join(':');

            document.getElementById('result').textContent =
              [before, shadowed, restored].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:false:false|cw:true:true|true:false:false:122",
    )?;
    Ok(())
}
