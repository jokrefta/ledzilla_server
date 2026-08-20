use std::{
    fs::File,
    sync::{Arc, Mutex, mpsc::SyncSender},
};

use log::{debug, error, info, warn};
use rouille::{Request, Response, router, try_or_404};

use crate::{LedzillaServerConfig, renderer, upload::UploadManager};

mod api;

fn log_err_result<T, U: std::error::Error>(result: Result<T, U>) -> Result<T, U> {
    if let Err(ref e) = result {
        warn!("Got error result - {}", e);
        debug!(" -- error cause was {:?}", e.source());
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

pub fn handle_request(
    renderer_sender: SyncSender<renderer::Command>,
    req: &Request,
    upload_manager: Arc<Mutex<UploadManager>>,
    config: &LedzillaServerConfig,
) -> Response {
    rouille::log_custom(req, log_ok, log_err, || {
        router!(req,
            (DELETE) (/api/files/{name_}) => {
                let name: String = name_; // make macro infer the right type
                api::handle_delete_file(&name, &upload_manager)
            },
            (GET) (/) => {
                let index = try_or_404!(log_err_result(File::open(config.content_root.clone() + "/index.html")));
                Response::from_file("text/html", index)
            },
            (GET) (/api/files) => {
                api::handle_files_get(&upload_manager)
            },
            (GET) (/api/info) => {
                // debug_sender.send(format!("{:?}", req)).unwrap();
                api::handle_info_get(config)
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
            (PUT) (/api/files/{name}) => {
                api::handle_upload(req, name, &upload_manager)
            },
            _ => {
                if req.method() == "GET" {
                    rouille::match_assets(req, &config.content_root)
                }
                else {
                    Response::empty_404()
                }
            }
        )
    })
}
