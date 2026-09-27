use std::ops::{RangeFull, RangeInclusive};

fn vec2_from_direction(direction_degrees: u16) -> glam::Vec2 {
    match direction_degrees {
        0 => (1.0, 0.0).into(),
        90 => (0.0, 1.0).into(),
        180 => (-1.0, 0.0).into(),
        270 => (0.0, -1.0).into(),
        _ => glam::Vec2::from_angle(f32::from(direction_degrees).to_radians()),
    }
}

#[derive(Debug, Clone)]
enum IntegerRange {
    Bounded(RangeInclusive<i32>),
    Full(RangeFull),
}

impl IntegerRange {
    fn intersect(&self, other: &Self) -> Self {
        let Self::Bounded(self_range_inc) = self else {
            return other.clone();
        };
        let Self::Bounded(other_range_inc) = other else {
            return self.clone();
        };

        let result_start = *self_range_inc.start().max(other_range_inc.start());
        let result_end = *self_range_inc.end().min(other_range_inc.end());

        Self::Bounded(result_start..=result_end)
    }
}

fn mk_empty_range() -> IntegerRange {
    #[allow(clippy::reversed_empty_ranges)]
    IntegerRange::Bounded(1..=0)
}

/// Returns set of all integer k such that
/// lower <= k <= upper.
fn find_all_integers_between_two_float_bounds(lower: f32, upper: f32) -> IntegerRange {
    IntegerRange::Bounded(lower.ceil() as i32..=upper.floor() as i32)
}

/// Returns an iterator over all integer k (in order) such that
/// lower <= (k * value) <= upper.
/// If no such values exist, return an iterator that yields no items.
/// If the bounds are satisfied for all possible values of k, return None. This could happen
/// if k == 0.
///
/// Example: Calling with (value = 6.0, lower=-20.0, upper=10.0) would return an iterator that
/// yields {-3, -2, -1, 0, 1} because -3*6, -2*6, ..., 1*6 are all within the bounds.
fn find_integer_k_vals_for_scaling_in_bounds(value: f32, lower: f32, upper: f32) -> IntegerRange {
    if value == 0.0 {
        // Scaling it won't do anything, so it's either always in bounds or never in bounds.
        if lower <= value && value <= upper {
            return IntegerRange::Full(..);
        } else {
            return mk_empty_range();
        };
    }

    // lower <= (k * value) <= upper implies that
    // (lower / value) <= k <= (upper / value).
    // Unless value is negative, in which case the bounds will flip and we must reverse the max/min.
    let (mut lower_k_bound, mut upper_k_bound) = (lower / value, upper / value);
    if value < 0.0 {
        std::mem::swap(&mut lower_k_bound, &mut upper_k_bound);
    }

    find_all_integers_between_two_float_bounds(lower_k_bound, upper_k_bound)
}

