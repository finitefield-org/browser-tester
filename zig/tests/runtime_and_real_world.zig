const std = @import("std");
const bt = @import("browser_tester_zig");

fn harnessFromHtml(html: []const u8) bt.Result(bt.Harness) {
    return bt.Harness.fromHtml(std.testing.allocator, html);
}

test "open issue: click toggles button inside open dialog" {
    const html =
        \\<button id="open">Open</button>
        \\<div id="dialog" class="hidden" role="dialog" aria-modal="true">
        \\  <button id="settings-toggle" type="button" aria-expanded="false">Settings</button>
        \\  <div id="settings-panel" class="hidden">Panel</div>
        \\</div>
        \\<p id="status"></p>
        \\<p id="trace"></p>
        \\<script>
        \\  (() => {
        \\    const el = {
        \\      open: document.getElementById("open"),
        \\      dialog: document.getElementById("dialog"),
        \\      settingsToggle: document.getElementById("settings-toggle"),
        \\      settingsPanel: document.getElementById("settings-panel"),
        \\      status: document.getElementById("status"),
        \\      trace: document.getElementById("trace"),
        \\    };
        \\    let settingsOpen = false;
        \\    function setHiddenClass(node, hidden) {
        \\      node.classList.toggle("hidden", hidden);
        \\    }
        \\    function syncStatus() {
        \\      el.status.textContent = [
        \\        String(settingsOpen),
        \\        el.settingsToggle.getAttribute("aria-expanded"),
        \\        String(el.settingsPanel.classList.contains("hidden")),
        \\      ].join("|");
        \\    }
        \\    function render() {
        \\      el.trace.textContent += "render>";
        \\      setHiddenClass(el.settingsPanel, !settingsOpen);
        \\      el.settingsToggle.setAttribute("aria-expanded", settingsOpen ? "true" : "false");
        \\      syncStatus();
        \\    }
        \\    el.open.addEventListener("click", () => {
        \\      el.trace.textContent += "open>";
        \\      el.dialog.classList.remove("hidden");
        \\      syncStatus();
        \\    });
        \\    el.settingsToggle.addEventListener("click", () => {
        \\      el.trace.textContent += "toggle>";
        \\      settingsOpen = !settingsOpen;
        \\      render();
        \\    });
        \\    syncStatus();
        \\  })();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#open");
    try subject.click("#settings-toggle");

    try subject.assertValue("#trace", "open>toggle>render>");
    try subject.assertValue("#status", "true|true|false");
}

