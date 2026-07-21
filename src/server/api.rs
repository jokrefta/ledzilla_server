use std::{
    io::Read,
    sync::mpsc::{SyncSender, channel},
};

use log::trace;
use rouille::{Request, Response, input::post::BufferedFile, post_input, try_or_400};

use super::log_err_result;
use crate::{graphics_component::ComponentList, renderer::RendererCommand};

const API_VERSION: &str = "0.1.0";

#[derive(serde::Serialize)]
struct DisplayInfo {
    width: u32,
    height: u32,
    api_version: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ComponentState {
    components: ComponentList,
}

/// Equivalent to rouille::input::json::json_input() except it extracts the message as a string
/// and logs it
fn json_input<O>(request: &Request) -> Result<O, rouille::input::json::JsonError>
where
    O: serde::de::DeserializeOwned,
{
    use rouille::input::json::JsonError;

    if let Some(header) = request.header("Content-Type") {
        if !header.starts_with("application/json") {
            return Err(JsonError::WrongContentType);
        }
    } else {
        return Err(JsonError::WrongContentType);
    }

    if let Some(mut b) = request.data() {
        let mut buf = Vec::<u8>::new();

        b.read_to_end(&mut buf).unwrap();

        const MAX_LEN_TO_LOG: usize = 300;
        let len_to_log = MAX_LEN_TO_LOG.min(buf.len());
        if let Ok(s) = str::from_utf8(&buf[..len_to_log]) {
            trace!("Parsing JSON: {}", s);
        } else {
            trace!("Could not log json (not utf-8)");
        }

        serde_json::from_slice::<O>(&buf).map_err(From::from)
    } else {
        Err(JsonError::BodyAlreadyExtracted)
    }
}

// ------------------------ API handlers ------------------------ //
// These are all called from the rouille worker threads. They must return a Response, or panic.
// If they panic, Rouille automatically creates a 500 response.

pub fn handle_info_get() -> Response {
    // TODO - un-hardcode width/height!
    Response::json(&DisplayInfo {
        width: 64 * 4,
        height: 64,
        api_version: API_VERSION.to_string(),
    })
}

pub fn handle_state_get(renderer: &SyncSender<RendererCommand>) -> Response {
    let (response_sender, response_receiver) = channel::<ComponentList>();
    trace!("sending get component state command");
    renderer
        .send(RendererCommand::GetComponents { response_sender })
        .unwrap();

    let components = response_receiver.recv().unwrap();
    trace!("got state response {:?}", components);
    Response::json(&ComponentState { components })
}

pub fn handle_state_post(req: &Request, renderer: &SyncSender<RendererCommand>) -> Response {
    let state: ComponentState = try_or_400!(log_err_result(json_input(req)));
    let (response_sender, response_receiver) = channel::<bool>();
    renderer
        .send(RendererCommand::SetComponents {
            components: state.components,
            response_sender,
        })
        .unwrap();

    if !response_receiver.recv().unwrap() {
        panic!("Failed to write display components");
    }
    trace!("got display start response");
    Response::empty_204()
}

pub fn handle_display_on(renderer: &SyncSender<RendererCommand>) -> Response {
    let (response_sender, response_receiver) = channel::<bool>();
    trace!("sending display start command");
    renderer.send(RendererCommand::Start { response_sender }).unwrap();

    if !response_receiver.recv().unwrap() {
        panic!("Failed to start display");
    }
    trace!("got display start response");
    Response::empty_204()
}

pub fn handle_display_off(renderer: &SyncSender<RendererCommand>) -> Response {
    let (response_sender, response_receiver) = channel::<bool>();
    trace!("sending display stop command");
    renderer.send(RendererCommand::Stop { response_sender }).unwrap();

    if !response_receiver.recv().unwrap() {
        panic!("Failed to stop display");
    }
    trace!("got display stop response");

    Response::empty_204()
}
