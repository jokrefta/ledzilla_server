use std::{fs::File, sync::mpsc::Sender};

use rouille::{Request, Response, router, try_or_404};

mod api;

pub fn handle_request(debug_sender: Sender<String>, req: &Request) -> Response {
    router!(req,
        (GET) (/) => {
            let index = try_or_404!(File::open("web_content/index.html"));
            Response::from_file("text/html", index)
        },
        (GET) (/api/info) => {
            debug_sender.send(format!("{:?}", req)).unwrap();
            api::handle_info(req)
        },
        (GET) (/api/state) => {
            debug_sender.send(format!("{:?}", req)).unwrap();
            api::handle_state_get(req)

        },
        (POST) (/api/state) => {
            debug_sender.send(format!("{:?}", req)).unwrap();
            api::handle_state_post(&debug_sender, req)
        },
        (POST) (/api/display/on) => {
            debug_sender.send(format!("{:?}", req)).unwrap();
            api::handle_display_on(req)
        },
        (POST) (/api/display/off) => {
            debug_sender.send(format!("{:?}", req)).unwrap();
            api::handle_display_off(req)
        },
        _ => {
            if req.method() == "GET" {
                rouille::match_assets(req, "web_content/")
            }
            else {
                Response::empty_404()
            }
        }
    )
}
