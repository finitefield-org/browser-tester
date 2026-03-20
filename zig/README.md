# browser-tester zig

This directory is a clean-room Zig rewrite workspace for `browser-tester`.
The rewrite follows [`next.md`](../next.md) and keeps the public surface small while the internal phases are built out.

Current state:

- phase 0 scaffold plus the internal phase 1 DOM bootstrap slice, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice, the phase 4 deterministic mock and fake-clock slice, the phase 5 hardening suite, the phase 6 selector expansion slice, the phase 7 script DOM query and collection slices (`document.querySelector`, `document.querySelectorAll`, `element.querySelector`, `element.querySelectorAll`, `Element.matches`, and `Element.closest`), the phase 8 attribute reflection slice (`getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, and `toggleAttribute`), the phase 8 class and dataset views slice (`className`, `classList`, and `dataset`), the phase 8 tree mutation primitives slice (`appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`, `append`, `prepend`, `before`, `after`, and `remove`), and the phase 8 collection API broadening slice 2 (`document.scripts`)
- script-side HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) are available through inline scripts, with bounded fragment parsing on setters, deterministic serialization on getters, position-aware fragment insertion, `DocumentFragment`-style stringification for template content, and namespace-aware SVG / MathML name adjustments during serialization
- `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, and `Harness.dumpDom(...)` are available for inspection
- `Harness.nowMs(...)`, `Harness.advanceTime(...)`, `Harness.flush(...)`, `Harness.mocksMut(...)`, `Harness.fetch(...)`, `Harness.alert(...)`, `Harness.confirm(...)`, `Harness.prompt(...)`, `Harness.readClipboard(...)`, `Harness.writeClipboard(...)`, `Harness.captureDownload(...)`, `Harness.navigate(...)`, and `Harness.setFiles(...)` are available for deterministic runtime and mock control
- `Harness.click(...)`, `Harness.typeText(...)`, `Harness.setChecked(...)`, `Harness.setSelectValue(...)`, `Harness.focus(...)`, `Harness.blur(...)`, `Harness.submit(...)`, and `Harness.dispatch(...)` are available for user-like actions
- inline `<script>` bootstrapping runs during `Harness.fromHtml(...)` construction for the `document.getElementById(...).textContent = ...` slice, and script-side selector lookups can reuse the shared DOM selector engine through `querySelector`, `querySelectorAll`, `matches`, and `closest`, with minimal `NodeList` snapshots for collection queries including `forEach(callback[, thisArg])` and a live `document.scripts` HTMLCollection surface with `length`, `item(index)`, and `namedItem(name)`; attribute reflection methods update the shared DOM attribute store and keep selector and form-control views in sync, and `className` / `classList` / `dataset` stay aligned with the same store
- `Harness` and `HarnessBuilder` are available
- `Session` stays internal and owns the copied configuration state plus the DOM store, script runtime state, event listener registry, focus state, fake clock state, and mock registry
- `DomStore` builds, selects, serializes, and dumps HTML trees for tests, including class selectors and descendant/child combinators, but it is not part of the public API
- phase 8 HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) are available, including namespace-aware serialization compatibility; broader collection and selector grammar slices remain planned

## Quick Start

```bash
cd zig
zig build test
```

## Minimal Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<form id='profile'><input id='name'><input id='agree' type='checkbox'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value + ':' + String(document.getElementById('agree').checked); });</script>",
    );
    defer harness.deinit();

    try harness.typeText("#name", "Alice");
    try harness.click("#agree");
    try harness.click("#submit");
    try harness.assertChecked("#agree", true);
    try harness.assertValue("#out", "Alice:true");
}
```

## Mock Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(std.heap.page_allocator, "<main></main>");
    defer harness.deinit();

    try harness.mocksMut().fetch().respondText("https://app.local/api/message", 200, "ok");
    const response = try harness.fetch("https://app.local/api/message");
    try std.testing.expectEqualStrings("ok", response.body);
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().fetch().calls().len);

    try harness.captureDownload("report.csv", "downloaded bytes");
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().downloads().artifacts().len);
}
```

