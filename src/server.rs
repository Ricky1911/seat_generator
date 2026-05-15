use std::path::Path;
use std::time::Duration;

const SERVER_URL: &str = "https://secret.hammers.top/generate";

pub fn try_fetch_from_server(
    template_path: &Path,
    history1_path: &Path,
    history2_path: &Path,
    output_path: &Path,
) -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(3000))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let template = match std::fs::read(template_path) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let history1 = match std::fs::read(history1_path) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let history2 = match std::fs::read(history2_path) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let form = reqwest::blocking::multipart::Form::new()
        .part(
            "template",
            reqwest::blocking::multipart::Part::bytes(template).file_name("template.xlsx"),
        )
        .part(
            "history1",
            reqwest::blocking::multipart::Part::bytes(history1).file_name("history1.xlsx"),
        )
        .part(
            "history2",
            reqwest::blocking::multipart::Part::bytes(history2).file_name("history2.xlsx"),
        );

    let response = match client.post(SERVER_URL).multipart(form).send() {
        Ok(r) => r,
        Err(_) => return false,
    };

    if response.status() != 200 {
        return false;
    }

    let bytes = match response.bytes() {
        Ok(b) => b,
        Err(_) => return false,
    };

    std::fs::write(output_path, bytes.as_ref()).is_ok()
}
