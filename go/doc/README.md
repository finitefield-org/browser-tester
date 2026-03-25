# browser-tester go

This directory is the Go rewrite workspace for `browser-tester`.
It is still a plan and specification set, but the phase 0 scaffold now exists and the module builds with skeleton tests.

The design follows the lessons captured in [`../next.md`](../../next.md) and [`../next-reflection.md`](../../next-reflection.md):

- keep `Harness` thin
- keep state in explicit subsystems
- make deterministic mocks first-class
- separate debug views from the public action surface
- keep configuration explicit instead of hiding seeds in unrelated state

## Document Index

- [Subsystem Map](subsystem-map.md)
- [Capability Matrix](capability-matrix.md)
- [Implementation Guide](implementation-guide.md)
- [Mock Guide](mock-guide.md)
- [Release Checklist](release-checklist.md)
- [Roadmap](roadmap.md)

## Core Rules

- Check `../html-standard/` before adding HTML, DOM, selector, or serialization behavior.
- Add a new public `Harness` method only after deciding whether it belongs on `Harness`, `DebugView`, or a mock family.
- Add a new test-only mock through the runtime registry, then expose it through the public facade.
- Prefer explicit configuration structs over hidden encodings or seed keys.
- Keep the Go implementation deterministic. Avoid background goroutines unless a subsystem explicitly requires them.

## Current Status

