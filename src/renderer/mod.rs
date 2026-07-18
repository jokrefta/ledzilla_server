use std::sync::mpsc::{Receiver, Sender};

use log::debug;

use crate::display::GraphicsDisplay;

#[derive(Debug)]
pub enum RendererCommand {
    // Will likely mirror the API pretty closely
    Start { response_sender: Sender<bool> },
    Stop { response_sender: Sender<bool> },
    Todo,
}

pub fn run_renderer_thread<F, Disp>(
    mut display_provider: F,
    command_receiver: Receiver<RendererCommand>,
) where
    F: FnMut() -> Disp,
    Disp: GraphicsDisplay,
{
    let mut display: Option<Disp> = None;
    loop {
        let cmd = command_receiver.recv().unwrap();
        debug!("received {:?}", cmd);
        match cmd {
            RendererCommand::Start { response_sender } => {
                if display.is_none() {
                    display = Some(display_provider());
                }
                response_sender.send(true).unwrap();
            }
            RendererCommand::Stop { response_sender } => {
                if display.is_some() {
                    display = None;
                }
                response_sender.send(true).unwrap();
            }
            _ => {
                todo!()
            }
        }
    }
}
