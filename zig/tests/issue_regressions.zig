const std = @import("std");
const bt = @import("browser_tester_zig");

fn harnessFromHtml(html: []const u8) bt.Result(bt.Harness) {
    return bt.Harness.fromHtml(std.testing.allocator, html);
}

test "issue 134 object assign global is available" {
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  const out = document.getElementById('out');
        \\  try {
        \\    const target = { a: 1 };
        \\    const src = { b: 2 };
        \\    Object.assign(target, src);
        \\    out.textContent = String(target.a) + ':' + String(target.b);
        \\  } catch (err) {
        \\    out.textContent = 'err:' + String(err && err.message ? err.message : err);
        \\  }
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "1:2");
}

test "issue 134 object assign returns target and ignores nullish sources" {
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  const target = { a: 1, b: 1 };
        \\  const returned = Object.assign(target, null, { b: 4 }, undefined, { c: 5 });
        \\  document.getElementById('out').textContent = [
        \\    String(target.a),
        \\    String(target.b),
        \\    String(target.c),
        \\    String(returned === target),
        \\  ].join('|');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "1|4|5|true");
}

test "issue 134 object assign uses getters and setters" {
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  let getCount = 0;
        \\  let setTotal = 0;
        \\  const source = {
        \\    get amount() {
        \\      getCount += 1;
        \\      return 7;
        \\    }
        \\  };
        \\  const target = {
        \\    set amount(value) {
        \\      setTotal += value;
        \\    }
        \\  };
        \\  Object.assign(target, source);
        \\  document.getElementById('out').textContent = [
        \\    String(getCount),
        \\    String(setTotal),
        \\  ].join('|');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "1|7");
}

test "issue 134 object assign wraps primitive target and rejects null target" {
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  const wrapped = Object.assign(3, { a: 1 });
        \\  let threwForNull = false;
        \\  try {
        \\    Object.assign(null, { a: 1 });
        \\  } catch (err) {
        \\    threwForNull = String(err && err.message ? err.message : err)
        \\      .includes('Cannot convert undefined or null to object');
        \\  }
        \\  document.getElementById('out').textContent = [
        \\    typeof wrapped,
        \\    String(wrapped.a),
        \\    String(threwForNull),
        \\  ].join('|');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "object|1|true");
}

test "issue 135 optional chaining listener on member path parses and runs" {
    const html =
        \\<button id='btn'>run</button>
        \\<p id='out'></p>
        \\<script>
        \\  const actionEls = { close: document.getElementById('btn') };
        \\  actionEls.close?.addEventListener('click', () => {
        \\    document.getElementById('out').textContent = 'ok';
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#btn");
    try subject.assertValue("#out", "ok");
}

test "issue 136 html button element global supports instanceof checks" {
    const html =
        \\<button id='btn'>run</button>
        \\<p id='out'></p>
        \\<script>
        \\  document.getElementById('btn').addEventListener('click', (event) => {
        \\    const checks = [
        \\      typeof HTMLButtonElement,
        \\      String(window.HTMLButtonElement === HTMLButtonElement),
        \\      String(event.target instanceof HTMLButtonElement),
        \\      String(event.target instanceof HTMLElement),
        \\    ];
        \\    document.getElementById('out').textContent = checks.join('|');
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#btn");
    try subject.assertValue("#out", "function|true|true|true");
}

test "issue 137 tofixed chain parses after escape normalization with unicode" {
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  const quotePair = "\"\"";
        \\  const label = `ABC-001 (${quotePair.length} 件)`;
        \\  const rect = { w: 4.2 };
        \\  const formatted = Math.max(0, rect.w).toFixed(2);
        \\  document.getElementById('out').textContent = `${label}|${formatted}`;
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "ABC-001 (2 件)|4.20");
}

test "issue 138 generic format two args is not hijacked as intl relative time" {
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  const helper = {
        \\    format(template, values) {
        \\      return template
        \\        .replace('{shown}', String(values.shown))
        \\        .replace('{total}', String(values.total));
        \\    }
        \\  };
        \\  const shown = 3;
        \\  const total = 8;
        \\  document.getElementById('out').textContent = helper.format('Shown {shown}/{total}', { shown, total });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "Shown 3/8");
}

test "issue 139 function can reference later declared const" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  function ensure() {
        \\    return state.value;
        \\  }
        \\  const state = { value: 123 };
        \\  document.getElementById('out').textContent = String(ensure());
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "123");
}