/// Returns set of all v such that:
///   x_min <= v.x <= x_max and
///   y_min <= v.y <= y_max and
///   v can be expressed as (k * vector) for some integer k.
/// A None return means there are infinitely many such values (possible when |vector| == 0)
fn find_all_multiples_of_vector_within_bounding_box(
    vector: glam::Vec2,
    bbox_top_left: glam::Vec2,
    bbox_bot_right: glam::Vec2,
) -> Option<Vec<glam::Vec2>> {
    dbg!((vector, bbox_top_left, bbox_bot_right));
    // Find the possible range for k when considering only the x coordinates
    let k_bounds_from_x =
        find_integer_k_vals_for_scaling_in_bounds(vector.x, bbox_top_left.x, bbox_bot_right.x);

    // Find the possible range for k when considering only the y coordinates
    let k_bounds_from_y =
        find_integer_k_vals_for_scaling_in_bounds(vector.y, bbox_top_left.y, bbox_bot_right.y);

    let k_bounds = k_bounds_from_x.intersect(&k_bounds_from_y);

    match k_bounds {
        IntegerRange::Bounded(k_range) => Some(k_range.map(|k| k as f32 * vector).collect()),
        IntegerRange::Full(_) => None,
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
    periodicity_vector: glam::Vec2,

    /// Offset of an instance of the component, relative to the initial position.
    /// The positions of other instances of the component can be found from this value by adding
    /// or subtracting multiples of the periodicity vector.
    current_position: glam::Vec2,
}

impl ScrollingMovementTracker {
    pub fn new(config: ScrollingMovementConfig) -> Result<Self, String> {
        let translation_per_tick = vec2_from_direction(config.direction_degrees) * config.distance_per_tick;

        let periodicity_vector = vec2_from_direction(config.direction_degrees) * config.periodicity as f32;

        // The coordinate plane is shifted to be relative to the initial component position for
        // all calculations.
        // We determine the bounds for the object being displayable; if the position offset
        // falls outside of these computed bounds, then the object is entirely off screen.
        let displayable_x_bounds = (
            -(config.rendered_component_size.0 as f32) - config.initial_pos.x as f32,
            config.canvas_size.0 as f32 - config.initial_pos.x as f32,
        );
        let displayable_y_bounds = (
            -(config.rendered_component_size.1 as f32) - config.initial_pos.y as f32,
            config.canvas_size.1 as f32 - config.initial_pos.y as f32,
        );

        // start out with the initial position (offset 0,0)
        let current_position = glam::Vec2::ZERO;
        log::debug!(
            "ScrollingMovementTracker bounds: {:?} / {:?}",
            displayable_x_bounds,
            displayable_y_bounds
        );

        Ok(Self {
            translation_per_tick,
            displayable_x_bounds,
            displayable_y_bounds,
            periodicity_vector,
            current_position,
        })
    }

    /// Update current positions
    pub fn tick(&mut self) {
        // Update existing instance offset
        self.current_position += self.translation_per_tick;

        // No need for the position to keep getting bigger and bigger
        if is_instance_oob(
            self.displayable_x_bounds,
            self.displayable_y_bounds,
            self.current_position,
        ) {
            println!("{}", line!());
            let on_screen_positions = self.get_unrounded_instance_positions();
            dbg!(&on_screen_positions);
            if !on_screen_positions.is_empty() {
                println!("{}", line!());
                self.current_position = on_screen_positions[0];
            }
        }

        log::trace!(
            "ScrollingMovementTracker offset after update: {:?}",
            self.current_position
        );
    }

    /// Gets positions for each visible instance of the component.
    /// These are expressed as offsets relative to the initial component position,
    /// rounded to integer coordinates for rendering.
    fn get_unrounded_instance_positions(&self) -> Vec<glam::Vec2> {
        /* let B1, B2 be two points that form opposite corners of the displayable bounding box.
         *
         * Let inst_pos(k) = (current_position + k * periodicity) for some integer k.
         *
         * We want the set of all instance positions such that inst_pos(k) falls within the rectangle
         * formed by B1 and B2.
         *
         * So, we need to find the set of all (k * periodicity) offsets that fall within the rectanlge
         * formed by (B1 - current_position), (B2 - current_position).
         */

        let bbox_corner_1 = glam::vec2(self.displayable_x_bounds.0, self.displayable_y_bounds.0);
        let bbox_corner_2 = glam::vec2(self.displayable_x_bounds.1, self.displayable_y_bounds.1);
        dbg!((bbox_corner_1, bbox_corner_2));
        let result = find_all_multiples_of_vector_within_bounding_box(
            self.periodicity_vector,
            bbox_corner_1 - self.current_position,
            bbox_corner_2 - self.current_position,
        );
        dbg!(&result);

        match result {
            Some(positions) => positions.iter().map(|v| v + self.current_position).collect(),
            None => {
                // In this case the periodicity vector is zero magnitude, so just return a single position
                // instead of infinitely many copies on top of each other.
                vec![self.current_position]
            }
        }
    }

    pub fn get_display_instance_positions(&self) -> Vec<glam::IVec2> {
        // Round by adding 0.5 and flooring because we want .5 to round the same direction whether
        // positive or negative, which the round() method doesn't do.
        self.get_unrounded_instance_positions()
            .iter()
            .map(|v| (v + 0.5).floor().as_ivec2())
            .collect()
    }

    /// Calls the provided function `f` once for each current component instance.
    /// `f` takes a single argument, the pixel offset (relative to initial component position)
    /// of the instance.
    pub fn for_each_display_instance<F>(&self, mut f: F)
    where
        F: FnMut(glam::IVec2),
    {
        for pos in self.get_display_instance_positions() {
            f(pos)
        }
    }
}

/// Check if an instance of the drawn component is out of bounds.
///
/// This function just checks if the given point is within the bounds. If the provided bounds are
/// properly adjusted to account for the drawn component size, then it will tell us whether the
/// component is entirely out of bounds or if some portion of it is visible.
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