test "open issue: window open returns popup stub for print flows" {
    const html =
        \\<button id="go">go</button>
        \\<div id="out"></div>
        \\<script>
        \\  document.getElementById("go").addEventListener("click", () => {
        \\    const win = window.open("", "_blank", "noopener,noreferrer");
        \\    if (!win) {
        \\      document.getElementById("out").textContent = "null";
        \\      return;
        \\    }
        \\    win.document.open();
        \\    win.document.write("<p>print view</p>");
        \\    win.document.close();
        \\    win.focus();
        \\    win.print();
        \\    document.getElementById("out").textContent = [
        \\      String(win.closed),
        \\      String(win.opener === null),
        \\      String(win.document.readyState),
        \\    ].join("|");
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#go");
    try subject.assertValue("#out", "false|true|complete");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().print().calls().len);
}

test "open issue: click preserves pre request animation frame processing state" {
    const html =
        \\<button id="run" type="button">Run</button>
        \\<div id="processing" class="hidden">Processing</div>
        \\<p id="status"></p>
        \\<script>
        \\  (() => {
        \\    const el = {
        \\      run: document.getElementById("run"),
        \\      processing: document.getElementById("processing"),
        \\      status: document.getElementById("status"),
        \\    };
        \\    function setProcessing(processing) {
        \\      el.processing.classList.toggle("hidden", !processing);
        \\      el.run.disabled = processing;
        \\      el.status.textContent = [
        \\        String(el.run.disabled),
        \\        String(el.processing.classList.contains("hidden")),
        \\      ].join("|");
        \\    }
        \\    function nextFrame() {
        \\      return new Promise((resolve) => {
        \\        window.requestAnimationFrame(() => resolve());
        \\      });
        \\    }
        \\    async function runTask() {
        \\      setProcessing(true);
        \\      await nextFrame();
        \\      setProcessing(false);
        \\    }
        \\    el.run.addEventListener("click", runTask);
        \\    setProcessing(false);
        \\  })();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#run");
    try subject.assertValue("#status", "true|false");

    try subject.flush();
    try subject.assertValue("#status", "false|true");
}

test "open issue: sibling closure calls do not prune scope capture env" {
    const html =
        \\<button id="open">open</button>
        \\<button id="print">print</button>
        \\<div id="out"></div>
        \\<script>
        \\  (() => {
        \\    const el = {
        \\      out: document.getElementById("out"),
        \\      open: document.getElementById("open"),
        \\      print: document.getElementById("print"),
        \\    };
        \\    let state = { mode: "ready" };
        \\    let lastResult = { value: "ok" };
        \\    function openDialog() {
        \\      el.out.dataset.dialog = "open";
        \\    }
        \\    function openPrintView() {
        \\      el.out.textContent = lastResult.value + ":" + state.mode;
        \\    }
        \\    el.open.addEventListener("click", openDialog);
        \\    el.print.addEventListener("click", openPrintView);
        \\  })();
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#open");
    try subject.click("#print");
    try subject.assertValue("#out", "ok:ready");
}

test "runtime state: listener error keeps state changes before throw" {
    const html =
        \\<button id='boom'>boom</button>
        \\<button id='check'>check</button>
        \\<p id='result'></p>
        \\<script>
        \\  let x = 0;
        \\  document.getElementById('boom').addEventListener('click', () => {
        \\    x = 1;
        \\    unknown_fn();
        \\  });
        \\  document.getElementById('check').addEventListener('click', () => {
        \\    document.getElementById('result').textContent = String(x);
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try std.testing.expectError(bt.Error.ScriptRuntime, subject.click("#boom"));
    try subject.click("#check");
    try subject.assertValue("#result", "1");
}

test "runtime state: recursive const arrow closure can reference itself" {
    const html =
        \\<button id='run'>run</button>
        \\<p id='out'></p>
        \\<script>
        \\  const choose = (arr, k) => {
        \\    const out = [];
        \\    const recur = (start, cur) => {
        \\      if (cur.length === k) {
        \\        out.push([...cur]);
        \\        return;
        \\      }
        \\      for (let i = start; i < arr.length; i += 1) {
        \\        cur.push(arr[i]);
        \\        recur(i + 1, cur);
        \\        cur.pop();
        \\      }
        \\    };
        \\    recur(0, []);
        \\    return out;
        \\  };
        \\  document.getElementById('run').addEventListener('click', () => {
        \\    const combos = choose([1, 2, 3], 2);
        \\    document.getElementById('out').textContent = String(combos.length);
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#run");
    try subject.assertValue("#out", "3");
}

test "runtime state: unicode text is preserved even when quote escape normalization runs" {
    const html =
        \\<div id='out'></div>
        \\<script>
        \\  const quotePair = "\"\"";
        \\  const label = `ABC-001 (${quotePair.length} 件)`;
        \\  document.getElementById('out').textContent = label;
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#out", "ABC-001 (2 件)");
}

test "runtime state: object shorthand inside map callback does not trigger false tdz" {
    const html =
        \\<textarea id='t'></textarea>
        \\<button id='run'>run</button>
        \\<p id='out'></p>
        \\<script>
        \\  function detectDelimiter(text) {
        \\    const candidates = [",", "\t"];
        \\    const ranked = candidates.map((delimiter) => {
        \\      const measure = { score: 1 };
        \\      return { delimiter, ...measure };
        \\    }).sort((a, b) => b.score - a.score);
        \\    return ranked.length ? ranked[0].delimiter : ",";
        \\  }
        \\  document.getElementById('run').addEventListener('click', () => {
        \\    const delimiter = detectDelimiter('a,b');
        \\    document.getElementById('out').textContent = delimiter;
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#run");
    try subject.assertValue("#out", ",");
}

test "real world html: ignores json ld script blocks and runs executable script" {
    const html =
        \\<div id="result">init</div>
        \\<script type="application/ld+json">
        \\  {"@context":"https://schema.org","@type":"FAQPage"}
        \\</script>
        \\<script>
        \\  document.getElementById("result").textContent = "ok";
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#result", "ok");
}

test "real world html: array from supports nodelist and map callback" {
    const html =
        \\<ul>
        \\  <li class="item">A</li>
        \\  <li class="item">B</li>
        \\</ul>
        \\<div id="result"></div>
        \\<script>
        \\  const nodes = Array.from(document.querySelectorAll(".item"));
        \\  const mapped = Array.from(nodes, (node, idx) => node.textContent + idx);
        \\  document.getElementById("result").textContent = nodes.length + ":" + mapped.join(",");
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.assertValue("#result", "2:A0,B1");
}

test "real world html: function can call global function declared later" {
    const html =
        \\<button id="btn">run</button>
        \\<div id="result">init</div>
        \\<script>
        \\  function openDialog() {
        \\    closePasteDialog();
        \\  }
        \\  function closePasteDialog() {
        \\    document.getElementById("result").textContent = "closed";
        \\  }
        \\  document.getElementById("btn").addEventListener("click", openDialog);
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.click("#btn");
    try subject.assertValue("#result", "closed");
}
