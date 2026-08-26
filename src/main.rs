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
    format: Option<Vec<Format>>,
    w: Option<i32>,
    h: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Format {
    w: Option<i32>,
    h: Option<i32>,
    wratio: Option<i32>,
    hratio: Option<i32>,
    wmin: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Video {
    mimes: Option<Vec<String>>,
    minduration: Option<i32>,
    maxduration: Option<i32>,
    protocols: Option<Vec<i32>>,
    w: Option<i32>,
    h: Option<i32>,
    startdelay: Option<i32>,
    linearity: Option<i32>,
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
}
