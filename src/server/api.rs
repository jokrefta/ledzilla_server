use std::sync::mpsc::{SyncSender, channel};

use log::trace;
use rouille::{Request, Response, input::post::BufferedFile, post_input, try_or_400};

use crate::{component::ComponentList, renderer::RendererCommand};

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

pub fn handle_state_post(req: &Request) -> Response {
    let data = try_or_400!(post_input!(
        req, {
            state: String,
            uploads: Vec<BufferedFile>,
        }
    ));
    todo!();

    Response::empty_204()
}

pub fn handle_display_on(renderer: &SyncSender<RendererCommand>) -> Response {
    let (response_sender, response_receiver) = channel::<bool>();
    trace!("sending display start command");
    renderer
        .send(RendererCommand::Start { response_sender })
        .unwrap();

    if !response_receiver.recv().unwrap() {
        panic!("Failed to start display");
    }
    trace!("got display start response");
    Response::empty_204()
}

pub fn handle_display_off(renderer: &SyncSender<RendererCommand>) -> Response {
    let (response_sender, response_receiver) = channel::<bool>();
    trace!("sending display stop command");
    renderer
        .send(RendererCommand::Stop { response_sender })
        .unwrap();

    if !response_receiver.recv().unwrap() {
        panic!("Failed to stop display");
    }
    trace!("got display stop response");

    Response::empty_204()
}
