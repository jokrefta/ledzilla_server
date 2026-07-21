use crate::{display::GraphicsDisplay, graphics_component::ComponentList};
use log::debug;
use std::fmt::Debug;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub enum RendererCommand {
    // Will likely mirror the API pretty closely
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

#[derive(Debug)]
enum RendererState<Disp: GraphicsDisplay + Debug> {
    Stopped,
    RenderingStatic { display: Disp },
    RenderingDynamic { display: Disp },
}

pub struct Renderer<F, Disp>
where
    F: FnMut() -> Disp,
    Disp: GraphicsDisplay + Debug,
{
    display_provider: F,
    state: RendererState<Disp>,
    components: ComponentList,
}

impl<F, Disp> Renderer<F, Disp>
where
    F: FnMut() -> Disp,
    Disp: GraphicsDisplay + Debug,
{
    pub fn new(display_provider: F) -> Self {
        Self {
            display_provider,
            state: RendererState::Stopped,
            components: ComponentList::new(),
        }
    }

    pub fn run(&mut self, command_receiver: Receiver<RendererCommand>) {
        // let mut state = RendererState::<Disp>::Stopped;
        // let mut components = ComponentList::new();

        loop {
            let command = self.run_until_next_command(&command_receiver);
            debug!("received {:?} in state {:?}", command, self.state);
            self.handle_command(command);
        }
    }

    // Run until a command is received, and return it. Since a command may cause a state transition, we
    // break from this function to handle it before re-entering in the updated state.
    fn run_until_next_command(&mut self, command_receiver: &Receiver<RendererCommand>) -> RendererCommand {
        // If we are rendering something dynamic, we want to keep doing work in this loop and
        // do a non-blocking check each iteration to see if a command has come in.
        // Otherwise, we want to block until a command has come in.
        if let RendererState::RenderingDynamic { display: _display } = &self.state {
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
    fn handle_command(&mut self, command: RendererCommand) {
        match (&self.state, command) {
            // -------------------------------------------------------------------------
            (RendererState::Stopped, RendererCommand::Start { response_sender }) => {
                self.state = self.start_up(response_sender);
            }
            // -------------------------------------------------------------------------
            (RendererState::Stopped, RendererCommand::Stop { response_sender }) => {
                // no-op
                response_sender.send(true).unwrap();
            }
            // -------------------------------------------------------------------------
            (
                RendererState::RenderingStatic { .. } | RendererState::RenderingDynamic { .. },
                RendererCommand::Stop { response_sender },
            ) => {
                self.state = self.shut_down(response_sender);
            }
            // -------------------------------------------------------------------------
            (
                RendererState::RenderingStatic { .. } | RendererState::RenderingDynamic { .. },
                RendererCommand::Start { response_sender },
            ) => {
                response_sender.send(true).unwrap();
            }
            // -------------------------------------------------------------------------
            (_, RendererCommand::GetComponents { response_sender }) => {
                response_sender.send(self.components.clone()).unwrap();
            }
            // -------------------------------------------------------------------------
            (
                _,
                RendererCommand::SetComponents {
                    response_sender,
                    components,
                },
            ) => {
                self.components = components;
                response_sender.send(true).unwrap();
            }
            // -------------------------------------------------------------------------
            #[allow(unreachable_patterns)]
            (curstate, cmd) => {
                debug!("Ignoring command {:?} in state {:?}", cmd, curstate);
            } // -------------------------------------------------------------------------
        }
    }

    // Create display and return next state
    fn start_up(&mut self, success_sender: Sender<bool>) -> RendererState<Disp> {
        // TODO handle transitioning into dynamic
        let display = (self.display_provider)();
        success_sender.send(true).unwrap();
        RendererState::RenderingStatic { display }
    }

    // Shut down display and return next state
    fn shut_down(&mut self, success_sender: Sender<bool>) -> RendererState<Disp> {
        success_sender.send(true).unwrap();
        RendererState::Stopped
    }
}
