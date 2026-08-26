use openrtb_validator::{validate, ErrorCode, Severity};

#[test]
fn valid_request_from_outside_the_crate() {
    let json = r#"{"id":"a","imp":[{"id":"1","banner":{"format":[{"w":300,"h":250}]}}],"site":{"page":"https://example.com"}}"#;
    assert!(validate(json).is_empty());
}

#[test]
fn eur_floor_without_cur_warns_against_implied_usd() {
    let json = r#"{"id":"a","imp":[{"id":"1","banner":{},"bidfloorcur":"EUR"}],"site":{}}"#;
    let out = validate(json);
    assert!(out.iter().any(|e| e.code == ErrorCode::CurMismatch
        && e.severity == Severity::Warning));
}

#[test]
fn native_only_imp_is_valid() {
    let json = r#"{"id":"a","imp":[{"id":"1","native":{"request":"{}"}}],"app":{"bundle":"com.x"}}"#;
    assert!(validate(json).is_empty());
}
