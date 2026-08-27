# Spec 3 — HTTP service

Wrap the existing `validate()` in an axum web service.

## Endpoints

### POST /validate
Accepts a bid-request JSON body. Runs `validate()`. Returns:

```json
{ "valid": true, "errors": [] }
```

Status codes:
- 200 when there are no Error-severity diagnostics (warnings may still be present)
- 422 when at least one Error-severity diagnostic is present
- 400 when the body is not valid JSON at all

Each entry in `errors` serialises as:

```json
{ "code": "InvalidBidFloorCur", "severity": "error",
  "path": "imp[0].bidfloorcur", "message": "..." }
```

### GET /health
Returns 200 with body `{"status":"ok"}`. No dependencies checked.

## Configuration
- Bind port read from the `PORT` environment variable, defaulting to 8080.
- Bind address 0.0.0.0, not 127.0.0.1 (required for containers).

## Constraints
- Do not modify `validate()` or the model structs, except to add
  `Serialize` derives where needed for the response.
- Handlers must not panic. A malformed body is a 400, not a 500.
- Accept the body as a raw string and pass it to `validate()`. Do not
  use axum's `Json<T>` extractor for the bid request — that would reject
  malformed JSON before `validate()` can produce a ParseError diagnostic.
- Keep server code in `src/main.rs`. The library stays transport-agnostic.

## Acceptance
- `cargo run` starts the server and logs the bound address
- `curl localhost:8080/health` returns 200
- A valid bid request POSTed to /validate returns 200 and `"valid": true`
- `{"id":"","imp":[]}` returns 422 with EmptyId and EmptyImp in errors
- `not json` returns 400
- Integration tests in `tests/` covering each of the above
