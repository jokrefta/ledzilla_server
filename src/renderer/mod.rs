use crate::graphics_component;
use crate::renderer::state::{State, StateWrapper};
use crate::{
    display::GraphicsDisplay,
    graphics_component::{ComponentDrawer, ComponentList},
};
use log::debug;
use std::fmt::Debug;
use std::sync::mpsc::{Receiver, Sender};

mod graphics;
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
    // components: ComponentList,
    components: Vec<Box<dyn ComponentDrawer<Disp::DrawTarget>>>,
}

impl<F, Disp> Renderer<F, Disp>
where
    F: FnMut() -> Disp,
    Disp: GraphicsDisplay + Debug,
{
    pub fn new(display_provider: F) -> Self {
        Self {
            display_provider,
            state: StateWrapper::new(State::Stopped),
            components: Vec::new(),
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
        // If we are rendering something dynamic, we want to keep doing work in this loop and
        // do a non-blocking check each iteration to see if a command has come in.
        // Otherwise, we want to block until a command has come in.
        if let State::RenderingDynamic { display: _display } = self.state.get_ref() {
            loop {
                // Early return if command found
                match command_receiver.try_recv() {
                    Ok(c) => {
                        return c;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => (),
                    Err(_) => panic!(),
                };

                todo!("need to tick dynamic state and re-render");
            }
        } else {
            // Any static rendering has already been done (it happened when the corresponding command was handled).
            // So nothing to do besides wait for next command.
            return command_receiver.recv().unwrap();
        }
    }

    // Handle event, updating state if necessary
    fn handle_command(&mut self, command: Command) {
        self.state.update(|curstate| {
            // #[rustfmt::skip]
            let nextstate = match (curstate, command) {
                //__________________________________________________
                (State::Stopped, Command::Start { response_sender }) => {
                    Self::start_up(&mut self.display_provider, response_sender)
                }
                //__________________________________________________
                (State::Stopped, Command::Stop { response_sender }) => {
                    // no-op
                    response_sender.send(true).unwrap();
                    State::Stopped
                }
                //__________________________________________________
                (
                    State::RenderingStatic { .. } | State::RenderingDynamic { .. },
                    Command::Stop { response_sender },
                ) => Self::shut_down(response_sender),
                //__________________________________________________
                (
                    state @ (State::RenderingStatic { .. } | State::RenderingDynamic { .. }),
                    Command::Start { response_sender },
                ) => {
                    response_sender.send(true).unwrap();
                    state
                }
                //__________________________________________________
                (state, Command::GetComponents { response_sender }) => {
                    response_sender.send(self.components.clone()).unwrap();
                    state
                }
                //__________________________________________________
                (
                    state,
                    Command::SetComponents {
                        response_sender,
                        components,
                    },
                ) => {
                    self.components = components.into_iter().map(|a| Box::new(a.into())).collect();
                    response_sender.send(true).unwrap();
                    state
                }
                //__________________________________________________
                #[allow(unreachable_patterns)]
                (state, cmd) => {
                    debug!("Ignoring command {:?} in state {:?}", cmd, state);
                    state
                }
            };
            nextstate
        });
    }

    // Create display and return next state
    fn start_up(display_provider: &mut F, success_sender: Sender<bool>) -> State<Disp> {
        // TODO handle transitioning into dynamic
        let display = (display_provider)();
        success_sender.send(true).unwrap();
        State::RenderingStatic { display }
    }

    // Shut down display and return next state
    fn shut_down(success_sender: Sender<bool>) -> State<Disp> {
        success_sender.send(true).unwrap();
        State::Stopped
    }

    fn render_static(mut display: Disp, components: ComponentList) -> Disp {
        let canvas = display.get_draw_target();

        display.update_display()
    }
}
