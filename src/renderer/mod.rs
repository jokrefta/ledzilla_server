use self::state::{State, StateWrapper};
use crate::{
    display::GraphicsDisplay,
    graphics_component::draw,
    graphics_component::{ComponentDrawer, ComponentList},
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
    components: Vec<ComponentDrawer>,
    upload_manager: Arc<Mutex<upload::UploadManager>>,
}

impl<F, Disp> Renderer<F, Disp>
where
    F: FnMut() -> Disp,
    Disp: GraphicsDisplay + Debug,
{
    pub fn new(display_provider: F, upload_manager: Arc<Mutex<upload::UploadManager>>) -> Self {
        Self {
            display_provider,
            state: StateWrapper::new(State::Stopped),
            components: Vec::new(),
            upload_manager,
        }
    }

    pub fn run(&mut self, command_receiver: Receiver<Command>) {
        loop {
            let command = self.run_until_next_command(&command_receiver);
            debug!("received {:?} in state {:?}", command, self.state);
            self.handle_command(command);
        }
    }

    // Run until a command is received, and return it. Since a command may cause a state transition, we
    // break from this function to handle it before re-entering in the updated state.
    fn run_until_next_command(&mut self, command_receiver: &Receiver<Command>) -> Command {
        // If we are rendering something dynamic, we want to keep doing work in a loop and
        // do a non-blocking check each iteration to see if a command has come in.
        // Otherwise, we want to block until a command has come in.

        // Note the use of update_display allows us to render without having ownership of the display.

        match self.state.get_ref() {
            State::Stopped => command_receiver.recv().unwrap(),
            State::RenderingStatic { .. } => {
                self.state
                    .update_display(|display| Self::render(&mut self.components, display));
                command_receiver.recv().unwrap()
            }
            State::RenderingDynamic { .. } => {
                loop {
                    self.state
                        .update_display(|display| Self::render(&mut self.components, display));

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
                    Self::start_up(&mut self.display_provider, &self.components, response_sender)
                }
                //__________________________________________________
                (State::Stopped, Command::Stop {response_sender}) => {
                    // no-op
                    response_sender.send(true).unwrap();
                    State::Stopped
                }
                //__________________________________________________
                (State::RenderingStatic {..} | State::RenderingDynamic {..}, Command::Stop {response_sender}) => {
                    Self::shut_down(response_sender)
                }
                //__________________________________________________
                (state @ (State::RenderingStatic {..} | State::RenderingDynamic {..}), Command::Start {response_sender}) => {
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
                        &*self.upload_manager,
                        state
                    )
                }
                // No default case - we want every command to be responded to.
            }
        });
    }

    // These are implemented as associated functions rather than methods because
    // they are called from a context where self.state has already been borrowed exclusively
    // so we cannot borrow self.

    /// Create display and return next state. Send a true response on the channel.
    fn start_up(
        display_provider: &mut F,
        components: &[ComponentDrawer],
        success_sender: Sender<bool>,
    ) -> State<Disp> {
        let display = (display_provider)();
        success_sender.send(true).unwrap();

        if components.iter().all(|c| c.is_static()) {
            State::RenderingStatic { display }
        } else {
            State::RenderingDynamic { display }
        }
    }

    /// Shut down display and return next state. Send a true response on the channel.
    fn shut_down(success_sender: Sender<bool>) -> State<Disp> {
        success_sender.send(true).unwrap();
        State::Stopped
    }

    /// Take the display, draw onto it, update it, and return it back.
    fn render(components: &mut Vec<ComponentDrawer>, mut display: Disp) -> Disp {
        debug!("rendering");
        let canvas = display.get_draw_target();
        for drawer in components {
            drawer.draw_next_frame(canvas);
        }

        display.update_display()
    }

    /// Update the given components and send a success response on the channel.
    fn set_components(
        to_update: &mut Vec<ComponentDrawer>,
        new_components: ComponentList,
        success_sender: Sender<Result<(), CommandError>>,
        upload_manager: &Mutex<upload::UploadManager>,
        curstate: State<Disp>,
    ) -> State<Disp> {
        debug!("Changing components. New size: {}", new_components.len());

        match Self::make_component_drawers(new_components, upload_manager) {
            Ok(drawers) => {
                *to_update = drawers;
                success_sender.send(Ok(())).unwrap();

                match curstate {
                    State::Stopped => curstate,
                    State::RenderingStatic { display } | State::RenderingDynamic { display } => {
                        if to_update.iter().all(|c| c.is_static()) {
                            State::RenderingStatic { display }
                        } else {
                            State::RenderingDynamic { display }
                        }
                    }
                }
            }
            Err(e) => {
                success_sender.send(Err(e.into())).unwrap();
                curstate
            }
        }
    }

    /// Extract copies of the graphics components into a ComponentList and send it
    /// on the channel
    fn get_components(component_drawers: &[ComponentDrawer], response_sender: Sender<ComponentList>) {
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
    ) -> Result<Vec<ComponentDrawer>, draw::DrawerCreationError> {
        from_components
            .into_iter()
            .map(|a| draw::into_drawer(a, upload_manager))
            .collect()
    }
}