test "issue 140 nested state paths are not treated as dom element variables" {
    const html =
        \\<button id='btn'>run</button>
        \\<p id='out'></p>
        \\<script>
        \\  const state = {
        \\    ratio: { mode: 'a' },
        \\    measurements: [{ value: 1 }],
        \\  };
        \\  document.getElementById('btn').addEventListener('click', () => {
        \\    state.ratio.mode = 'b';
        \\    state.measurements[0].value = 2;
        \\    document.getElementById('out').textContent = state.ratio.mode + ':' + String(state.measurements[0].value);
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#btn");
    try subject.assertValue("#out", "b:2");
}

test "issue 151 pickmap get or fallback does not overwrite map binding" {
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  const pickMap = new Map();
        \\  pickMap.set('W1', { fixed: 0, unit: 2 });
        \\  const getUnitCost = (warehouse) => {
        \\    const pick = pickMap.get(warehouse) || { fixed: 0, unit: 0 };
        \\    return pick.unit;
        \\  };
        \\  const costMissing = getUnitCost('W9');
        \\  const costExisting = getUnitCost('W1');
        \\  const mapValue = pickMap.get('W1');
        \\  document.getElementById('out').textContent = [
        \\    String(costMissing),
        \\    String(costExisting),
        \\    mapValue ? String(mapValue.unit) : 'none',
        \\  ].join('|');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "0|2|2");
}

test "issue 151 plain object get method is not hijacked as form data" {
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  const store = {
        \\    get(name) {
        \\      return 'value:' + name;
        \\    }
        \\  };
        \\  document.getElementById('out').textContent = store.get('W1');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "value:W1");
}

test "issue 153 dynamic index compound assignment is supported" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  const values = [1, 2];
        \\  const index = 1;
        \\  values[index] += 3;
        \\  document.getElementById('out').textContent = String(values[1]);
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "5");
}

test "issue 154 function listener binds this to current target" {
    const html =
        \\<button id='button' data-value='ok'>go</button>
        \\<div id='out'></div>
        \\<script>
        \\  const button = document.getElementById('button');
        \\  const out = document.getElementById('out');
        \\  button.addEventListener('click', function () {
        \\    out.textContent = this.getAttribute('data-value');
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#button");
    try subject.assertValue("#out", "ok");
}

test "issue 155 closest accepts selector variable in if condition" {
    const html =
        \\<div class='btn-wrap'>
        \\  <span id='child'>child</span>
        \\</div>
        \\<p id='out'></p>
        \\<script>
        \\  const child = document.getElementById('child');
        \\  const buttonWrapSelector = '.btn-wrap, .button-block';
        \\  if (child.closest(buttonWrapSelector)) {
        \\    document.getElementById('out').textContent = 'matched';
        \\  }
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "matched");
}

test "issue 157 date toLocaleDateString is available" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  const date = new Date('2024-02-03T00:00:00Z');
        \\  document.getElementById('out').textContent = date.toLocaleDateString('en-US');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "2/3/2024");
}

test "issue 158 closest accepts selector variable in expression position" {
    const html =
        \\<section class='card'>
        \\  <button id='child'>open</button>
        \\</section>
        \\<p id='out'></p>
        \\<script>
        \\  const child = document.getElementById('child');
        \\  const selector = '.card';
        \\  const matched = child.closest(selector);
        \\  document.getElementById('out').textContent = matched ? matched.tagName : 'none';
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "SECTION");
}

test "issue 160 array flatmap is supported" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  const values = ['north', 'south'];
        \\  const result = values.flatMap((value) => [value]);
        \\  document.getElementById('out').textContent = result.join(',');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "north,south");
}

test "issue 168 object from entries supports page init lookup tables" {
    const html =
        \\<pre id='out'></pre>
        \\<script>
        \\  const kanaPairs = [
        \\    ['full', 'アイウ'],
        \\    ['half', 'ｱｲｳ']
        \\  ];
        \\  const normalized = Object.fromEntries(
        \\    kanaPairs.map(([key, value]) => [key, value.slice(0, 2)])
        \\  );
        \\  const aliases = Object.fromEntries(
        \\    new Map([
        \\      ['zenkaku', normalized.full],
        \\      ['hankaku', normalized.half]
        \\    ])
        \\  );
        \\  document.getElementById('out').textContent =
        \\    aliases.zenkaku + '|' + aliases.hankaku + '|' + Object.keys(aliases).join(',');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "アイ|ｱｲ|zenkaku,hankaku");
}

