# browser-tester zig

This directory is a clean-room Zig rewrite workspace for `browser-tester`.
The rewrite follows [`next.md`](../next.md) and keeps the public surface small while the internal phases are built out.

Current state:

- phase 0 scaffold plus the internal phase 1 DOM bootstrap slice, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice, the phase 4 deterministic mock and fake-clock slice, the phase 5 hardening suite, the phase 6 selector expansion slice, the phase 6 `:scope` pseudo-class slice, the phase 6 `:has(...)` pseudo-class slice, the phase 6 `:lang(...)` / `:dir(...)` pseudo-class slice, the phase 6 bounded structural/state pseudo-class slice (`:root`, `:empty`, `:first-child`, `:last-child`, `:only-child`, `:first-of-type`, `:last-of-type`, `:only-of-type`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:link`, `:any-link`, `:placeholder-shown`, `:indeterminate`, `:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, `:read-only`, and `:read-write`), the phase 6 `:defined` pseudo-class slice, the `:not(...)` / `:is(...)` / `:where(...)` selector-list pseudo-class slice, the focus/target/nth pseudo-class slice (`:focus`, `:focus-within`, `:target`, and bounded `:nth-*` forms including `of <selector-list>` support on `:nth-child`, `:nth-last-child`, `:nth-of-type`, and `:nth-last-of-type`), the phase 7 script DOM query and collection slices (`document.querySelector`, `document.querySelectorAll`, `element.querySelector`, `element.querySelectorAll`, `Element.matches`, and `Element.closest`), the phase 8 attribute reflection slice (`getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, and `toggleAttribute`), the phase 8 class and dataset views slice (`className`, `classList`, and `dataset`), the phase 8 inline style declaration slice (`style`, `cssText`, `getPropertyValue`, `setProperty`, `removeProperty`, `length`, and `item`), the phase 8 tree mutation primitives slice (`appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`, `append`, `prepend`, `before`, `after`, and `remove`), and the phase 8 collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `entries()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements`), 8 (`select.options`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`), 19 (`entries()` helpers across `NodeList`, `HTMLCollection`, `StyleSheetList`, and `RadioNodeList`), and 20 (`select.options.add()` / `select.options.remove()`), plus sibling combinators (`A + B`, `A ~ B`) and the limited `Location` host object (`href`, `assign()`, `replace()`, `reload()`), plus the limited `History` host object (`length`, `state`, `back()`, `forward()`, `go(delta)`, `pushState(state, title, url)`, and `replaceState(state, title, url)`), plus deterministic viewport metadata (`window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, and `window.outerHeight`) and visibility aliases (`document.visibilityState`, `document.hidden`, and `document.hasFocus()`)
- deterministic screen-position aliases are fixed at `0` through `window.screenX`, `window.screenY`, `window.screenLeft`, and `window.screenTop`
- `window.screen` is available as a deterministic read-only host object with fixed `width`, `height`, `availWidth`, `availHeight`, `availLeft`, `availTop`, `left`, `top`, `colorDepth`, and `pixelDepth`
- script-side document and window alias surfaces (`document.documentElement`, `document.head`, `document.body`, `document.activeElement`, `document.defaultView`, `document.title`, `document.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.visibilityState`, `document.hidden`, `document.origin`, `document.hasFocus()`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.closed`, `window.children`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `javaEnabled()`), `window.performance` (`now()` / `timeOrigin`), `window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, `window.outerHeight`, `window.screenX`, `window.screenY`, `window.screenLeft`, `window.screenTop`, `window.name`, `window.title`, `window.location`, `window.origin`, and `Element.baseURI` / `Element.origin`) are available through inline scripts and stay wired into the copied session state; the viewport metrics are deterministic constants, the screen-position aliases are fixed at `0`, `window.performance.now()` is backed by the fake clock, `window.performance.timeOrigin` is deterministic, `document.visibilityState` is `visible`, `document.hidden` is `false`, and `document.hasFocus()` is deterministic
- script-side storage surfaces (`window.localStorage` and `window.sessionStorage`) are available through inline scripts and stay wired into the copied session state through `HarnessBuilder.addLocalStorage(...)` and `HarnessBuilder.addSessionStorage(...)`; convenience constructors `fromHtmlWithSessionStorage(...)` and `fromHtmlWithUrlAndSessionStorage(...)` are also available
- script-side `window.open()` / `window.close()` / `window.print()` / `window.scrollTo()` / `window.scrollBy()` are wired into the deterministic mock families; `HarnessBuilder.openFailure(...)`, `HarnessBuilder.closeFailure(...)`, `HarnessBuilder.printFailure(...)`, and `HarnessBuilder.scrollFailure(...)` can seed bootstrap failures for inline scripts
- script-side HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) are available through inline scripts, with bounded fragment parsing on setters, deterministic serialization on getters, position-aware fragment insertion, `DocumentFragment`-style stringification for template content, and namespace-aware SVG / MathML name adjustments during serialization
- script-side `document.location` / `window.location` are available as `Location` host objects with `href`, `assign()`, `replace()`, and `reload()`, and still coerce to the current URL string during inline script evaluation; `window.history` is available as a limited `History` host object with `length`, `state`, `back()`, `forward()`, `go(delta)`, `pushState(state, title, url)`, and `replaceState(state, title, url)`, while `history.state` keeps a minimal payload snapshot (`null` / `undefined` stay `null`, and other values are stringified)
- the selector engine also accepts the universal selector `*` in the internal DOM layer and script-side query APIs
- `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, and `Harness.dumpDom(...)` are available for inspection
- `Harness.nowMs(...)`, `Harness.advanceTime(...)`, `Harness.flush(...)`, `Harness.mocksMut(...)`, `Harness.fetch(...)`, `Harness.alert(...)`, `Harness.confirm(...)`, `Harness.prompt(...)`, `Harness.readClipboard(...)`, `Harness.writeClipboard(...)`, `Harness.captureDownload(...)`, `Harness.open(...)`, `Harness.close()`, `Harness.print()`, `Harness.scrollTo(...)`, `Harness.scrollBy(...)`, `Harness.navigate(...)`, and `Harness.setFiles(...)` are available for deterministic runtime and mock control, including fetch, dialogs, clipboard, open, close, print, scroll, location, matchMedia, downloads, file-input, and storage families; inline scripts can schedule microtasks with `queueMicrotask()` / `window.queueMicrotask()` and timers with `setTimeout()` / `window.setTimeout()` / `clearTimeout()` / `window.clearTimeout()` plus repeating timers with `setInterval()` / `window.setInterval()` / `clearInterval()` / `window.clearInterval()`, `window.performance.now()` reads from the fake clock, and `advanceTime()` / `flush()` drive due timers plus queued microtasks
- `Harness.click(...)`, `Harness.typeText(...)`, `Harness.setChecked(...)`, `Harness.setSelectValue(...)`, `Harness.focus(...)`, `Harness.blur(...)`, `Harness.submit(...)`, and `Harness.dispatch(...)` are available for user-like actions
- inline `<script>` bootstrapping runs during `Harness.fromHtml(...)` construction for the `document.getElementById(...).textContent = ...` slice, and script-side selector lookups can reuse the shared DOM selector engine through `querySelector`, `querySelectorAll`, `matches`, and `closest`, including sibling combinators (`A + B`, `A ~ B`), the `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-within`, `:target`, and bounded `:nth-*` pseudo-classes including `of <selector-list>` support on the `nth-child`, `nth-last-child`, `nth-of-type`, and `nth-last-of-type` families, and the bounded structural/state pseudo-class slice (`:root`, `:empty`, `:first-child`, `:last-child`, `:only-child`, `:first-of-type`, `:last-of-type`, `:only-of-type`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:link`, `:any-link`, and `:placeholder-shown`), plus the `:defined` pseudo-class slice, while inline bootstrap also exposes `document.currentScript`, `document.readyState`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.activeElement`, `document.defaultView`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.closed`, and `window.children`, with minimal `NodeList` snapshots for collection queries including `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, plus live `document.scripts` / `document.anchors` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, and `entries()`, live `document.forms`, `form.elements`, `fieldset.elements`, `datalist.options`, `map.areas`, and `table.tBodies` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `element.labels` NodeList support on labelable form controls and `fieldset` with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()` that reflects explicit `label[for]` associations and implicit ancestor labels in tree order, live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `document.styleSheets` StyleSheetList support with `length`, `item(index)`, `keys()`, `values()`, and `entries()`, live `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `getElementsByTagName`, `getElementsByTagNameNS`, and `getElementsByClassName` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, plus live `getElementsByName` NodeList support with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()` that reflects descendant elements in tree order, where `form.elements.namedItem(name)` returns a `RadioNodeList` when multiple matching controls share a name, live `select.options` and `select.selectedOptions` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, and live `select.options.add()` / `select.options.remove()` helpers, and live child-element / child-node surfaces on `Element`, `Document`, and `template.content` with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`; attribute reflection methods update the shared DOM attribute store and keep selector and form-control views in sync, and `className` / `classList` / `dataset` stay aligned with the same store
- `form.elements.namedItem(name)` can surface `RadioNodeList` objects for multi-match groups, and `RadioNodeList.value` is writable; assigning a matching radio value checks the first matching radio, and assigning a missing value clears the group
- inline `<script>` bootstrapping also exposes the minimal `Element.style` / `CSSStyleDeclaration` surface for simple declaration lists with comment stripping and `!important` priority handling, including `cssText`, `getPropertyValue(...)`, `getPropertyPriority(...)`, `setProperty(...)`, `removeProperty(...)`, `length`, `item(index)`, and property reflection through `style.someProperty = ...`
- `Harness` and `HarnessBuilder` are available, and `HarnessBuilder` can capture URL, HTML, local storage, session storage, and open/close/print/scroll bootstrap failure seeds
- `Session` stays internal and owns the copied configuration state plus the DOM store, which carries focus and target selector-state snapshots, script runtime state, event listener registry, queued microtasks, fake clock state, and mock registry
- `DomStore` builds, selects, serializes, and dumps HTML trees for tests, including class selectors, descendant/child/sibling combinators, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-within`, `:target`, bounded `:nth-*` forms, the bounded structural/state pseudo-class slice, and `:defined`, but it is not part of the public API
- inline scripts can also read the document/window alias surfaces (`document.documentElement`, `document.head`, `document.body`, `document.activeElement`, `document.title`, `document.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.origin`, `window.children`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `javaEnabled()`), `window.performance` (`now()` / `timeOrigin`), `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, `window.name`, `window.title`, `window.location`, `window.origin`, `Element.baseURI`, and `Element.origin`), but those bindings remain internal to the script runtime and are not part of the public `Harness` API; `document.location` / `window.location` are `Location` host objects with `href`, `assign()`, `replace()`, and `reload()`, `window.performance` is a deterministic clock-backed host object with `now()` / `timeOrigin`, and `window.history` is a limited `History` host object with `length`, `state`, `back()`, `forward()`, `go(delta)`, `pushState(state, title, url)`, and `replaceState(state, title, url)`, while `history.state` keeps a minimal payload snapshot (`null` / `undefined` stay `null`, and other values are stringified)
- phase 8 HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) are available, including namespace-aware serialization compatibility; broader CSS parsing beyond the bounded selector engine remains deferred until a specific user-visible gap needs it

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

    try harness.open("https://app.local/popup");
    try harness.close();
    try harness.print();
    try harness.scrollTo(10, 20);
    try harness.scrollBy(-5, 3);
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().open().calls().len);
    try std.testing.expectEqualStrings(
        "https://app.local/popup",
        harness.mocksMut().open().calls()[0].url.?,
    );
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().close().calls().len);
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().print().calls().len);
    try std.testing.expectEqual(@as(usize, 2), harness.mocksMut().scroll().calls().len);
}
```

Use `HarnessBuilder.openFailure(...)`, `HarnessBuilder.closeFailure(...)`, `HarnessBuilder.printFailure(...)`, and `HarnessBuilder.scrollFailure(...)` when you want inline `window.open()` / `window.close()` / `window.print()` / `window.scrollTo()` / `window.scrollBy()` calls to fail during bootstrap with `error.MockError`.

## MatchMedia Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); document.getElementById('out').textContent = String(mql) + ':' + String(mql.matches); });</script>",
    );
    defer harness.deinit();

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try harness.click("#toggle");
    try harness.assertValue("#out", "[object MediaQueryList]:true");
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().matchMedia().calls().len);
}
```

