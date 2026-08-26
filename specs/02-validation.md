# Spec 2 — Validation

Add `pub fn validate(json: &str) -> Vec<ValidationError>` to the library.
Returns an empty vec for a valid request. Never panics.

## ValidationError
Struct or enum carrying: a machine-readable code, a human-readable message,
and a JSON path to the offending field (e.g. `imp[0].bidfloorcur`).

## Rules
- Malformed JSON returns a single parse error, not a panic
- `id` must be non-empty
- `imp` must contain at least one object
- each `imp.id` must be non-empty and unique within the request
- exactly one of `site` / `app` must be present
- each `imp` must have at least one media type (banner or video)
- `bidfloor`, when present, must be >= 0
- `bidfloorcur`, when present, must be a valid ISO-4217 code
- warn when `bidfloorcur` is not present in `cur`

## Acceptance
- a valid request returns an empty vec
- one test per rule, each asserting the specific error code and path
