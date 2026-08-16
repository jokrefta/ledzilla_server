use std::collections::VecDeque;

fn vec2_from_direction(direction_degrees: u16) -> glam::Vec2 {
    match direction_degrees {
        0 => (1.0, 0.0).into(),
        90 => (0.0, 1.0).into(),
        180 => (-1.0, 0.0).into(),
        270 => (0.0, -1.0).into(),
        _ => glam::Vec2::from_angle(f32::from(direction_degrees).to_radians()),
    }
}

pub struct ScrollingMovementConfig {
    distance_per_tick: f32,
    direction_degrees: u16,
    periodicity: u32,
    canvas_size: (u32, u32),
    rendered_component_size: (u32, u32),
    initial_pos: glam::IVec2,
}

impl ScrollingMovementConfig {
    /// Arguments:
    /// * `motion_config`: Motion configuration parsed from the API component spec
    /// * `canvas_size`: (width, height)
    /// * `rendered_component_size`: (width, height)
    /// * `initial_pos`: (x, y) of TOP LEFT of object's bounding box for its initial
    ///   placement. All calculations and offsets will be relative to this point.
    pub fn from_parsed_motion_config(
        motion_config: &crate::graphics_component::MotionConfig,
        canvas_size: (u32, u32),
        rendered_component_size: (u32, u32),
        initial_position: (i32, i32),
    ) -> Self {
        Self {
            distance_per_tick: motion_config.distance_per_tick,
            direction_degrees: motion_config.direction_degrees,
            periodicity: motion_config.periodicity,
            canvas_size,
            rendered_component_size,
            initial_pos: initial_position.into(),
        }
    }
}

pub struct ScrollingMovementTracker {
    // direction: direction::CardinalDirection,
    translation_per_tick: glam::Vec2,
    /// Relative to the initial instance position
    displayable_x_bounds: (f32, f32),
    /// Relative to the initial instance position
    displayable_y_bounds: (f32, f32),
    repetition_offset: glam::Vec2,

    /// Offsets of all instances (duplicate copies) of the component. These are all relative
    /// to the initial position of the object. There may only be one of these, if the scroll
    /// periodicity is high enough that only one copy of the object is on screen at any time.
    ///
    /// The vec of offsets is kept sorted so that the instaces at the "leading edge"
    /// of the motion are at the end of the vec, and the instances at the "trailing" edge
    /// are at the start.
    current_offsets: VecDeque<glam::Vec2>,
}

impl ScrollingMovementTracker {
    pub fn new(config: ScrollingMovementConfig) -> Result<Self, String> {
        if config.distance_per_tick > config.periodicity as f32 {
            // not supported by tick() currently
            return Err("Unsupported when distance_per_tick > periodicity".to_string());
        }

        let mut current_offsets: VecDeque<glam::prelude::Vec2> = VecDeque::new();
        let translation_per_tick =
            vec2_from_direction(config.direction_degrees) * config.distance_per_tick;

        let repetition_offset = vec2_from_direction(config.direction_degrees) * config.periodicity as f32;

        // The coordinate plane is shifted to be relative to the initial component position for
        // all calculations.
        let displayable_x_bounds = (
            -(config.rendered_component_size.0 as f32) - config.initial_pos.x as f32,
            config.canvas_size.0 as f32 - config.initial_pos.x as f32,
        );
        let displayable_y_bounds = (
            -(config.rendered_component_size.1 as f32) - config.initial_pos.y as f32,
            config.canvas_size.1 as f32 - config.initial_pos.y as f32,
        );

        // prepend all instances that come "before" the initial position
        for i in 1.. {
            let instance = -repetition_offset * i as f32;
            if is_instance_oob(displayable_x_bounds, displayable_y_bounds, instance) {
                break; // We've gone past the end
            }
            log::trace!("Pushing offset {} to front", instance);
            current_offsets.push_front(instance);
        }
        // append the initial position and all instances that come "after" it
        for i in 0.. {
            let instance = repetition_offset * i as f32;
            if is_instance_oob(displayable_x_bounds, displayable_y_bounds, instance) {
                break; // We've gone past the end
            }
            log::trace!("Pushing offset {} to back", instance);
            current_offsets.push_back(instance);
        }
        log::debug!("ScrollingMovementTracker offsets to start: {:?}", current_offsets);
        log::debug!(
            "ScrollingMovementTracker bounds: {:?} / {:?}",
            displayable_x_bounds,
            displayable_y_bounds
        );

        Ok(Self {
            translation_per_tick,
            displayable_x_bounds,
            displayable_y_bounds,
            repetition_offset,
            current_offsets,
        })
    }

    /// Update current offsets
    pub fn tick(&mut self) {
        // Update existing instance offsets
        for v in self.current_offsets.iter_mut() {
            *v += self.translation_per_tick;
        }
        // log::trace!( "ScrollingMovementTracker offsets after update: {:?}", self.current_offsets);

        /*
         * If one has moved off screen, delete it. If it's time to add one on screen, create it.
         *
         * HOWEVER, never delete if it's the only instance left. We need at least one coordinate so
         * we have a reference point for creating the next one.
         * This means if the periodicity is large enough, there may be zero instances on the
         * screen for some frames, but self.current_offsets will always have at least one element.
         *
         * This assumes that only one will go off screen in a given tick - which is a safe
         * assumption iff the repetition separation is bigger than the distance per tick.
         */
        assert!(self.translation_per_tick.length_squared() < self.repetition_offset.length_squared());
        if self.current_offsets.len() > 1
            && is_instance_oob(
                self.displayable_x_bounds,
                self.displayable_y_bounds,
                *self.current_offsets.back().unwrap(),
            )
        {
            log::trace!("ScrollingMovementTracker - Destroy instance!");
            self.current_offsets.pop_back();
        }

        let potential_new_spawn = self.current_offsets.front().unwrap() - self.repetition_offset;
        // log::trace!("ScrollingMovementTracker - testing potential new spawn {}", potential_new_spawn);
        if !is_instance_oob(
            self.displayable_x_bounds,
            self.displayable_y_bounds,
            potential_new_spawn,
        ) {
            log::trace!("ScrollingMovementTracker - Spawn new!");
            self.current_offsets.push_front(potential_new_spawn);
        }
    }

    /// Gets offsets for each instance of the component.
    /// These are rounded to integer coordinates for rendering.
    pub fn get_offsets(&self) -> Vec<glam::IVec2> {
        // Round by adding 0.5 and flooring because we want .5 to round the same direction whether
        // positive or negative, which the round() method doesn't do.
        self.current_offsets
            .iter()
            .map(|v| (v + 0.5).floor().as_ivec2())
            .collect()
    }

    /// Calls the provided function `f` once for each current component instance.
    /// `f` takes a single argument, the offset (relative to initial component position)
    /// of the instance.
    pub fn for_each_instance<F>(&self, mut f: F)
    where
        F: FnMut(glam::IVec2),
    {
        for pos in self.get_offsets() {
            f(pos)
        }
    }
}

/// Check if an instance of the drawn component is out of bounds
fn is_instance_oob(
    drawable_x_bounds: (f32, f32),
    drawable_y_bounds: (f32, f32),
    instance_pos: glam::Vec2,
) -> bool {
    instance_pos.x < drawable_x_bounds.0
        || instance_pos.x > drawable_x_bounds.1
        || instance_pos.y < drawable_y_bounds.0
        || instance_pos.y > drawable_y_bounds.1
}
