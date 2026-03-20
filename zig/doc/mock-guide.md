# Mock Guide

Mocks are reserved as a first-class part of the Zig rewrite, but no public mock family is exposed yet.
The workspace is already shaped so that a future `Session` can own a typed mock registry without turning `Harness` into a bag of `set_*` methods.

## Reserved Families

- `fetch`
- `dialogs`
- `clipboard`
- `location`
- `downloads`
- `file_input`
- `storage`

## Capture Model

When a family is promoted to a public test-only mock, it should follow one of these patterns:

- call capture: record the inputs that were requested
- artifact capture: record the outputs or side effects that a test needs to inspect later

Examples:

- `fetch`: call capture plus response and failure injection
- `dialogs`: call capture plus queued responses
- `clipboard`: read seeding plus write capture
- `location`: navigation call capture
- `downloads`: artifact capture
- `file_input`: selection capture
- `storage`: seed state plus deterministic reads

## Promotion Checklist

When a mock family becomes public, the same change should include:

- a public API addition or update
- a minimal usage example
- success and failure tests where applicable
- a clear description of call capture or artifact capture
- `README.md` updates
- this guide updated in the same change

## Current State

Do not expect runtime mock behavior from this workspace yet.
The mock guide exists now so future mock families can be added with the same contract discipline from the start.

