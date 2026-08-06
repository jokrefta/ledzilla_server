use crate::display::GraphicsDisplay;
use std::fmt::Debug;

#[derive(Debug)]
pub enum State<Disp>
where
    Disp: GraphicsDisplay + Debug,
{
    Stopped,
    RenderingStatic { display: Disp },
    _RenderingDynamic { display: Disp },
}

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
            State::RenderingStatic { display } => State::RenderingStatic { display: f(display) },
            State::_RenderingDynamic { display } => State::_RenderingDynamic { display: f(display) },
        });
    }

    pub fn get_ref(&self) -> &State<Disp> {
        self.state.as_ref().unwrap()
    }
}
