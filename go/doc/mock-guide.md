# Mock Guide

Mocks are part of the intended core surface for the Go workspace.
They are not an implementation detail, and they are not a loose bag of `Set*` helpers.

`Harness.Mocks()` should return a typed `MockRegistryView` with family objects. Use it when a test needs deterministic network, dialogs, clipboard, location, open/close/print/scroll, matchMedia, download, file-input, or storage behavior.

## Current Mock Families

- `Fetch`
- `Dialogs`
- `Clipboard`
- `Location`
- `Open`
- `Close`
- `Print`
- `Scroll`
- `MatchMedia`
- `Downloads`
- `FileInput`
- `Storage`

## Public Mock Actions

The public `Harness` surface should stay thin and expose only the user-like actions that need to route through these families:

- `Fetch`
- `Alert`
- `Confirm`
- `Prompt`
- `ReadClipboard`
- `WriteClipboard`
- `Open`
- `Close`
- `Print`
- `ScrollTo`
- `ScrollBy`
- `Navigate`
- `SetFiles`
- `CaptureDownload`

`MatchMedia` is configured through the builder or registry and consumed from scripts via `window.matchMedia(...)`.
It is intentionally not a separate `Harness` action.

## Design Rules Per Family

Each family should support the following where it makes sense:

- explicit seed state
- failure injection where applicable
- call capture or artifact capture
- reset semantics

Examples:

- `Fetch`: response rules, error rules, request call capture
- `Dialogs`: queued confirm/prompt answers, alert capture, and message capture
- `Clipboard`: seeded read state and write capture
- `Location`: current URL seed and navigation capture, including `window.location.assign()`, `window.location.replace()`, `window.location.reload()`, and URL-property assignments; inline scripts drive the same family through `host:locationAssign(...)`, `host:locationReplace(...)`, `host:locationReload()`, and `host:locationSet(property, value)`, and navigation URLs are resolved against the current URL
- `Downloads`: artifact capture through the registry and `Harness.CaptureDownload(...)`, plus hyperlink clicks on `a` / `area` elements with `download` attributes
- `FileInput`: file selection capture and the `input.files` snapshot
- `Open`: call capture and optional bootstrap failure
- `Close`: call capture and optional bootstrap failure
- `Print`: call capture and optional bootstrap failure
- `Scroll`: call capture and optional bootstrap failure
- `MatchMedia`: query seed state, failure injection, call capture, and listener call capture
- `Storage`: explicit local/session seeds plus deterministic `storage` event dispatch

## Capture Rules

- Call capture should be append-only.
- Artifact capture should preserve the order in which it was produced.
- Listener capture should be separate from query capture.
- Returned slices are read-only views. Callers should not mutate them.

## Minimal Example

```go
package main

import "browsertester"

func main() error {
	h, err := browsertester.FromHTML("<input id='upload' type='file'>")
	if err != nil {
		return err
	}

	mocks := h.Mocks()
	mocks.Fetch().RespondText("https://app.local/api/message", 200, "ok")
	mocks.Dialogs().QueueConfirm(true)
	mocks.Clipboard().SeedText("copied text")

	resp, err := h.Fetch("https://app.local/api/message")
	if err != nil {
		return err
	}
	_ = resp

	if _, err := h.Confirm("Continue?"); err != nil {
		return err
	}
	if _, err := h.ReadClipboard(); err != nil {
		return err
	}
	if err := h.SetFiles("#upload", []string{"report.csv"}); err != nil {
		return err
	}
	if err := h.CaptureDownload("report.csv", []byte("downloaded bytes")); err != nil {
		return err
	}
	return nil
}
```

## Failure Example

```go
package main

import (
	"fmt"

	"browsertester"
)

func main() error {
	h, err := browsertester.NewHarnessBuilder().
		PrintFailure("print blocked").
		Build()
	if err != nil {
		return err
	}

	if err := h.Print(); err == nil {
		return fmt.Errorf("expected print failure")
	}
	return nil
}
```

Other failure cases the workspace should keep covered:

- a fetch call with no matching fetch rule
- a confirm or prompt call with an empty queue
- a clipboard read with no seed
- an unseeded `matchMedia(...)` query
- builder-seeded `Open`, `Close`, `Print`, `ScrollTo`, and `ScrollBy` failures, with the same seeds later available to bootstrap-time window bindings

## Adding a New Test-Only Mock

When a new mock family is added, the change set must include:

- a public API addition or update
- a minimal usage example
- failure coverage, including at least one negative case
- call capture or artifact capture documentation
- `go/doc/capability-matrix.md` update
- `go/doc/mock-guide.md` update
- `go/doc/README.md` update
- tests that cover the public contract and the regression case

Do not bypass the registry when wiring a new mock.
Add it in runtime, expose it through the facade, and keep the family typed.
Legacy and deprecated spec branches are not mock targets unless the capability matrix explicitly lists a bounded compatibility exception.