- Phase 0 scaffold is present, and internal DOM/runtime/script scaffolds have landed.
- The initial interaction slice (`Click`/`Focus`/`Blur`), the initial form-control slice (`TypeText`/`SetChecked`/`SetSelectValue`/`Submit`), and the initial assertion slice (`AssertText`/`AssertValue`/`AssertChecked`/`AssertExists`) are wired through the public facade and debug view, and `Click` also follows bounded hyperlink default actions for `a` / `area` elements and reset-button form reset through the location, open, download, and DOM form-control helpers. `DebugView` also exposes `DOMReady` and `DOMError` for DOM initialization readiness and parse/runtime failure inspection, plus `InitialHTML` for the original builder HTML input, plus `FocusedSelector`, `FocusedNodeID`, `TargetNodeID`, `HistoryLength`, `HistoryState`, `HistoryEntries`, `VisitedURLs`, `HistoryScrollRestoration`, `PendingTimers`, `PendingAnimationFrames`, `PendingMicrotasks`, `NavigationLog`, `Clipboard`, `MatchMediaRules`, `EventListeners`, `LocalStorage`, `SessionStorage`, `DocumentCookie`, `CookieJar`, and location parts (`LocationOrigin`, `LocationProtocol`, `LocationHost`, `LocationHostname`, `LocationPort`, `LocationPathname`, `LocationSearch`, `LocationHash`) for read-only observation of the current fragment target, history snapshot, visited URL snapshot, history stack, scheduled timer/animation-frame snapshots, queued microtask sources, clipboard text, match-media seed state, registered event listener snapshots, web-storage snapshot, navigation log, cookie string, cookie jar snapshot, and location decomposition. `DebugView.NodeCount()` exposes the current DOM node count after bootstrap as read-only inspection data, `DebugView.ScriptCount()` exposes the current script element count after bootstrap as read-only inspection data, `DebugView.ImageCount()` exposes the current image element count after bootstrap as read-only inspection data, `DebugView.FormCount()` exposes the current form element count after bootstrap as read-only inspection data, `DebugView.TableCount()` exposes the current table element count after bootstrap as read-only inspection data, `DebugView.ButtonCount()` exposes the current button element count after bootstrap as read-only inspection data, and `DebugView.TextAreaCount()` exposes the current textarea element count after bootstrap as read-only inspection data, and `DebugView.InputCount()` exposes the current input element count after bootstrap as read-only inspection data, `DebugView.FieldsetCount()` exposes the current fieldset element count after bootstrap as read-only inspection data, and `DebugView.LegendCount()` exposes the current legend element count after bootstrap as read-only inspection data, and `DebugView.OutputCount()` exposes the current output element count after bootstrap as read-only inspection data, and `DebugView.LabelCount()` exposes the current label element count after bootstrap as read-only inspection data, and `DebugView.ProgressCount()` exposes the current progress element count after bootstrap as read-only inspection data, and `DebugView.MeterCount()` exposes the current meter element count after bootstrap as read-only inspection data, and `DebugView.AudioCount()` / `DebugView.VideoCount()` expose the current audio and video element counts after bootstrap as read-only inspection data, `DebugView.IframeCount()` exposes the current iframe element count after bootstrap as read-only inspection data, `DebugView.EmbedCount()` exposes the current embed element count after bootstrap as read-only inspection data, and `DebugView.TrackCount()` exposes the current track element count after bootstrap as read-only inspection data. Inline `<script>` listeners can register capture/target/bubble handlers through the host bridge for the bounded event slice, can call `host:preventDefault()` to suppress click/reset default actions, can call `host:stopPropagation()` to stop later propagation, can opt into one-shot handling with a boolean `once` flag, can remove a previously registered handler with `host:removeEventListener()`, can queue bounded microtasks with `host:queueMicrotask()`, can schedule bounded timers with `host:setTimeout()` / `host:setInterval()` and `host:clearTimeout()` / `host:clearInterval()`, can schedule bounded animation-frame callbacks with `host:requestAnimationFrame()` / `host:cancelAnimationFrame()`, can observe the currently executing classic script through `host:documentCurrentScript()` and the explicit `expr(...)` wrapper in argument position, can drive the location mock through `host:locationAssign()` / `host:locationReplace()` / `host:locationReload()` / `host:locationSet()`, and can read location parts through `host:locationHref()` / `host:locationOrigin()` / `host:locationProtocol()` / `host:locationHost()` / `host:locationHostname()` / `host:locationPort()` / `host:locationPathname()` / `host:locationSearch()` / `host:locationHash()`, can drive a bounded `window.history` slice through `host:historyPushState()` / `host:historyReplaceState()` / `host:historyBack()` / `host:historyForward()` / `host:historyGo()` plus `host:historyLength()` / `host:historyState()` / `host:historyScrollRestoration()` / `host:historySetScrollRestoration()`, can read or write bounded `localStorage` / `sessionStorage` state through `host:localStorageGetItem()` / `host:localStorageSetItem()` / `host:localStorageRemoveItem()` / `host:localStorageClear()` / `host:localStorageLength()` / `host:localStorageKey()` and `host:sessionStorageGetItem()` / `host:sessionStorageSetItem()` / `host:sessionStorageRemoveItem()` / `host:sessionStorageClear()` / `host:sessionStorageLength()` / `host:sessionStorageKey()`, can use bounded DOM mutation helpers such as `host:setTextContent()` / `host:replaceChildren()` / `host:cloneNode()` / `host:setInnerHTML()` / `host:setOuterHTML()` / `host:insertAdjacentHTML()` / `host:removeNode()`, can read or write a bounded cookie jar through `host:documentCookie()` / `host:setDocumentCookie()` plus `host:navigatorCookieEnabled()`, and can read or write a bounded `window.name` state through `host:windowName()` / `host:setWindowName()`. Location URLs are resolved against the current URL just like navigation links, and history updates feed the same navigation log. It can also trigger bounded synthetic event helpers such as `Dispatch` and `DispatchKeyboard` for custom and keyboard event sequences, and it can query bounded `matchMedia` state through `MatchMedia()`. The `MatchMedia` mock family also exposes listener capture injection through the registry for tests, and the `Storage` mock family exposes deterministic change capture through `Events()` with ordered `seed` / `set` / `remove` / `clear` operations.
- `DebugView` also surfaces the builder failure seed readouts `OpenFailure`, `CloseFailure`, `PrintFailure`, and `ScrollFailure` as read-only inspection data.
- `DebugView.SelectCount()` exposes the current select element count as read-only inspection data.
- `DebugView.TemplateCount()` exposes the current template element count as read-only inspection data.
- `DebugView.PictureCount()` exposes the current picture element count as read-only inspection data.
- `DebugView.SourceCount()` exposes the current source element count as read-only inspection data.
- `DebugView.DialogCount()` exposes the current dialog element count as read-only inspection data.
- `DebugView.DetailsCount()` exposes the current details element count as read-only inspection data.
- `DebugView.SummaryCount()` exposes the current summary element count as read-only inspection data.
- `DebugView.SectionCount()` exposes the current section element count as read-only inspection data.
- `DebugView.MainCount()` exposes the current main element count as read-only inspection data.
- `DebugView.ArticleCount()` exposes the current article element count as read-only inspection data.
- `DebugView.NavCount()` exposes the current nav element count as read-only inspection data.
- `DebugView.AsideCount()` exposes the current aside element count as read-only inspection data.
- `DebugView.FigureCount()` exposes the current figure element count as read-only inspection data.
- `DebugView.FigcaptionCount()` exposes the current figcaption element count as read-only inspection data.
- `DebugView.HeaderCount()` exposes the current header element count as read-only inspection data.
- `DebugView.FooterCount()` exposes the current footer element count as read-only inspection data.
- `DebugView.AddressCount()` exposes the current address element count as read-only inspection data.
- `DebugView.BlockquoteCount()` exposes the current blockquote element count as read-only inspection data.
- `DebugView.ParagraphCount()` exposes the current paragraph element count as read-only inspection data.
- `DebugView.PreCount()` exposes the current pre element count as read-only inspection data.
- `DebugView.MarkCount()` exposes the current mark element count as read-only inspection data.
- `DebugView.QCount()` exposes the current q element count as read-only inspection data.
- `DebugView.CiteCount()` exposes the current cite element count as read-only inspection data.
- `DebugView.AbbrCount()` exposes the current abbr element count as read-only inspection data.
- `DebugView.StrongCount()` exposes the current strong element count as read-only inspection data.
- `DebugView.SpanCount()` exposes the current span element count as read-only inspection data.
- `DebugView.DataCount()` exposes the current data element count as read-only inspection data.
- `DebugView.DfnCount()` exposes the current dfn element count as read-only inspection data.
- `DebugView.KbdCount()` exposes the current kbd element count as read-only inspection data.
- `DebugView.SampCount()` exposes the current samp element count as read-only inspection data.
- `DebugView.RubyCount()` exposes the current ruby element count as read-only inspection data.
- `DebugView.RtCount()` exposes the current rt element count as read-only inspection data.
- `DebugView.VarCount()` exposes the current var element count as read-only inspection data.
- `DebugView.CodeCount()` exposes the current code element count as read-only inspection data.
- `DebugView.SmallCount()` exposes the current small element count as read-only inspection data.
- `DebugView.TimeCount()` exposes the current time element count as read-only inspection data.
- `DebugView.DOMReady()` and `DOMError()` expose DOM initialization readiness and the most recent DOM parse/runtime failure text as read-only inspection data.
- `DebugView.LastInlineScriptHTML()` exposes the most recently executed classic inline script outerHTML as read-only inspection data.
- `DebugView.HistoryEntries()` exposes the current history stack as a read-only inspection slice.
- `DebugView.HistoryIndex()` exposes the current history cursor as a read-only inspection integer.
- `DebugView.VisitedURLs()` exposes the current visited URL snapshot as a read-only inspection slice derived from the session history.
- `DebugView.PendingTimers()` and `PendingAnimationFrames()` expose scheduled timer and animation-frame snapshots as read-only inspection slices.
- `DebugView.PendingMicrotasks()` exposes queued microtask sources as a read-only inspection slice.
- `DebugView.OptionLabels()` exposes the current option labels as a read-only inspection slice.
- `DebugView.SelectedOptionLabels()` exposes the current selected option labels as a read-only inspection slice.
- `DebugView.OptionValues()` exposes the current option values as a read-only inspection slice.
- `DebugView.SelectedOptionValues()` exposes the current selected option values as a read-only inspection slice.
- `DebugView.OptionCount()` exposes the current option count as a read-only inspection integer.
- `DebugView.SelectedOptionCount()` exposes the current selected option count as a read-only inspection integer.
- `DebugView.OptgroupCount()` exposes the current optgroup count as a read-only inspection integer.
- `DebugView.LinkCount()` exposes the current link count as a read-only inspection integer.
- `DebugView.AnchorCount()` exposes the current anchor count as a read-only inspection integer.
- `DebugView.OptgroupLabels()` exposes the current optgroup labels as a read-only inspection slice.
- `DebugView.FetchCalls()` exposes the captured fetch call trace as a read-only inspection slice.
- `DebugView.CookieJar()` exposes the current cookie jar as a read-only inspection map.
- `DebugView.DialogAlerts()`, `DialogConfirmMessages()`, and `DialogPromptMessages()` expose captured dialog messages as read-only inspection slices.
- `DebugView.DownloadArtifacts()` and `FileInputSelections()` expose captured download and file-input traces as read-only inspection slices.
- `DebugView.StorageEvents()` exposes the captured storage change trace as a read-only inspection slice.
- `DebugView.OpenCalls()`, `CloseCalls()`, `PrintCalls()`, `ScrollCalls()`, and `MatchMediaCalls()` expose browser-action and matchMedia call traces as read-only inspection slices.
- `DebugView.MatchMediaListenerCalls()` exposes the captured matchMedia listener trace as a read-only inspection slice.
- `DebugView.ClipboardWrites()` exposes the captured clipboard write trace as a read-only inspection slice.
- `DebugView.EventListeners()` exposes the captured DOM event listener registrations as a read-only inspection slice.
- `DebugView.FetchResponseRules()` and `FetchErrorRules()` expose configured fetch response/error rules as read-only inspection slices.
- The selector engine now covers a bounded attribute selector slice (`[attr]`, `[attr=value]`, `[attr~=value]`, `[attr|=value]`, `[attr^=value]`, `[attr$=value]`, and `[attr*=value]`, plus bounded `i` / `s` flags on value operators) plus a bounded descendant/child/sibling combinator slice in addition to the simple tag/id/class forms, plus a bounded pseudo-class slice (`:root`, `:scope`, `:defined`, `:state(identifier)`, `:active`, `:hover`, `:empty`, `:checked`, `:indeterminate`, `:autofill`, `:-webkit-autofill`, `:default`, `:enabled`, `:disabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:valid`, `:invalid`, `:user-valid`, `:user-invalid`, `:in-range`, `:out-of-range`, `:first-child`, `:last-child`, `:first-of-type`, `:last-of-type`, `:only-child`, `:only-of-type`, `:nth-child()`, `:nth-of-type()`, `:nth-last-child()`, `:nth-last-of-type()`, `:link`, `:any-link`, `:visited`, `:local-link`, `:lang()`, `:dir()`, `:placeholder-shown`, `:blank`, `:heading`, `:heading(integer#)`, `:playing`, `:paused`, `:seeking`, `:buffering`, `:stalled`, `:muted`, `:volume-locked`, `:modal`, `:popover-open`, `:open`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, `:target-within`, `:is()`, `:where()`, `:not()`, and `:has()`). Document queries treat `:scope` as the document root scope, while element-level `Matches` and `Closest` use the element itself as scope; `:blank` is approximated for text-like inputs and textareas with empty or whitespace-only values, `:local-link` is approximated as a same-document link against the current session URL, and `:visited` is approximated against the current session history URLs. Custom element states are approximated through a tokenized `state` attribute on custom elements.
- Script DOM query helpers are available through host bindings for `querySelector` / `querySelectorAll` / `matches` / `closest`, `querySelectorAll` returns a minimal snapshot `NodeList`, a minimal live `HTMLCollection` covers `children`, `document.images`, `document.forms`, `form.elements`, `fieldset.elements`, `select.options`, `select.selectedOptions`, `datalist.options`, `table.rows`, `table.tBodies`, `HTMLTableSectionElement.rows`, `tr.cells`, `document.scripts`, `document.links`, and `document.anchors`, and bounded live `NodeList` slices cover `childNodes` and `template.content.childNodes`.
- Inline `<script>` blocks are preserved as raw text and execute during bootstrap through the bounded script host bridge, so source HTML can mutate the live DOM.
- Bounded attribute reflection helpers are available for `GetAttribute` / `HasAttribute` / `SetAttribute` / `RemoveAttribute`, and public live `ClassList` / `Dataset` views expose the same DOM slice through the facade.
- Internal bounded `classList` / `dataset` helpers still live in `internal/dom` and remain the source of truth for the live views.
- The public tree-mutation slice (`InnerHTML`, `TextContent`, `OuterHTML`, `SetInnerHTML`, `ReplaceChildren`, `CloneNode`, `SetTextContent`, `SetOuterHTML`, `InsertAdjacentHTML`, `RemoveNode`, `WriteHTML`) is wired through the facade; on `textarea`, content mutations that change its contents keep the reset default value in sync, `CloneNode()` duplicates the selected node and inserts the clone after the source, and `WriteHTML()` covers the bounded document-write-style replay slice with rollback of DOM and session state on failure.
- Bounded web-storage helpers for inline scripts are available through `host:localStorageGetItem()` / `host:localStorageSetItem()` / `host:localStorageRemoveItem()` / `host:localStorageClear()` / `host:localStorageLength()` / `host:localStorageKey()` and `host:sessionStorageGetItem()` / `host:sessionStorageSetItem()` / `host:sessionStorageRemoveItem()` / `host:sessionStorageClear()` / `host:sessionStorageLength()` / `host:sessionStorageKey()`, and storage mutations are captured as ordered `Events()` with explicit `seed` / `set` / `remove` / `clear` operations.
- Phase 5 hardening already includes seeded fuzz/property coverage for the script, selector, timer/scheduler, and location/history boundaries.
- Phase 5 hardening also includes seeded fuzz/property coverage for the cookie and `window.name` boundaries.
- Phase 5 hardening also includes seeded fuzz/property coverage for the mock registry boundaries.
- Phase 5 also has a repeatable release checklist in `release-checklist.md`.
- `go test ./...` passes for the current skeleton.
- The clipboard mock family scaffold is present, including `ReadClipboard` and `WriteClipboard`.
- Later phases remain intentionally bounded and future-facing.
- Keep this index aligned with the capability matrix and mock guide when the public surface changes.
- Do not implement legacy or deprecated spec branches unless they are required for a clearly bounded user-visible gap and are explicitly added to the capability matrix.

## Target Shape

- Public package: `browsertester`
- Internal packages: `internal/dom`, `internal/runtime`, `internal/script`, `internal/mocks`
- Public facade types: `Harness`, `HarnessBuilder`, `DebugView`, `MockRegistryView`, `Error`, `OptionLabel`, `OptionValue`, `OptgroupLabel`

## When Code Lands

The intended quick check is:

```bash
go test ./...
```