test "issue 185 inline object literal computed lookup returns selected value" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  const backgroundCss = {
        \\    checker: 'background:#f8fafc;',
        \\    dark: 'background:#0f172a;'
        \\  }['dark'] || 'background:#ffffff;';
        \\  const zoomScale = {
        \\    fit: 1,
        \\    '200': 2
        \\  }['200'] || 1;
        \\  document.getElementById('out').textContent =
        \\    backgroundCss + '|' + String(zoomScale);
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "background:#0f172a;|2");
}

test "issue 190 document active element tag name is supported" {
    const html =
        \\<textarea id='field'></textarea>
        \\<div id='out'></div>
        \\<script>
        \\  const field = document.getElementById('field');
        \\  field.focus();
        \\  document.getElementById('out').textContent = document.activeElement.tagName;
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "TEXTAREA");
}

test "issue 191 data url anchor download is captured as artifact" {
    const html =
        \\<button id='download'>Download</button>
        \\<script>
        \\  document.getElementById('download').addEventListener('click', () => {
        \\    const csv = '\ufeffa,b\n1,2';
        \\    const link = document.createElement('a');
        \\    link.href = `data:text/csv;charset=utf-8,${encodeURIComponent(csv)}`;
        \\    link.download = 'sample.csv';
        \\    document.body.appendChild(link);
        \\    link.click();
        \\    document.body.removeChild(link);
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#download");
    const downloads = subject.mocksMut().downloads().artifacts();
    try std.testing.expectEqual(@as(usize, 1), downloads.len);
    try std.testing.expectEqualStrings("sample.csv", downloads[0].file_name);
    try std.testing.expectEqualStrings("\xEF\xBB\xBFa,b\n1,2", downloads[0].bytes);
}

test "issue 192 array flat is supported" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  const values = [['north'], ['south']].flat();
        \\  document.getElementById('out').textContent = values.join(',');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "north,south");
}

test "issue 193 postfix increment inside expression is supported" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  let rowSeq = 1;
        \\  function createDefaultRow(partial = {}) {
        \\    return {
        \\      id: partial.id || 'r' + rowSeq++,
        \\    };
        \\  }
        \\  document.getElementById('out').textContent = createDefaultRow({}).id;
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "r1");
}

test "issue 194 array destructure assignment inside else if branch is supported" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  const state = {
        \\    rows: [{ id: 'a' }, { id: 'b' }, { id: 'c' }],
        \\  };
        \\  function reorder(action, index) {
        \\    if (action === 'duplicate') {
        \\      state.rows.splice(index + 1, 0, state.rows[index]);
        \\    } else if (action === 'delete') {
        \\      state.rows.splice(index, 1);
        \\    } else if (action === 'up' && index > 0) {
        \\      [state.rows[index - 1], state.rows[index]] = [state.rows[index], state.rows[index - 1]];
        \\    } else if (action === 'down' && index < state.rows.length - 1) {
        \\      [state.rows[index + 1], state.rows[index]] = [state.rows[index], state.rows[index + 1]];
        \\    }
        \\  }
        \\  reorder('up', 2);
        \\  document.getElementById('out').textContent = state.rows.map((row) => row.id).join(',');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "a,c,b");
}

test "issue 202 window property reads as global identifier inside function" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  function installAndRead() {
        \\    window.hashApi = { tag: 'ok' };
        \\    document.getElementById('out').textContent =
        \\      typeof hashApi + ':' + hashApi.tag;
        \\  }
        \\  installAndRead();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "object:ok");
}

test "issue 211 dump dom preserves adjusted svg attribute casing" {
    const html =
        \\<div id="probe">
        \\  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
        \\    <defs>
        \\      <marker
        \\        id="arrow"
        \\        viewBox="0 0 4 4"
        \\        markerWidth="4"
        \\        markerHeight="4"
        \\        refX="2"
        \\        refY="2"
        \\      >
        \\        <path d="M0,0 L4,2 L0,4 z"></path>
        \\      </marker>
        \\    </defs>
        \\  </svg>
        \\</div>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    const snippet = try subject.dumpDom(std.testing.allocator);
    defer std.testing.allocator.free(snippet);

    try std.testing.expect(std.mem.indexOf(u8, snippet, "viewBox=\"0 0 10 10\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, snippet, "viewBox=\"0 0 4 4\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, snippet, "markerWidth=\"4\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, snippet, "markerHeight=\"4\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, snippet, "refX=\"2\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, snippet, "refY=\"2\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, snippet, "viewbox=") == null);
}

test "issue 213 array index of and last index of match standard array search" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  const values = ['alpha', 'beta', 'gamma', 'beta'];
        \\  document.getElementById('out').textContent = [
        \\    String(values.indexOf('beta')),
        \\    String(values.indexOf('beta', 2)),
        \\    String(values.indexOf('beta', -2)),
        \\    String(values.lastIndexOf('beta')),
        \\    String(values.lastIndexOf('beta', 2)),
        \\    String(values.lastIndexOf('beta', -3))
        \\  ].join('|');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "1|3|3|3|1|1");
}

