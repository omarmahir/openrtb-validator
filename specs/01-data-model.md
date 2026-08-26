# Spec 1 — Data model

Define structs for a minimal OpenRTB 2.x bid request using serde for JSON
deserialization.

## Scope
- `BidRequest`: id, imp, at, tmax, site (opt), app (opt), device (opt), user (opt)
- `Imp`: id, banner (opt), video (opt), bidfloor, bidfloorcur
- Use `Option<T>` for optional fields
- Add serde (with derive feature) and serde_json to Cargo.toml

## Acceptance
- `cargo build` succeeds
- A sample bid-request JSON string deserializes into the structs in a test in main
