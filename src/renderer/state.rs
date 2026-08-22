use crate::display::GraphicsDisplay;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct RenderingState<Disp>
where
    Disp: GraphicsDisplay + Debug,
{
    pub display: Disp,
    pub recent_frame_timestamps: Vec<std::time::Instant>,
}

#[derive(Debug)]
pub enum State<Disp>
where
    Disp: GraphicsDisplay + Debug,
{
    Stopped,
    Rendering(RenderingState<Disp>),
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
    /// f gets ownership of the existing state and should return next state.
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(State<Disp>) -> State<Disp>,
    {
        self.state = Some(f(self.state.take().unwrap()));
    }

    /// This function is for use when the state is already known to be the rendering state.
    /// It allows getting ownership and updating the inner data for the rendering state,  without changing
    /// to a different state.
    /// f gets ownership of the existing RenderingState and should return the next RenderingState.
    ///
    /// Current state must be Rendering or this function will panic.
    pub fn update_rendering_state<F>(&mut self, f: F)
    where
        F: FnOnce(RenderingState<Disp>) -> RenderingState<Disp>,
    {
        if let State::Rendering(rendering_state) = self.state.take().unwrap() {
            self.state = Some(State::Rendering(f(rendering_state)));
        } else {
            panic!();
        }
    }

    pub fn get_ref(&self) -> &State<Disp> {
        self.state.as_ref().unwrap()
    }
}
