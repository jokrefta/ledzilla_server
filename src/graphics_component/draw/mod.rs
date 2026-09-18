use embedded_graphics::pixelcolor::Rgb888;
use thiserror::Error;

use std::fmt::Debug;
use std::sync::Mutex;

use crate::upload;

mod color_state;

mod movement_state;

/// A struct responsible for drawing a component and keeping track of its movement (e.g, scrolling) if configured.
///
/// Each component is turned into a ComponentDrawer which manages rendering for that component instance.
///
/// If scrolling is enabled for a particular component, there may be need to render multiple of them in one
/// frmae. For example, if the text "HELLO" is scrolling to the left (with a periodicity configured to match the
/// screen width), we might need to render the "ELLO" on the left edge of the screen as well as the "H"
/// on the right edge as it wraps around. This is handled by using a movement tracker to keep track of
/// how many copies of a component should be rendered, and where. This logic is kept separate from the
/// ComponentDrawer - which just knows how to render itself in a given location.
///
pub struct MovableComponentDrawer {
    drawer: component_drawer::ComponentDrawer,
    movement_tracker: Option<movement_state::ScrollingMovementTracker>,
}

impl MovableComponentDrawer {
    pub fn from_component(
        comp: super::Component,
        upload_manager: &Mutex<upload::UploadManager>,
        canvas_size: (u32, u32),
    ) -> Result<Self, DrawerCreationError> {
        let opt_motion_config = comp.get_motion_config().cloned();

        let drawer = component_drawer::ComponentDrawer::from_component(comp, upload_manager)?;

        let movement_tracker = if let Some(motion_config) = opt_motion_config {
            let bbox = drawer.get_bbox();
            let rendered_component_size = (bbox.size.width, bbox.size.height);
            let initial_position = (bbox.top_left.x, bbox.top_left.y);

            Some(
                movement_state::ScrollingMovementTracker::new(
                    movement_state::ScrollingMovementConfig::from_parsed_motion_config(
                        &motion_config,
                        canvas_size,
                        rendered_component_size,
                        initial_position,
                    ),
                )
                .map_err(
                    // Currently the movement tracker construction fails only when something
                    // about the motion config is unsupported, so treat as an invalid component spec
                    DrawerCreationError::BadComponentSpec,
                )?,
            )
        } else {
            None
        };

        Ok(Self {
            drawer,
            movement_tracker,
        })
    }

    /// Draw frame and advance internal state by one frame.
    pub fn draw_next_frame<T>(&mut self, target: &mut T)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        if let Some(tracker) = &mut self.movement_tracker {
            tracker.for_each_instance(|offset| self.drawer.draw(target, offset));
            tracker.tick();
        } else {
            self.drawer.draw(target, (0, 0).into());
        }
        self.drawer.advance_frame();
    }

    pub fn get_cloned_component(&self) -> super::Component {
        self.drawer.get_cloned_component()
    }
}

#[derive(Debug, Error)]
pub enum DrawerCreationError {
    #[error("Failed to construct drawer due to bad component spec: {0}")]
    BadComponentSpec(String),
    #[error("{0}")]
    _GeneralError(String),
}

mod component_drawer;
