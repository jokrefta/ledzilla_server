use crate::display::GraphicsDisplay;
use std::fmt::Debug;

#[derive(Debug)]
pub enum State<Disp>
where
    Disp: GraphicsDisplay + Debug,
{
    Stopped,
    RenderingStatic { display: Disp },
    RenderingDynamic { display: Disp },
}

#[derive(Debug)]
pub struct StateWrapper<Disp: GraphicsDisplay + Debug> {
    state: Option<State<Disp>>,
}

impl<Disp: GraphicsDisplay + Debug> StateWrapper<Disp> {
    pub fn new(s: State<Disp>) -> Self {
        Self { state: Some(s) }
    }

    // f takes in existing state and should return next state.
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(State<Disp>) -> State<Disp>,
    {
        self.state = Some(f(self.state.take().unwrap()));
    }

    pub fn get_ref(&self) -> &State<Disp> {
        self.state.as_ref().unwrap()
    }
    pub fn get_mut(&mut self) -> &State<Disp> {
        self.state.as_mut().unwrap()
    }
}
