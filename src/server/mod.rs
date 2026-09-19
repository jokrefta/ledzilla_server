use std::{
    fs::File,
    sync::{Arc, Mutex, mpsc::SyncSender},
};

use log::{debug, error, info, warn};
use rouille::{Request, Response, router, try_or_404};

use crate::{LedzillaServerConfig, LedzillaServerState, renderer, upload::UploadManager};

mod api;

const CLIENT_ID_HEADER: &str = "Ledzilla-Client-ID";

fn log_err_result<T, U: std::error::Error>(result: Result<T, U>) -> Result<T, U> {
    if let Err(ref e) = result {
        warn!("Got error result - {}", e);
        debug!(" -- error cause was {:?}", e.source());
    }
    result
}

fn log_ok(req: &Request, resp: &Response, _elap: std::time::Duration) {
    let client_str = if let Some(id) = req.header(CLIENT_ID_HEADER) {
        format!("(From {id})")
    } else {
        String::new()
    };
    if resp.is_error() {
        warn!(
            "{} {} {}-> {}",
            req.method(),
            req.raw_url(),
            client_str,
            resp.status_code
        );
    } else {
        info!(
            "{} {} {} -> {}",
            req.method(),
            req.raw_url(),
            client_str,
            resp.status_code
        );
    }
}

fn log_err(req: &Request, _elap: std::time::Duration) {
    error!("Handler panicked: {} {}", req.method(), req.raw_url());
}

pub fn handle_request(
    renderer_sender: SyncSender<renderer::Command>,
    req: &Request,
    upload_manager: Arc<Mutex<UploadManager>>,
    server_state: Arc<LedzillaServerState>,
    config: &LedzillaServerConfig,
) -> Response {
    rouille::log_custom(req, log_ok, log_err, || {
        router!(req,
            (DELETE) (/api/files/{name_}) => {
                let name: String = name_; // make macro infer the right type
                api::handle_delete_file(req, &name, &upload_manager, &server_state)
            },
            (GET) (/) => {
                let index = try_or_404!(log_err_result(File::open(config.content_root.clone() + "/index.html")));
                Response::from_file("text/html", index)
            },
            (GET) (/api/last-modified-by) => {
                api::handle_last_modified_id_get(&server_state)
            },
            (GET) (/api/files) => {
                api::handle_files_get(&upload_manager)
            },
            (GET) (/api/info) => {
                api::handle_info_get(config)
            },
            (GET) (/api/state) => {
                api::handle_state_get(&renderer_sender)
            },
            (POST) (/api/state) => {
                api::handle_state_post(req, &renderer_sender, &server_state)
            },
            (GET) (/api/display/on-off-state) => {
                api::handle_display_get_on_off(&renderer_sender)
            },
            (POST) (/api/display/on-off-state) => {
                api::handle_display_set_on_off(req, &renderer_sender, &server_state)
            },
            (PUT) (/api/files/{name}) => {
                api::handle_upload(req, name, &upload_manager, &server_state)
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