test "issue 214 array map callback mutations update outer let bindings" {
    const html =
        \\<div id='calculated'>0</div>
        \\<div id='errors'>0</div>
        \\<div id='preview'></div>
        \\<script>
        \\  const rows = [
        \\    { ok: true, label: 'valid' },
        \\    { ok: false, label: 'invalid' }
        \\  ];
        \\  let calculatedCount = 0;
        \\  let errorCount = 0;
        \\  const previewRows = rows.map((row) => {
        \\    const notes = [];
        \\    if (!row.ok) {
        \\      notes.push('bad');
        \\      errorCount += 1;
        \\      return { label: row.label, notes };
        \\    }
        \\    calculatedCount += 1;
        \\    return { label: row.label, notes };
        \\  });
        \\  document.getElementById('calculated').textContent = String(calculatedCount);
        \\  document.getElementById('errors').textContent = String(errorCount);
        \\  document.getElementById('preview').textContent = previewRows
        \\    .map((row) => row.label + ':' + row.notes.join(';'))
        \\    .join('|');
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#calculated", "1");
    try subject.assertValue("#errors", "1");
    try subject.assertValue("#preview", "valid:|invalid:bad");
}

test "issue 215 nested helper local index does not poison plain const declaration" {
    const html =
        \\<button id='go' type='button'>go</button>
        \\<div id='out'></div>
        \\<script>
        \\  (() => {
        \\    const state = { nested: {} };
        \\    function setDeepValue(obj, path, value) {
        \\      const parts = path.split('.');
        \\      let current = obj;
        \\      for (let index = 0; index < parts.length - 1; index += 1) {
        \\        const part = parts[index];
        \\        if (!current[part] || typeof current[part] !== 'object') {
        \\          current[part] = {};
        \\        }
        \\        current = current[part];
        \\      }
        \\      current[parts[parts.length - 1]] = value;
        \\    }
        \\    document.getElementById('go').addEventListener('click', () => {
        \\      setDeepValue(state.nested, 'percent.rateRaw', '20');
        \\      const index = 1;
        \\      document.getElementById('out').textContent =
        \\        String(index) + ':' + state.nested.percent.rateRaw;
        \\    });
        \\  })();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#go");
    try subject.assertValue("#out", "1:20");
}

test "issue 215 nested helper local index does not poison later const index" {
    const html =
        \\<button id="go" type="button">go</button>
        \\<div id="out"></div>
        \\<script>
        \\  (() => {
        \\    const state = {
        \\      stack: {
        \\        steps: [
        \\          { id: 'step-1', config: { type: 'percent', percent: { rateRaw: '10' } } },
        \\          { id: 'step-2', config: { type: 'fixed', fixed: { amountRaw: '10' } } }
        \\        ]
        \\      }
        \\    };
        \\    function setDeepValue(obj, path, value) {
        \\      if (!obj || !path) return;
        \\      const parts = path.split('.');
        \\      let current = obj;
        \\      for (let index = 0; index < parts.length - 1; index += 1) {
        \\        const part = parts[index];
        \\        if (!current[part] || typeof current[part] !== 'object') {
        \\          current[part] = {};
        \\        }
        \\        current = current[part];
        \\      }
        \\      current[parts[parts.length - 1]] = value;
        \\    }
        \\    function describeStep(step) {
        \\      if (step.config.type === 'percent') {
        \\        return step.id + ':' + step.config.percent.rateRaw;
        \\      }
        \\      return step.id + ':' + step.config.fixed.amountRaw;
        \\    }
        \\    function render() {
        \\      document.getElementById('out').textContent = state.stack.steps
        \\        .map((step) => describeStep(step))
        \\        .join('|');
        \\    }
        \\    document.getElementById('go').addEventListener('click', () => {
        \\      const edited = state.stack.steps.find((item) => item.id === 'step-1');
        \\      if (!edited) return;
        \\      setDeepValue(edited.config, 'percent.rateRaw', '20');
        \\      const index = state.stack.steps.findIndex((item) => item.id === 'step-2');
        \\      const moved = state.stack.steps.splice(index, 1)[0];
        \\      state.stack.steps.splice(index - 1, 0, moved);
        \\      render();
        \\    });
        \\    render();
        \\  })();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#go");
    try subject.assertValue("#out", "step-2:10|step-1:20");
}