## Storage Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var builder = bt.Harness.builder(std.heap.page_allocator);
    defer builder.deinit();

    _ = builder.html("<main id='out'></main><script>const local = window.localStorage; const session = window.sessionStorage; local.setItem('theme', 'dark'); session.setItem('scratch', 'xyz'); document.getElementById('out').textContent = local.getItem('token') + ':' + session.getItem('session-token') + '|' + local.getItem('theme') + ':' + session.getItem('scratch');</script>");
    try builder.addLocalStorage("token", "abc");
    try builder.addSessionStorage("session-token", "seed");

    var harness = try builder.build();
    defer harness.deinit();

    try harness.assertValue("#out", "abc:seed|dark:xyz");
    try std.testing.expectEqualStrings("dark", harness.mocksMut().storage().local().get("theme").?);
    try std.testing.expectEqualStrings("xyz", harness.mocksMut().storage().session().get("scratch").?);
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

## Inline Style Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='root'><div id='box' style='color: red; background-color: white;'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = box.style; style.backgroundColor = 'blue'; style.setProperty('border-top-width', '2px'); style.removeProperty('color'); document.getElementById('out').textContent = String(style.cssText) + ':' + style.getPropertyValue('background-color') + ':' + String(style.length) + ':' + style.item(0);</script></main>",
    );
    defer harness.deinit();

    try harness.assertValue(
        "#out",
        "background-color: blue; border-top-width: 2px;:blue:2:background-color",
    );
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
- `OpenCall`
- `OpenMocks`
- `CloseCall`
- `CloseMocks`
- `PrintCall`
- `PrintMocks`
- `ScrollMethod`
- `ScrollCall`
- `ScrollMocks`
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
- `HarnessBuilder.openFailure(message)`
- `HarnessBuilder.closeFailure(message)`
- `HarnessBuilder.printFailure(message)`
- `HarnessBuilder.scrollFailure(message)`
- `Harness.fetch(url)`
- `Harness.alert(message)`
- `Harness.confirm(message)`
- `Harness.prompt(message)`
- `Harness.readClipboard()`
- `Harness.writeClipboard(text)`
- `Harness.captureDownload(file_name, bytes)`
- `Harness.open(url)`
- `Harness.close()`
- `Harness.print()`
- `Harness.scrollTo(x, y)`
- `Harness.scrollBy(x, y)`
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
