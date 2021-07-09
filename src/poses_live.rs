// A client library for their portal https://poses.live

const API_KEY: &str = "81acc597-be90-418c-90aa-0dfac878aeb0";

crate::entry_point!("poses_live_demo", poses_live_demo);
fn poses_live_demo() {
    let resp = ureq::get("https://poses.live/api/hello")
        .set("Authorization", &format!("Bearer {}", API_KEY))
        .call().unwrap();
    eprintln!("{:?}", resp);
    let body = resp.into_string().unwrap();
    eprintln!("{}", body);
}
