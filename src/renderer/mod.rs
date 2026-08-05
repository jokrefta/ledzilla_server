use crate::renderer::state::{State, StateWrapper};
use crate::{
    display::GraphicsDisplay,
    graphics_component::{ComponentDrawer, ComponentList},
};
use crate::{graphics_component, upload};
use log::debug;
use std::fmt::Debug;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

mod state;

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
        response_sender: Sender<bool>,
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
    components: Vec<Box<dyn ComponentDrawer<Disp::DrawTarget>>>,
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

        match self.state.get_ref() {
            State::Stopped => command_receiver.recv().unwrap(),
            // Rendering requires ownership of display, so we must use update_display here as opposed to
            // matching display by reference in the pattern.
            State::RenderingStatic { .. } => {
                self.state
                    .update_display(|display| Self::render(&mut self.components, display));
                command_receiver.recv().unwrap()
            }
            State::RenderingDynamic { .. } => {
                loop {
                    // Early return if command found
                    match command_receiver.try_recv() {
                        Ok(c) => {
                            return c;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => (),
                        Err(_) => panic!(),
                    }

                    todo!("need to tick dynamic state and re-render");
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
                    Self::set_components(&mut self.components, components, response_sender);
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
        // TODO handle transitioning into dynamic
        let display = (display_provider)();
        success_sender.send(true).unwrap();
        State::RenderingStatic { display }
    }

    /// Shut down display and return next state. Send a true response on the channel.
    fn shut_down(success_sender: Sender<bool>) -> State<Disp> {
        success_sender.send(true).unwrap();
        State::Stopped
    }

    /// Take the display, draw onto it, update it, and return it back.
    fn render(components: &mut Vec<Box<dyn ComponentDrawer<Disp::DrawTarget>>>, mut display: Disp) -> Disp {
        debug!("rendering");
        let canvas = display.get_draw_target();
        for drawer in components {
            drawer.draw_next_frame(canvas);
        }

        display.update_display()
    }

    /// Update the given components and send a true response on the channel.
    fn set_components(
        to_update: &mut Vec<Box<dyn ComponentDrawer<Disp::DrawTarget>>>,
        new_components: ComponentList,
        success_sender: Sender<bool>,
    ) {
        debug!("Changing components. New size: {}", new_components.len());
        *to_update = new_components
            .into_iter()
            .map(|a| graphics_component::draw::into_drawer(a))
            .collect();
        success_sender.send(true).unwrap();
    }

    /// Extract copies of the graphics components into a ComponentList and send it
    /// on the channel
    fn get_components(
        component_drawers: &Vec<Box<dyn ComponentDrawer<Disp::DrawTarget>>>,
        response_sender: Sender<ComponentList>,
    ) {
        response_sender
            .send(
                component_drawers
                    .iter()
                    .map(|a| a.get_cloned_component())
                    .collect(),
            )
            .unwrap();
    }
}
