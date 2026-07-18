use std::{fs::File, sync::mpsc::SyncSender};

use log::{error, info};
use rouille::{Request, Response, router, try_or_404};

use crate::renderer::RendererCommand;

mod api;

fn log_ok(req: &Request, resp: &Response, _elap: std::time::Duration) {
    info!("{} {} -> {}", req.method(), req.raw_url(), resp.status_code);
}

fn log_err(req: &Request, _elap: std::time::Duration) {
    error!("Handler panicked: {} {}", req.method(), req.raw_url());
}

pub fn handle_request(renderer_sender: SyncSender<RendererCommand>, req: &Request) -> Response {
    rouille::log_custom(req, log_ok, log_err, || {
        router!(req,
            (GET) (/) => {
                let index = try_or_404!(File::open("web_content/index.html"));
                Response::from_file("text/html", index)
            },
            (GET) (/api/info) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_info(req)
            },
            (GET) (/api/state) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_state_get(req)

            },
            (POST) (/api/state) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_state_post(req)
            },
            (POST) (/api/display/on) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_display_on(req)
            },
            (POST) (/api/display/off) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
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
    })
}