test "issue 217 nested helper call in return expression keeps outer return value" {
    const html =
        \\<div id="out"></div>
        \\<script>
        \\  (() => {
        \\    function renderLabel(label) {
        \\      return `<div class="field">${escapeHtml(label)}</div>`;
        \\    }
        \\    function escapeHtml(value) {
        \\      return String(value || "")
        \\        .replace(/&/g, "&amp;")
        \\        .replace(/</g, "&lt;")
        \\        .replace(/>/g, "&gt;")
        \\        .replace(/"/g, "&quot;")
        \\        .replace(/'/g, "&#39;");
        \\    }
        \\    document.getElementById("out").textContent = renderLabel("Holding rate");
        \\  })();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "<div class=\"field\">Holding rate</div>");
}

test "issue 217 batch mapping grid keeps select markup with late helper declaration" {
    const html =
        \\<div id="grid"></div>
        \\<script>
        \\  (() => {
        \\    function renderBatchMappingGrid() {
        \\      const labels = [
        \\        "#1 annual_demand",
        \\        "#2 order_cost",
        \\        "#3 alt_rate",
        \\        "#4 unit_cost",
        \\      ];
        \\      const mapping = {
        \\        annualDemand: 0,
        \\        orderCost: 1,
        \\        holdingRate: -1,
        \\        unitCost: 3,
        \\      };
        \\      const mappingFields = [
        \\        ["annualDemand", "Annual demand"],
        \\        ["orderCost", "Order cost"],
        \\        ["holdingRate", "Holding rate"],
        \\        ["unitCost", "Unit cost"],
        \\      ];
        \\      document.getElementById("grid").innerHTML = mappingFields.map(([key, label]) => {
        \\        const options = [`<option value="-1">${escapeHtml("Unused")}</option>`]
        \\          .concat(labels.map((header, index) => `<option value="${index}" ${mapping[key] === index ? "selected" : ""}>${escapeHtml(header)}</option>`))
        \\          .join("");
        \\        return `<div class="field">
        \\          <label class="field-label" for="eoq-calculator-map-${key}">${escapeHtml(label)}</label>
        \\          <select id="eoq-calculator-map-${key}" data-map-key="${key}">${options}</select>
        \\        </div>`;
        \\      }).join("");
        \\    }
        \\    function escapeHtml(value) {
        \\      return String(value || "")
        \\        .replace(/&/g, "&amp;")
        \\        .replace(/</g, "&lt;")
        \\        .replace(/>/g, "&gt;")
        \\        .replace(/"/g, "&quot;")
        \\        .replace(/'/g, "&#39;");
        \\    }
        \\    renderBatchMappingGrid();
        \\  })();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertExists("#eoq-calculator-map-holdingRate");
    try subject.assertValue("#eoq-calculator-map-orderCost", "1");

    const snippet = try subject.dumpDom(std.testing.allocator);
    defer std.testing.allocator.free(snippet);

    try std.testing.expect(std.mem.indexOf(u8, snippet, "id=\"eoq-calculator-map-holdingRate\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, snippet, "<option value=\"2\">#3 alt_rate</option>") != null);
}

test "issue 212 bulk callback can update outer let counter inside listener render" {
    const html =
        \\<textarea id="bulk"></textarea>
        \\<div id="out"></div>
        \\<script>
        \\  (() => {
        \\    const state = {
        \\      bulkText: "",
        \\    };
        \\    function parseRows(text) {
        \\      return text
        \\        .split(/\r?\n/)
        \\        .filter((line) => line.trim() !== "")
        \\        .map((line) => line.split(","));
        \\    }
        \\    function computeBulkResult() {
        \\      const rows = parseRows(state.bulkText);
        \\      let calculated = 0;
        \\      const resultRows = rows.map((row) => {
        \\        if (row[0]) {
        \\          calculated += 1;
        \\        }
        \\        return row[0] || "";
        \\      });
        \\      return `${calculated}:${resultRows.join("|")}`;
        \\    }
        \\    function render() {
        \\      document.getElementById("out").textContent = computeBulkResult();
        \\    }
        \\    document.getElementById("bulk").addEventListener("input", () => {
        \\      state.bulkText = document.getElementById("bulk").value;
        \\      render();
        \\    });
        \\    render();
        \\  })();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.typeText("#bulk", "A,1\nB,2");
    try subject.assertValue("#out", "2:A|B");
}
