use std::sync::mpsc::Sender;

use rouille::{Request, Response, input::post::BufferedFile, post_input, try_or_400};

#[derive(serde::Serialize)]
struct DisplayInfo {
    width: u32,
    height: u32,
    version: String,
}

pub fn handle_info(req: &Request) -> Response {
    Response::json(&DisplayInfo {
        width: 64 * 4,
        height: 64,
        version: "0.0.0".to_string(),
    })
}

pub fn handle_state_get(req: &Request) -> Response {
    Response::text("ok.")
}

pub fn handle_state_post(debug_sender: &Sender<String>, req: &Request) -> Response {
    let data = try_or_400!(post_input!(
        req, {
            state: String,
            uploads: Vec<BufferedFile>,
        }
    ));
    debug_sender.send(format!("{:?}", data)).unwrap();

    Response::empty_204()
}

pub fn handle_display_on(req: &Request) -> Response {
    // TODO
    Response::empty_204()
}

pub fn handle_display_off(req: &Request) -> Response {
    // TODO
    Response::empty_204()
}
