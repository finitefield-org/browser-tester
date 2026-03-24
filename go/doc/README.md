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
- [Roadmap](roadmap.md)

## Core Rules

- Check `../html-standard/` before adding HTML, DOM, selector, or serialization behavior.
- Add a new public `Harness` method only after deciding whether it belongs on `Harness`, `DebugView`, or a mock family.
- Add a new test-only mock through the runtime registry, then expose it through the public facade.
- Prefer explicit configuration structs over hidden encodings or seed keys.
- Keep the Go implementation deterministic. Avoid background goroutines unless a subsystem explicitly requires them.

## Current Status

- Phase 0 scaffold is present, and internal DOM/runtime/script scaffolds have landed.
- The initial interaction slice (`Click`/`Focus`/`Blur`), the initial form-control slice (`TypeText`/`SetChecked`/`SetSelectValue`/`Submit`), and the initial assertion slice (`AssertText`/`AssertValue`/`AssertChecked`/`AssertExists`) are wired through the public facade and debug view, and `Click` also follows bounded hyperlink default actions for `a` / `area` elements and reset-button form reset through the location, open, download, and DOM form-control helpers. Inline `<script>` listeners can register target-phase handlers through the host bridge, and those handlers can chain bounded `host:` statements separated by `;`.
- The selector engine now covers a bounded descendant/child combinator slice in addition to the simple tag/id/class forms.
- Script DOM query helpers are available through host bindings for `querySelector` / `querySelectorAll` / `matches` / `closest`, `querySelectorAll` returns a minimal snapshot `NodeList`, and a minimal live `HTMLCollection` covers `children`.
- Inline `<script>` blocks are preserved as raw text and execute during bootstrap through the bounded script host bridge, so source HTML can mutate the live DOM.
- Bounded attribute reflection helpers are available for `GetAttribute` / `HasAttribute` / `SetAttribute` / `RemoveAttribute`; explicit `classList` and `dataset` views remain later.
- Internal bounded `classList` / `dataset` helpers now live in `internal/dom`; the public facade will stay narrow until they are clearly needed.
- The public tree-mutation slice (`InnerHTML`, `OuterHTML`, `SetInnerHTML`, `SetOuterHTML`, `InsertAdjacentHTML`, `RemoveNode`) is wired through the facade; `cloneNode` remains internal for now.
- `go test ./...` passes for the current skeleton.
- The clipboard mock family scaffold is present, including `ReadClipboard` and `WriteClipboard`.
- Later phases remain intentionally bounded and future-facing.
- Keep this index aligned with the capability matrix and mock guide when the public surface changes.
- Do not implement legacy or deprecated spec branches unless they are required for a clearly bounded user-visible gap and are explicitly added to the capability matrix.

## Target Shape

- Public package: `browsertester`
- Internal packages: `internal/dom`, `internal/runtime`, `internal/script`, `internal/mocks`
- Public facade types: `Harness`, `HarnessBuilder`, `DebugView`, `MockRegistryView`, `Error`

## When Code Lands

The intended quick check is:

```bash
go test ./...
```
