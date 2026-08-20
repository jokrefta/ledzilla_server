use crate::display::GraphicsDisplay;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum State<Disp>
where
    Disp: GraphicsDisplay + Debug,
{
    Stopped,
    Rendering { display: Disp },
}

impl<Disp> Display for State<Disp>
where
    Disp: GraphicsDisplay + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Stopped => write!(f, "Stopped"),
            State::Rendering { .. } => write!(f, "Rendering"),
        }
    }
}

/// Wraps the state.
/// The underlying state enum is wrapped in an Option. That is because we want
/// to consume the state on transitions, even when only a mut reference is held.
/// This can be done using take() with an option.
/// To enforce the invariant that state can never be None, this wrapper was created.
#[derive(Debug)]
pub struct StateWrapper<Disp: GraphicsDisplay + Debug> {
    state: Option<State<Disp>>,
}

impl<Disp: GraphicsDisplay + Debug> StateWrapper<Disp> {
    pub fn new(s: State<Disp>) -> Self {
        Self { state: Some(s) }
    }

    /// Pass a closure to update the state.
    /// f takes in existing state and should return next state.
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(State<Disp>) -> State<Disp>,
    {
        self.state = Some(f(self.state.take().unwrap()));
    }

    /// Pass a closure to update the display (if present).
    /// If in Stopped state, does nothing.
    pub fn update_display<F>(&mut self, f: F)
    where
        F: FnOnce(Disp) -> Disp,
    {
        self.update(|state| match state {
            State::Stopped => {
                log::warn!("Nothing to do for state update in state Stopped");
                state
            }
            State::Rendering { display } => State::Rendering { display: f(display) },
        });
    }

    pub fn get_ref(&self) -> &State<Disp> {
        self.state.as_ref().unwrap()
    }
}
