# openrtb-validator

An OpenRTB 2.x bid-request validation service in Rust.

## Why I built this

I work in programmatic advertising, and wanted hands-on practice with Rust,
deploying to GCP, and an agent-driven development workflow, applied to a
problem from my own domain. This is a learning project, not production
infrastructure.

## Running locally

```
cargo build
cargo test
cargo run
```

`cargo run` binds `0.0.0.0` on the port given by the `PORT` environment
variable, defaulting to `8080`, and logs the bound address on startup:

```
listening on 0.0.0.0:8080
```

Set a different port with:

```
PORT=3000 cargo run
```

With the server running, hit it with curl:

```
curl localhost:8080/health
# {"status":"ok"}
```

## API

### `GET /health`

No dependencies checked. Always returns 200:

```
curl localhost:8080/health
```

```json
{"status":"ok"}
```

### `POST /validate`

Accepts a bid-request JSON body, runs it through `validate()`, and returns
the diagnostics.

- **200** — no error-severity diagnostics. `valid` is `true`. Warnings, if
  any, are still listed in `errors`.
- **422** — at least one error-severity diagnostic.
- **400** — the body is not valid JSON at all.

Warnings alone never produce a 422: a request with only warning-severity
diagnostics returns 200 with `valid: true` and a non-empty `errors` array.

**Valid request:**

```
curl -X POST localhost:8080/validate -H 'content-type: application/json' -d '{
  "id": "req-1",
  "imp": [
    { "id": "imp-1", "banner": { "w": 300, "h": 250 }, "bidfloor": 0.5, "bidfloorcur": "USD" }
  ],
  "site": { "id": "site-1", "domain": "example.com" },
  "cur": ["USD"]
}'
```

```json
{"valid":true,"errors":[]}
```

**Request with error-severity diagnostics (422):**

```
curl -X POST localhost:8080/validate -H 'content-type: application/json' -d '{"id":"","imp":[]}'
```

```json
{"valid":false,"errors":[{"code":"EmptyId","severity":"error","path":"id","message":"id must not be empty"},{"code":"EmptyImp","severity":"error","path":"imp","message":"imp must contain at least one object"},{"code":"MissingSiteAndApp","severity":"error","path":"","message":"exactly one of site or app must be present"}]}
```

**Warnings-only request (still 200):**

```
curl -X POST localhost:8080/validate -H 'content-type: application/json' -d '{
  "id": "req-1",
  "imp": [
    { "id": "imp-1", "banner": { "w": 300, "h": 250 }, "bidfloorcur": "ZZZ" }
  ],
  "site": { "id": "site-1" }
}'
```

```json
{"valid":true,"errors":[{"code":"UnknownBidFloorCur","severity":"warning","path":"imp[0].bidfloorcur","message":"bidfloorcur is well-formed but not a recognized ISO-4217 code"}]}
```

**Malformed body (400):**

```
curl -X POST localhost:8080/validate -H 'content-type: application/json' -d 'not json'
```

```json
{"valid":false,"errors":[{"code":"ParseError","severity":"error","path":"","message":"invalid JSON: expected ident at line 1 column 2"}]}
```

## Validation rules

| Code | Severity | Description |
|---|---|---|
| `ParseError` | error | The request body is not valid JSON |
| `EmptyId` | error | `id` must not be empty |
| `EmptyImp` | error | `imp` must contain at least one object |
| `EmptyImpId` | error | Each `imp.id` must not be empty |
| `DuplicateImpId` | error | Each `imp.id` must be unique within the request |
| `MissingSiteAndApp` | error | Exactly one of `site` or `app` must be present |
| `SiteAndAppBothPresent` | error | Exactly one of `site` or `app` must be present, not both |
| `MissingMediaType` | error | Each `imp` must have at least one of banner, video, native, or audio |
| `NegativeBidFloor` | error | `bidfloor`, when present, must be >= 0 |
| `InvalidBidFloorCur` | error | `bidfloorcur` must be a 3-letter uppercase ISO-4217-shaped code |
| `UnknownBidFloorCur` | warning | `bidfloorcur` is well-formed but not a recognized ISO-4217 code |
| `CurMismatch` | warning | `bidfloorcur` is not present in `cur` (or the implied `["USD"]` default) |

## Design notes

**`Option<T>` everywhere, not `#[serde(default)]`.** Nearly every field is
`Option<T>` rather than defaulted at deserialization. This lets validation
tell an omitted field apart from one explicitly set to its OpenRTB spec
default — `bidfloor: 0.0` sent explicitly is a different fact than
`bidfloor` absent, even though both would collapse to `0.0` under
`#[serde(default)]`.

**`imp: Vec<Imp>`, not `Option<Vec<Imp>>`.** A missing `imp` field fails to
deserialize at all, so it surfaces as a `ParseError`, not a validation
error. An empty `imp: []` deserializes fine and is caught by the `EmptyImp`
rule instead. This pushes the "field must be present" requirement into the
type system and leaves `validate()` to handle only "present but empty".

**Unknown-but-well-formed currency codes warn, not error.** A hardcoded
ISO-4217 list will always drift out of date as codes are added or retired.
Erroring on an unrecognized-but-correctly-shaped code would produce false
positives on legitimate traffic using a currency this list hasn't caught up
to, which is worse than missing the edge case, so it's a warning
(`UnknownBidFloorCur`) rather than an error.

**`cur` defaults to `["USD"]` in the validation layer, not at
deserialization.** `BidRequest.cur` stays `Option<Vec<String>>` through
deserialization; the `["USD"]` default is applied only when `validate()`
checks `bidfloorcur` against it. That way a request with `bidfloorcur: "EUR"`
and no `cur` field is still caught as a `CurMismatch` against the implied
default, rather than silently passing because there was nothing to compare
against.

## Built with Claude Code

Each piece of this project started as a written spec in `specs/` (data
model, validation rules, HTTP service, CI) before any code was written.
Architecture decisions and their rationale are recorded in `docs/adr`.

## Live deployment

Deployed at: _TBD (Cloud Run URL)_
