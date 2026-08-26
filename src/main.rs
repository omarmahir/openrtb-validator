use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BidRequest {
    id: String,
    imp: Vec<Imp>,
    at: Option<i32>,
    tmax: Option<i64>,
    site: Option<Site>,
    app: Option<App>,
    device: Option<Device>,
    user: Option<User>,
}

#[derive(Debug, Deserialize)]
struct Imp {
    id: String,
    banner: Option<Banner>,
    video: Option<Video>,
    #[serde(default)]
    bidfloor: f64,
    bidfloorcur: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Banner {
    w: Option<i32>,
    h: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Video {
    w: Option<i32>,
    h: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Site {
    id: Option<String>,
    domain: Option<String>,
}

#[derive(Debug, Deserialize)]
struct App {
    id: Option<String>,
    bundle: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Device {
    ua: Option<String>,
    ip: Option<String>,
}

#[derive(Debug, Deserialize)]
struct User {
    id: Option<String>,
}

fn main() {
    println!("Hello, world!");
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
            "site": { "id": "site-1", "domain": "example.com" },
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
        assert_eq!(imp.bidfloor, 0.5);
        assert_eq!(imp.bidfloorcur.as_deref(), Some("USD"));
        assert!(imp.banner.is_some());
        assert!(imp.video.is_none());

        assert!(bid_request.site.is_some());
        assert!(bid_request.app.is_none());
        assert!(bid_request.device.is_some());
        assert!(bid_request.user.is_some());
    }

    #[test]
    fn deserializes_request_without_at() {
        let json = r#"{"id":"req-2","imp":[{"id":"imp-1"}]}"#;
        let br: BidRequest = serde_json::from_str(json)
            .expect("at is optional in OpenRTB 2.x");
        assert_eq!(br.at, None);
    }
}
