use serde::Deserialize;

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
}

#[derive(Debug, Deserialize)]
pub struct Imp {
    pub id: String,
    pub banner: Option<Banner>,
    pub video: Option<Video>,
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

    #[test]
    fn omitted_bidfloor_is_distinguishable_from_zero() {
        let omitted: BidRequest =
            serde_json::from_str(r#"{"id":"a","imp":[{"id":"i"}]}"#).unwrap();
        let explicit: BidRequest =
            serde_json::from_str(r#"{"id":"a","imp":[{"id":"i","bidfloor":0.0}]}"#).unwrap();
        assert_eq!(omitted.imp[0].bidfloor, None);
        assert_eq!(explicit.imp[0].bidfloor, Some(0.0));
    }
}
