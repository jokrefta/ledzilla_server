use std::{fs::File, sync::mpsc::SyncSender};

use log::{error, info, warn};
use rouille::{Request, Response, router, try_or_404};

use crate::renderer;

mod api;

fn log_err_result<T, U: std::error::Error>(result: Result<T, U>) -> Result<T, U> {
    if let Err(ref e) = result {
        warn!("Got error result - {}", e.to_string());
    }
    result
}

fn log_ok(req: &Request, resp: &Response, _elap: std::time::Duration) {
    if resp.is_error() {
        warn!("{} {} -> {}", req.method(), req.raw_url(), resp.status_code);
    } else {
        info!("{} {} -> {}", req.method(), req.raw_url(), resp.status_code);
    }
}

fn log_err(req: &Request, _elap: std::time::Duration) {
    error!("Handler panicked: {} {}", req.method(), req.raw_url());
}

pub fn handle_request(renderer_sender: SyncSender<renderer::Command>, req: &Request) -> Response {
    rouille::log_custom(req, log_ok, log_err, || {
        router!(req,
            (GET) (/) => {
                let index = try_or_404!(log_err_result(File::open("web_content/index.html")));
                Response::from_file("text/html", index)
            },
            (GET) (/api/info) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_info_get()
            },
            (GET) (/api/state) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_state_get(&renderer_sender)
            },
            (POST) (/api/state) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_state_post(req, &renderer_sender)
            },
            (POST) (/api/display/on) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_display_on(&renderer_sender)
            },
            (POST) (/api/display/off) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_display_off(&renderer_sender)
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
