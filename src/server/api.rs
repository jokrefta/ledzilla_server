use anyhow::Result;
use std::{
    io::Read,
    sync::{
        Mutex,
        mpsc::{SyncSender, channel},
    },
};

use log::{debug, trace};
use rouille::{Request, Response, input::post::BufferedFile, post_input, try_or_400};
use strum::VariantNames;

use super::log_err_result;
use crate::{
    graphics_component::ComponentList,
    renderer::{Command, CommandError},
    upload::{AnimatedImageBuf, ImageBuf, UploadManager, UploadedAsset},
};

const API_VERSION: &str = "0.6.1";

#[derive(serde::Serialize)]
struct DisplayInfo {
    width: u32,
    height: u32,
    api_version: String,
    available_fonts: &'static [&'static str],
}

#[derive(serde::Serialize)]
struct FilesList<'a> {
    files: Vec<&'a str>,
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
        available_fonts: crate::graphics_component::Font::VARIANTS,
    })
}

pub fn handle_state_get(renderer: &SyncSender<Command>) -> Response {
    let (response_sender, response_receiver) = channel::<ComponentList>();
    trace!("sending get component state command");
    renderer.send(Command::GetComponents { response_sender }).unwrap();

    let components = response_receiver.recv().unwrap();
    trace!("got state response {:?}", components);
    Response::json(&ComponentState { components })
}

pub fn handle_state_post(req: &Request, renderer: &SyncSender<Command>) -> Response {
    let state: ComponentState = try_or_400!(log_err_result(json_input(req)));
    let (response_sender, response_receiver) = channel::<Result<(), CommandError>>();
    renderer
        .send(Command::SetComponents {
            components: state.components,
            response_sender,
        })
        .unwrap();

    match response_receiver.recv().unwrap() {
        Ok(_) => Response::empty_204(),
        Err(CommandError::InternalServerError(s)) => panic!("Failed to set components - {}", s),
        Err(CommandError::UserInputError(s)) => Response::text(s).with_status_code(400),
    }
}

pub fn handle_display_on(renderer: &SyncSender<Command>) -> Response {
    let (response_sender, response_receiver) = channel::<bool>();
    trace!("sending display start command");
    renderer.send(Command::Start { response_sender }).unwrap();

    if !response_receiver.recv().unwrap() {
        panic!("Failed to start display");
    }
    trace!("got display start response");
    Response::empty_204()
}

pub fn handle_display_off(renderer: &SyncSender<Command>) -> Response {
    let (response_sender, response_receiver) = channel::<bool>();
    trace!("sending display stop command");
    renderer.send(Command::Stop { response_sender }).unwrap();

    if !response_receiver.recv().unwrap() {
        panic!("Failed to stop display");
    }
    trace!("got display stop response");

    Response::empty_204()
}

pub fn handle_files_get(upload_manager: &Mutex<UploadManager>) -> Response {
    let upload_manager = upload_manager.lock().unwrap();
    let files = upload_manager.list_files();
    Response::json(&FilesList { files })
}

pub fn handle_upload(req: &Request, filename: String, upload_manager: &Mutex<UploadManager>) -> Response {
    let input = try_or_400!(post_input!(req, {
        width: Option<u32>,
        height: Option<u32>,
        // bool means something special in post_input!, so capture as a string and convert later
        animated: String,
        file: BufferedFile,
    }));
    let is_animated = input.animated.trim().eq_ignore_ascii_case("true");
    debug!(
        "File upload request, width={:?} height={:?} animated='{}' ({})",
        input.width, input.height, input.animated, is_animated
    );
    if input.width.is_some() || input.height.is_some() {
        todo!("Implement image resizing (and test if it works for GIFS!)");
    }

    let asset = if is_animated {
        UploadedAsset::AnimatedImage(AnimatedImageBuf::from_encoded_buffer(input.file.data))
    } else {
        UploadedAsset::Image(ImageBuf::from_encoded_buffer(input.file.data))
    };

    let mut upload_manager = upload_manager.lock().unwrap();
    match upload_manager.insert(filename, asset) {
        // Special 201 code indicates new resource created; otherwise use 204
        Some(_) => Response::empty_204(),
        None => Response::text("Created ".to_string() + req.raw_url()).with_status_code(201),
    }
}

pub fn handle_delete_file(name: &str, upload_manager: &Mutex<UploadManager>) -> Response {
    let mut upload_manager = upload_manager.lock().unwrap();
    if upload_manager.try_delete(name).is_ok() {
        Response::empty_204()
    } else {
        Response::empty_404()
    }
}
