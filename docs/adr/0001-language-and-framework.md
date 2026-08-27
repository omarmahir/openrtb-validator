# 1. Language and framework for the bid-request validator

## Status

Accepted

## Context

This service sits in the request path of a programmatic advertising
pipeline: an HTTP handler (`src/main.rs`, `src/http.rs`) receives a raw
bid-request body, hands it to a pure validation function
(`validate(json: &str) -> Vec<ValidationError>` in `src/lib.rs`) that
deserializes it into a `BidRequest` struct tree with `serde`, runs a fixed
set of field-presence and range checks, and returns a list of typed
diagnostics with a severity, a code, and a JSON path. `POST /validate`
maps those diagnostics to 200, 422, or 400; `GET /health` is a static
200. There is no database call, no outbound network call, and no shared
mutable state — the entire cost of a request is JSON parsing plus rule
evaluation over already-in-memory data.

Two things about the domain shape this decision. First, real ad-exchange
traffic arrives at high queries-per-second with latency budgets in the
single-digit milliseconds, so tail latency and per-request overhead
matter more than developer throughput once the service is past initial
build-out. Second, the workload is CPU-bound and allocation-heavy (struct
trees per request) rather than I/O-bound, so the usual argument for a
GC'd, async-first language — that most latency is spent waiting on other
services — doesn't apply here.

## Decision

Use Rust with the `axum` web framework, on top of `tokio` and `serde`.

Weighed against Go: Go's collector has had sub-millisecond pause targets
since 1.8, and for most services in this shape that's a non-issue. The
sharper difference here is tail latency variance under sustained
allocation, not pause time — this handler builds a fresh struct tree per
request, and under high sustained QPS Go's concurrent collector competes
with request-handling goroutines for CPU and can impose GC-assist work on
allocating goroutines when the heap grows faster than it can keep up,
which shows up as p99/p999 jitter rather than a single stop-the-world
pause. Rust frees each request's struct tree deterministically as it goes
out of scope, with no background collector contending for CPU against
live request handling, which trades that jitter for slower iteration
speed and a smaller pool of engineers who can pick up the codebase
without ramp-up.

Weighed against Python/FastAPI: FastAPI would have been the fastest
framework to prototype the validation rules in, and Pydantic overlaps
significantly with what `serde` derives here. But an interpreted,
GC'd runtime carries per-request overhead (dict/object churn,
reference-count and GC pressure per parsed request) that is hard to get
under single-digit-millisecond tail latency at high volume without
dropping to a compiled extension for the hot path anyway — at which
point the Python layer is just a thin wrapper around compiled code, and
we may as well write the compiled code directly.

Within Rust, `axum` over `actix-web`: both perform comparably for this
kind of stateless JSON-in/JSON-out handler, but `axum` is built directly
on `tower` and `hyper`, which keeps the framework surface small (a
`Router`, extractors, `Service`) and lets the validation core stay a
plain function with zero framework types leaking into `src/lib.rs`, as
this project's own layering constraint requires. `actix-web` is not
meaningfully worse here; the choice is closer to a coin flip than the
Go/Python comparisons.

`ErrorCode` being a Rust enum also matters for this specific domain:
any future `match` over it — a status-code mapping, a metrics label, a
per-rule remediation hint — is checked for exhaustiveness at compile
time, so adding a new validation rule without updating every consumer of
`ErrorCode` fails the build instead of silently falling through. Neither
Go's untyped string/const error codes nor Python's lack of exhaustiveness
checking on enums (even with `match` and `Enum`) catches that.

## Consequences

Per-request latency and memory behavior are predictable, with no GC
pause and cheap enough allocation for this size of struct tree — the
property the domain most needed.

The cost is real. Rust's compile times are the slowest of the three
options by a wide margin, which slows the edit-test loop on every change
to `src/lib.rs`, including this project's own iteration on the rule set.
The hiring and onboarding pool for Rust is smaller than for Go or Python;
a contributor unfamiliar with the borrow checker and `Option`/`Result`
idioms will be slower to land changes than they would be in either
alternative. Async Rust specifically (`tokio`, `axum`'s extractor
traits) has a steeper learning curve than Go's goroutines or Python's
`async def`, and error messages from trait-bound mismatches in handler
signatures are frequently long and indirect. And because the validation
logic (`validate()`, `BidRequest` and friends) is plain synchronous Rust
with no framework dependency, most of axum's own value — routing,
extractors, middleware — is only exercised in the thin `src/http.rs`
layer; a simpler framework, or even a hand-rolled `hyper` server, would
have covered this service's actual surface area (two routes, one JSON
body) about as well. The choice is defensible for the latency profile
this service is meant to have in production, not obviously correct for
a service this small as it exists today.
