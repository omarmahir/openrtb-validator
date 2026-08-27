use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod http;
pub use http::app;

#[derive(Debug, Deserialize)]
pub struct BidRequest {
    pub id: String,
    pub imp: Vec<Imp>,
    pub at: Option<i32>,
    pub tmax: Option<i64>,
    pub site: Option<Site>,
    pub app: Option<App>,
    pub device: Option<Device>,
    pub user: Option<User>,
    pub cur: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Imp {
    pub id: String,
    pub banner: Option<Banner>,
    pub video: Option<Video>,
    pub native: Option<serde_json::Value>,
    pub audio: Option<serde_json::Value>,
    #[serde(default)]
    pub bidfloor: Option<f64>,
    pub bidfloorcur: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Banner {
    pub format: Option<Vec<Format>>,
    pub w: Option<i32>,
    pub h: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Format {
    pub w: Option<i32>,
    pub h: Option<i32>,
    pub wratio: Option<i32>,
    pub hratio: Option<i32>,
    pub wmin: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Video {
    pub mimes: Option<Vec<String>>,
    pub minduration: Option<i32>,
    pub maxduration: Option<i32>,
    pub protocols: Option<Vec<i32>>,
    pub w: Option<i32>,
    pub h: Option<i32>,
    pub startdelay: Option<i32>,
    pub linearity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Site {
    pub id: Option<String>,
    pub page: Option<String>,
    pub domain: Option<String>,
    pub name: Option<String>,
    pub cat: Option<Vec<String>>,
    /// `ref` is a Rust keyword; r# maps to the JSON key "ref".
    pub r#ref: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct App {
    pub id: Option<String>,
    pub bundle: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    pub ua: Option<String>,
    pub ip: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ErrorCode {
    ParseError,
    EmptyId,
    EmptyImp,
    EmptyImpId,
    DuplicateImpId,
    MissingSiteAndApp,
    SiteAndAppBothPresent,
    MissingMediaType,
    NegativeBidFloor,
    InvalidBidFloorCur,
    UnknownBidFloorCur,
    CurMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationError {
    pub code: ErrorCode,
    pub severity: Severity,
    pub path: String,
    pub message: String,
}

impl ValidationError {
    fn error(code: ErrorCode, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    fn warning(code: ErrorCode, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Active ISO-4217 currency codes.
const KNOWN_CURRENCIES: &[&str] = &[
    "AED", "AFN", "ALL", "AMD", "ANG", "AOA", "ARS", "AUD", "AWG", "AZN", "BAM", "BBD", "BDT",
    "BGN", "BHD", "BIF", "BMD", "BND", "BOB", "BOV", "BRL", "BSD", "BTN", "BWP", "BYN", "BZD",
    "CAD", "CDF", "CHE", "CHF", "CHW", "CLF", "CLP", "CNY", "COP", "COU", "CRC", "CUC", "CUP",
    "CVE", "CZK", "DJF", "DKK", "DOP", "DZD", "EGP", "ERN", "ETB", "EUR", "FJD", "FKP", "GBP",
    "GEL", "GHS", "GIP", "GMD", "GNF", "GTQ", "GYD", "HKD", "HNL", "HTG", "HUF", "IDR", "ILS",
    "INR", "IQD", "IRR", "ISK", "JMD", "JOD", "JPY", "KES", "KGS", "KHR", "KMF", "KPW", "KRW",
    "KWD", "KYD", "KZT", "LAK", "LBP", "LKR", "LRD", "LSL", "LYD", "MAD", "MDL", "MGA", "MKD",
    "MMK", "MNT", "MOP", "MRU", "MUR", "MVR", "MWK", "MXN", "MXV", "MYR", "MZN", "NAD", "NGN",
    "NIO", "NOK", "NPR", "NZD", "OMR", "PAB", "PEN", "PGK", "PHP", "PKR", "PLN", "PYG", "QAR",
    "RON", "RSD", "RUB", "RWF", "SAR", "SBD", "SCR", "SDG", "SEK", "SGD", "SHP", "SLE", "SOS",
    "SRD", "SSP", "STN", "SVC", "SYP", "SZL", "THB", "TJS", "TMT", "TND", "TOP", "TRY", "TTD",
    "TWD", "TZS", "UAH", "UGX", "USD", "USN", "UYI", "UYU", "UYW", "UZS", "VED", "VES", "VND",
    "VUV", "WST", "XAF", "XAG", "XAU", "XBA", "XBB", "XBC", "XBD", "XCD", "XDR", "XOF", "XPD",
    "XPF", "XPT", "XSU", "XTS", "XUA", "XXX", "YER", "ZAR", "ZMW", "ZWG",
];

fn is_well_formed_currency_code(s: &str) -> bool {
    s.len() == 3 && s.bytes().all(|b| b.is_ascii_uppercase())
}

/// Validates a raw OpenRTB bid request. Never panics; malformed JSON yields a
/// single `ParseError` and no other rules run.
pub fn validate(json: &str) -> Vec<ValidationError> {
    let request: BidRequest = match serde_json::from_str(json) {
        Ok(request) => request,
        Err(e) => {
            return vec![ValidationError::error(
                ErrorCode::ParseError,
                format!("invalid JSON: {e}"),
                "",
            )];
        }
    };

    let mut errors = Vec::new();

    if request.id.is_empty() {
        errors.push(ValidationError::error(
            ErrorCode::EmptyId,
            "id must not be empty",
            "id",
        ));
    }

    if request.imp.is_empty() {
        errors.push(ValidationError::error(
            ErrorCode::EmptyImp,
            "imp must contain at least one object",
            "imp",
        ));
    }

    // OpenRTB 2.6 adds `dooh` as a third channel object alongside site/app;
    // this exclusivity check does not account for it.
    match (&request.site, &request.app) {
        (None, None) => errors.push(ValidationError::error(
            ErrorCode::MissingSiteAndApp,
            "exactly one of site or app must be present",
            "",
        )),
        (Some(_), Some(_)) => errors.push(ValidationError::error(
            ErrorCode::SiteAndAppBothPresent,
            "exactly one of site or app must be present, not both",
            "",
        )),
        _ => {}
    }

    let allowed_curs: Vec<String> = match &request.cur {
        Some(cur) if !cur.is_empty() => cur.clone(),
        _ => vec!["USD".to_string()],
    };

    let mut seen_imp_ids: HashSet<&str> = HashSet::new();
    for (i, imp) in request.imp.iter().enumerate() {
        if imp.id.is_empty() {
            errors.push(ValidationError::error(
                ErrorCode::EmptyImpId,
                "imp.id must not be empty",
                format!("imp[{i}].id"),
            ));
        } else if !seen_imp_ids.insert(imp.id.as_str()) {
            errors.push(ValidationError::error(
                ErrorCode::DuplicateImpId,
                "imp.id must be unique within the request",
                format!("imp[{i}].id"),
            ));
        }

        if imp.banner.is_none()
            && imp.video.is_none()
            && imp.native.is_none()
            && imp.audio.is_none()
        {
            errors.push(ValidationError::error(
                ErrorCode::MissingMediaType,
                "imp must have at least one of banner, video, native, or audio",
                format!("imp[{i}]"),
            ));
        }

        if let Some(bidfloor) = imp.bidfloor
            && bidfloor < 0.0
        {
            errors.push(ValidationError::error(
                ErrorCode::NegativeBidFloor,
                "bidfloor must be >= 0",
                format!("imp[{i}].bidfloor"),
            ));
        }

        if let Some(bidfloorcur) = &imp.bidfloorcur {
            if !is_well_formed_currency_code(bidfloorcur) {
                errors.push(ValidationError::error(
                    ErrorCode::InvalidBidFloorCur,
                    "bidfloorcur must be a 3-letter uppercase ISO-4217 code",
                    format!("imp[{i}].bidfloorcur"),
                ));
            } else if !KNOWN_CURRENCIES.contains(&bidfloorcur.as_str()) {
                errors.push(ValidationError::warning(
                    ErrorCode::UnknownBidFloorCur,
                    "bidfloorcur is well-formed but not a recognized ISO-4217 code",
                    format!("imp[{i}].bidfloorcur"),
                ));
            } else if !allowed_curs.iter().any(|c| c == bidfloorcur) {
                errors.push(ValidationError::warning(
                    ErrorCode::CurMismatch,
                    "bidfloorcur is not present in cur",
                    format!("imp[{i}].bidfloorcur"),
                ));
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_sample_bid_request() {
        let json = r#"
        {
            "id": "req-123",
            "imp": [
                {
                    "id": "imp-1",
                    "banner": { "w": 300, "h": 250 },
                    "bidfloor": 0.5,
                    "bidfloorcur": "USD"
                }
            ],
            "at": 2,
            "tmax": 120,
            "site": { "id": "site-1", "domain": "example.com" , "page":"https://example.com/article"},
            "device": { "ua": "Mozilla/5.0", "ip": "203.0.113.1" },
            "user": { "id": "user-1" }
        }
        "#;

        let bid_request: BidRequest =
            serde_json::from_str(json).expect("should deserialize sample bid request");

        assert_eq!(bid_request.id, "req-123");
        assert_eq!(bid_request.at, Some(2));
        assert_eq!(bid_request.tmax, Some(120));
        assert_eq!(bid_request.imp.len(), 1);

        let imp = &bid_request.imp[0];
        assert_eq!(imp.id, "imp-1");
        assert_eq!(imp.bidfloor, Some(0.5));
        assert_eq!(imp.bidfloorcur.as_deref(), Some("USD"));
        assert!(imp.banner.is_some());
        assert!(imp.video.is_none());

        assert!(bid_request.site.is_some());
        assert!(bid_request.app.is_none());
        assert!(bid_request.device.is_some());
        assert!(bid_request.user.is_some());

        let site = bid_request.site.as_ref().unwrap();
        assert_eq!(site.page.as_deref(), Some("https://example.com/article"));
    }

    #[test]
    fn deserializes_request_without_at() {
        let json = r#"{"id":"req-2","imp":[{"id":"imp-1"}]}"#;
        let br: BidRequest = serde_json::from_str(json).expect("at is optional in OpenRTB 2.x");
        assert_eq!(br.at, None);
    }

    #[test]
    fn deserializes_banner_with_format_array() {
        let json = r#"{
            "id":"req-3",
            "imp":[{"id":"imp-1","banner":{"format":[{"w":300,"h":250},{"w":300,"h":600}]}}]
        }"#;
        let br: BidRequest = serde_json::from_str(json).unwrap();
        let fmts = br.imp[0].banner.as_ref().unwrap().format.as_ref().unwrap();
        assert_eq!(fmts.len(), 2);
        assert_eq!(fmts[0].w, Some(300));
        assert_eq!(fmts[1].h, Some(600));
    }

    #[test]
    fn deserializes_video_imp() {
        let json = r#"{
            "id":"req-4",
            "imp":[{"id":"imp-1","video":{"mimes":["video/mp4"],"minduration":5,"maxduration":30,"protocols":[2,3,5,6]}}]
        }"#;
        let br: BidRequest = serde_json::from_str(json).unwrap();
        let v = br.imp[0].video.as_ref().unwrap();
        assert_eq!(v.mimes.as_ref().unwrap()[0], "video/mp4");
        assert_eq!(v.protocols.as_ref().unwrap().len(), 4);
        assert_eq!(v.maxduration, Some(30));
    }

    #[test]
    fn omitted_bidfloor_is_distinguishable_from_zero() {
        let omitted: BidRequest = serde_json::from_str(r#"{"id":"a","imp":[{"id":"i"}]}"#).unwrap();
        let explicit: BidRequest =
            serde_json::from_str(r#"{"id":"a","imp":[{"id":"i","bidfloor":0.0}]}"#).unwrap();
        assert_eq!(omitted.imp[0].bidfloor, None);
        assert_eq!(explicit.imp[0].bidfloor, Some(0.0));
    }

    fn valid_request_json() -> &'static str {
        r#"{
            "id": "req-1",
            "imp": [
                {
                    "id": "imp-1",
                    "banner": { "w": 300, "h": 250 },
                    "bidfloor": 0.5,
                    "bidfloorcur": "USD"
                }
            ],
            "site": { "id": "site-1", "domain": "example.com" },
            "cur": ["USD"]
        }"#
    }

    #[test]
    fn valid_request_returns_empty_vec() {
        assert_eq!(validate(valid_request_json()), vec![]);
    }

    #[test]
    fn malformed_json_returns_single_parse_error() {
        let errors = validate("{ this is not json");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ErrorCode::ParseError);
        assert_eq!(errors[0].path, "");
    }

    #[test]
    fn empty_id_is_error() {
        let json = r#"{
            "id": "",
            "imp": [{ "id": "imp-1", "banner": {} }],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::EmptyId && e.path == "id")
        );
    }

    #[test]
    fn empty_imp_is_error() {
        let json = r#"{
            "id": "req-1",
            "imp": [],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::EmptyImp && e.path == "imp")
        );
    }

    #[test]
    fn empty_imp_id_is_error() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "", "banner": {} }],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::EmptyImpId && e.path == "imp[0].id")
        );
    }

    #[test]
    fn duplicate_imp_id_is_error() {
        let json = r#"{
            "id": "req-1",
            "imp": [
                { "id": "dup", "banner": {} },
                { "id": "dup", "banner": {} }
            ],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        assert!(
            !errors
                .iter()
                .any(|e| e.code == ErrorCode::DuplicateImpId && e.path == "imp[0].id")
        );
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::DuplicateImpId && e.path == "imp[1].id")
        );
    }

    #[test]
    fn missing_site_and_app_is_error() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "imp-1", "banner": {} }]
        }"#;
        let errors = validate(json);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::MissingSiteAndApp && e.path.is_empty())
        );
    }

    #[test]
    fn site_and_app_both_present_is_error() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "imp-1", "banner": {} }],
            "site": { "id": "site-1" },
            "app": { "id": "app-1" }
        }"#;
        let errors = validate(json);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::SiteAndAppBothPresent && e.path.is_empty())
        );
    }

    #[test]
    fn missing_media_type_is_error() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "imp-1" }],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::MissingMediaType && e.path == "imp[0]")
        );
    }

    #[test]
    fn negative_bidfloor_is_error() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "imp-1", "banner": {}, "bidfloor": -1.0 }],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::NegativeBidFloor && e.path == "imp[0].bidfloor")
        );
    }

    #[test]
    fn malformed_bidfloorcur_is_error() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "imp-1", "banner": {}, "bidfloorcur": "usd" }],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::InvalidBidFloorCur && e.path == "imp[0].bidfloorcur")
        );
    }

    #[test]
    fn unknown_bidfloorcur_is_warning() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "imp-1", "banner": {}, "bidfloorcur": "ZZZ" }],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        let e = errors
            .iter()
            .find(|e| e.code == ErrorCode::UnknownBidFloorCur)
            .expect("expected an UnknownBidFloorCur diagnostic");
        assert_eq!(e.path, "imp[0].bidfloorcur");
        assert_eq!(e.severity, Severity::Warning);
    }

    #[test]
    fn bidfloorcur_not_in_cur_is_warning() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "imp-1", "banner": {}, "bidfloorcur": "USD" }],
            "site": { "id": "site-1" },
            "cur": ["EUR"]
        }"#;
        let errors = validate(json);
        let e = errors
            .iter()
            .find(|e| e.code == ErrorCode::CurMismatch)
            .expect("expected a CurMismatch diagnostic");
        assert_eq!(e.path, "imp[0].bidfloorcur");
        assert_eq!(e.severity, Severity::Warning);
    }

    #[test]
    fn absent_cur_defaults_to_usd() {
        let json = r#"{
            "id": "req-1",
            "imp": [{ "id": "imp-1", "banner": {}, "bidfloorcur": "EUR" }],
            "site": { "id": "site-1" }
        }"#;
        let errors = validate(json);
        let e = errors
            .iter()
            .find(|e| e.code == ErrorCode::CurMismatch)
            .expect("absent cur should default to [\"USD\"], flagging EUR as a mismatch");
        assert_eq!(e.path, "imp[0].bidfloorcur");
    }

    #[test]
    fn absent_imp_field_is_parse_error() {
        let json = r#"{"id":"req-1","site":{"page":"https://example.com"}}"#;
        let errors = validate(json);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ErrorCode::ParseError);
    }

    #[test]
    fn realistic_web_display_request_is_valid() {
        let json = r#"{
        "id": "80ce30c53c16e6ede735f123ef6e32361bfc7b22",
        "at": 1,
        "tmax": 120,
        "cur": ["USD"],
        "imp": [{
            "id": "1",
            "tagid": "/1234/sports/leaderboard",
            "bidfloor": 1.75,
            "bidfloorcur": "USD",
            "secure": 1,
            "banner": {
                "format": [{"w": 728, "h": 90}, {"w": 970, "h": 250}],
                "pos": 1,
                "battr": [13],
                "api": [7]
            }
        }],
        "site": {
            "id": "102855",
            "domain": "example.com",
            "page": "https://example.com/sports/article-123",
            "cat": ["IAB17"],
            "publisher": {"id": "8953", "name": "Example Media"}
        },
        "device": {
            "ua": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
            "ip": "203.0.113.1",
            "devicetype": 2,
            "os": "OS X",
            "language": "en",
            "geo": {"country": "USA", "region": "CA", "type": 2}
        },
        "user": {"id": "55816b39711f9b5acf3b90e313ed29e51665623f"},
        "regs": {"gdpr": 0},
        "bcat": ["IAB25", "IAB26"],
        "badv": ["competitor.com"]
    }"#;
        assert!(validate(json).is_empty());
    }

    #[test]
    fn errors_are_returned_in_document_order() {
        let json = r#"{
            "id": "",
            "imp": [
                { "id": "", "banner": {} },
                { "id": "imp-2" }
            ]
        }"#;
        let errors = validate(json);
        let positions: Vec<(ErrorCode, &str)> =
            errors.iter().map(|e| (e.code, e.path.as_str())).collect();
        assert_eq!(
            positions,
            vec![
                (ErrorCode::EmptyId, "id"),
                (ErrorCode::MissingSiteAndApp, ""),
                (ErrorCode::EmptyImpId, "imp[0].id"),
                (ErrorCode::MissingMediaType, "imp[1]"),
            ]
        );
    }
}
