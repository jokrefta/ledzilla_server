use self::state::{State, StateWrapper};
use crate::{
    LedzillaServerConfig,
    display::GraphicsDisplay,
    graphics_component::{
        ComponentList,
        draw::{self, MovableComponentDrawer},
    },
    upload,
};

use anyhow::Result;
use log::debug;
use std::fmt::Debug;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use thiserror::Error;

mod state;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Internal error: {0}")]
    InternalServerError(String),
    #[error("Bad user input. {0}")]
    UserInputError(String),
}

impl From<draw::DrawerCreationError> for CommandError {
    fn from(e: draw::DrawerCreationError) -> Self {
        match e {
            draw::DrawerCreationError::BadComponentSpec(s) => Self::UserInputError(s),
            draw::DrawerCreationError::_GeneralError(s) => Self::InternalServerError(s),
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Start {
        response_sender: Sender<bool>,
    },
    Stop {
        response_sender: Sender<bool>,
    },
    GetComponents {
        response_sender: Sender<ComponentList>,
    },
    SetComponents {
        components: ComponentList,
        response_sender: Sender<Result<(), CommandError>>,
    },
}

pub struct Renderer<F, Disp>
where
    F: FnMut() -> Disp,
    Disp: GraphicsDisplay + Debug,
{
    display_provider: F,
    // Should always be valid, but is Option so we can move out of &mut self
    state: StateWrapper<Disp>,
    components: Vec<MovableComponentDrawer>,
    upload_manager: Arc<Mutex<upload::UploadManager>>,
    config: LedzillaServerConfig,
}

impl<F, Disp> Renderer<F, Disp>
where
    F: FnMut() -> Disp,
    Disp: GraphicsDisplay + Debug,
{
    pub fn new(
        display_provider: F,
        upload_manager: Arc<Mutex<upload::UploadManager>>,
        config: LedzillaServerConfig,
    ) -> Self {
        Self {
            display_provider,
            state: StateWrapper::new(State::Stopped),
            components: Vec::new(),
            upload_manager,
            config,
        }
    }

    pub fn run(&mut self, command_receiver: Receiver<Command>) {
        loop {
            let command = self.run_until_next_command(&command_receiver);
            debug!("received {:?} in state {}", command, self.state.get_ref());
            self.handle_command(command);
            debug!("Afterwards, state is now {}", self.state.get_ref());
        }
    }

    // Run until a command is received, and return it. Since a command may cause a state transition, we
    // break from this function to handle it before re-entering in the updated state.
    fn run_until_next_command(&mut self, command_receiver: &Receiver<Command>) -> Command {
        // we want to keep doing work in a loop and do a non-blocking check
        // each iteration to see if a command has come in.

        // Note the use of update_display allows us to render without having ownership of the display.

        match self.state.get_ref() {
            State::Stopped => command_receiver.recv().unwrap(),
            State::Rendering(..) => {
                loop {
                    self.state.update_rendering_state(|rs| {
                        Self::render(&mut self.components, rs, self.config.fps_log_lvl)
                    });

                    // Early return if command found
                    match command_receiver.try_recv() {
                        Ok(c) => {
                            return c;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => (),
                        Err(_) => panic!(),
                    }
                }
            }
        }
    }

    // Handle event, updating state if necessary
    fn handle_command(&mut self, command: Command) {
        #[rustfmt::skip]
        self.state.update(|curstate| {
            match (curstate, command) {
                //__________________________________________________
                (State::Stopped, Command::Start {response_sender}) => {
                    Self::start_up(&mut self.display_provider, response_sender)
                }
                //__________________________________________________
                (State::Stopped, Command::Stop {response_sender}) => {
                    // no-op
                    response_sender.send(true).unwrap();
                    State::Stopped
                }
                //__________________________________________________
                (State::Rendering {..}, Command::Stop {response_sender}) => {
                    Self::shut_down(response_sender)
                }
                //__________________________________________________
                (state @ State::Rendering {..}, Command::Start {response_sender}) => {
                    // no-op
                    response_sender.send(true).unwrap();
                    state
                }
                //__________________________________________________
                (state, Command::GetComponents {response_sender}) => {
                    Self::get_components(&self.components, response_sender);
                    state
                }
                //__________________________________________________
                (state, Command::SetComponents {response_sender, components}) => {
                    Self::set_components(
                        &mut self.components,
                        components,
                        response_sender,
                        &self.upload_manager,
                        self.config.canvas_size
                    );
                    state
                }
                // No default case - we want every command to be responded to.
            }
        });
    }

    // These are implemented as associated functions rather than methods because
    // they are called from a context where self.state has already been borrowed exclusively
    // so we cannot borrow self.

    /// Create display and return next state. Send a true response on the channel.
    fn start_up(display_provider: &mut F, success_sender: Sender<bool>) -> State<Disp> {
        let display = (display_provider)();
        success_sender.send(true).unwrap();

        State::Rendering(state::RenderingState {
            display,
            recent_frame_timestamps: vec![],
        })
    }

    /// Shut down display and return next state. Send a true response on the channel.
    fn shut_down(success_sender: Sender<bool>) -> State<Disp> {
        success_sender.send(true).unwrap();
        State::Stopped
    }

    /// Take the display, draw onto it, update it, and return it back.
    fn render(
        components: &mut Vec<MovableComponentDrawer>,
        rendering_state: state::RenderingState<Disp>,
        fps_log_lvl: log::Level,
    ) -> state::RenderingState<Disp> {
        // debug!("rendering");
        let state::RenderingState {
            mut display,
            mut recent_frame_timestamps,
        } = rendering_state;

        let canvas = display.get_draw_target();
        for drawer in components {
            drawer.draw_next_frame(canvas);
        }
        display = display.update_display();

        recent_frame_timestamps.push(std::time::Instant::now());
        const LOG_THRESH: std::time::Duration = std::time::Duration::from_secs(1);
        if recent_frame_timestamps.len() > 1
            && *recent_frame_timestamps.last().unwrap() - *recent_frame_timestamps.first().unwrap()
                >= LOG_THRESH
        {
            log::log!(
                fps_log_lvl,
                "FPS: {:.0}",
                (recent_frame_timestamps.len() - 1) as f32
                    / (*recent_frame_timestamps.last().unwrap() - *recent_frame_timestamps.first().unwrap())
                        .as_secs_f32()
            );
            recent_frame_timestamps.clear();
        }

        state::RenderingState {
            display,
            recent_frame_timestamps,
        }
    }

    /// Update the given components and send a success response on the channel.
    fn set_components(
        to_update: &mut Vec<MovableComponentDrawer>,
        new_components: ComponentList,
        success_sender: Sender<Result<(), CommandError>>,
        upload_manager: &Mutex<upload::UploadManager>,
        canvas_size: (u32, u32),
    ) {
        debug!("Changing components. New size: {}", new_components.len());

        match Self::make_component_drawers(new_components, upload_manager, canvas_size) {
            Ok(drawers) => {
                *to_update = drawers;
                success_sender.send(Ok(())).unwrap();
            }
            Err(e) => {
                success_sender.send(Err(e.into())).unwrap();
            }
        }
    }

    /// Extract copies of the graphics components into a ComponentList and send it
    /// on the channel
    fn get_components(component_drawers: &[MovableComponentDrawer], response_sender: Sender<ComponentList>) {
        response_sender
            .send(
                component_drawers
                    .iter()
                    .map(|a| a.get_cloned_component())
                    .collect(),
            )
            .unwrap();
    }

    /// Transform the list of component descriptions into a list of component drawers. These drawers hold
    /// all the info from the original component description, but also hold state needed for rendering
    fn make_component_drawers(
        from_components: ComponentList,
        upload_manager: &Mutex<upload::UploadManager>,
        canvas_size: (u32, u32),
    ) -> Result<Vec<MovableComponentDrawer>, draw::DrawerCreationError> {
        from_components
            .into_iter()
            .map(|a| draw::MovableComponentDrawer::from_component(a, upload_manager, canvas_size))
            .collect()
    }
}