## Class/Dataset Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>document.getElementById('button').className = 'primary secondary'; document.getElementById('button').classList.add('active'); document.getElementById('button').dataset.userId = '42'; document.getElementById('out').textContent = document.getElementById('button').className + ':' + document.getElementById('button').dataset.userId;</script></main>",
    );
    defer harness.deinit();

    try harness.assertExists(".active");
    try harness.assertExists("[data-user-id]");
    try harness.assertValue("#out", "primary secondary active:42");
}
```

## Tree Mutation Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='root'><section id='target'></section><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button><div id='out'></div><script>document.getElementById('target').append(document.getElementById('first'), document.getElementById('second')); document.getElementById('target').prepend(document.getElementById('third')); document.getElementById('first').remove(); document.getElementById('out').textContent = document.getElementById('target').textContent + ':' + String(document.querySelectorAll('#target > button').length);</script></main>",
    );
    defer harness.deinit();

    try harness.assertValue("#out", "ThirdSecond:2");
    try harness.assertExists("#target > #third");
    try harness.assertExists("#target > #second");
}
```

## HTML Serialization Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section><div id='out'></div><script>document.getElementById('target').insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>'); document.getElementById('target').insertAdjacentHTML('afterbegin', '<span id=\"first\">One</span>'); document.getElementById('target').insertAdjacentHTML('beforeend', '<span id=\"second\">Two</span>'); document.getElementById('target').insertAdjacentHTML('afterend', '<aside id=\"after\">After</aside>'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + document.getElementById('target').innerHTML + '|' + String(document.querySelectorAll('#target > span').length);</script></main>",
    );
    defer harness.deinit();

    try harness.assertValue(
        "#out",
        "<aside id=\"before\">Before</aside><section id=\"target\"><span id=\"first\">One</span><button class=\"primary\" id=\"old\">Old</button><span id=\"second\">Two</span></section><aside id=\"after\">After</aside>|<span id=\"first\">One</span><button class=\"primary\" id=\"old\">Old</button><span id=\"second\">Two</span>|2",
    );
    try harness.assertExists("#before");
    try harness.assertExists("#after");
    try harness.assertExists("#target > #first");
    try harness.assertExists("#target > #second");
}
```

## Template Content Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>document.getElementById('out').textContent = String(document.getElementById('tpl').content) + '|' + document.getElementById('tpl').content.innerHTML; document.getElementById('tpl').content.innerHTML = '<span id=\"second\">Second</span>'; document.getElementById('out').textContent += '|' + document.getElementById('tpl').content.innerHTML;</script>",
    );
    defer harness.deinit();

    try harness.assertValue(
        "#out",
        "[object DocumentFragment]|<span id=\"inner\">Inner</span>|<span id=\"second\">Second</span>",
    );
    try harness.assertExists("#second");
}
```

## Public Surface

- `Harness`
- `HarnessBuilder`
- `StorageSeed`
- `MockRegistry`
- `FetchMocks`
- `FetchResponseRule`
- `FetchErrorRule`
- `FetchCall`
- `FetchResponse`
- `DialogMocks`
- `ClipboardMocks`
- `LocationMocks`
- `DownloadMocks`
- `DownloadCapture`
- `FileInputMocks`
- `FileInputSelection`
- `StorageSeeds`
- `Error`
- `Result(T)`
- `Error` currently includes `InvalidUrl`, `InvalidSelector`, `AssertionFailed`, `DomError`, `EventError`, `ScriptParse`, `ScriptRuntime`, `HtmlParse`, `MockError`, `TimerError`, and `OutOfMemory`
- `Harness.assertExists(selector)`
- `Harness.assertValue(selector, expected)`
- `Harness.assertChecked(selector, expected)`
- `Harness.nowMs()`
- `Harness.advanceTime(delta_ms)`
- `Harness.flush()`
- `Harness.mocksMut()`
- `Harness.fetch(url)`
- `Harness.alert(message)`
- `Harness.confirm(message)`
- `Harness.prompt(message)`
- `Harness.readClipboard()`
- `Harness.writeClipboard(text)`
- `Harness.captureDownload(file_name, bytes)`
- `Harness.navigate(url)`
- `Harness.setFiles(selector, files)`
- `Harness.click(selector)`
- `Harness.typeText(selector, text)`
- `Harness.setChecked(selector, checked)`
- `Harness.setSelectValue(selector, value)`
- `Harness.focus(selector)`
- `Harness.blur(selector)`
- `Harness.submit(selector)`
- `Harness.dispatch(selector, event_type)`
- `Harness.dumpDom(allocator)`

## Docs

- [Architecture](doc/architecture.md)
- [Capability Matrix](doc/capability-matrix.md)
- [Implementation Guide](doc/implementation-guide.md)
- [Subsystem Map](doc/subsystem-map.md)
- [Mock Guide](doc/mock-guide.md)
- [Limitations](doc/limitations.md)
- [Roadmap](doc/roadmap.md)
- [Publish Checklist](doc/publish-checklist.md)
