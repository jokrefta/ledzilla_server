use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub use color::ColorSpec;
pub use font::Alignment;
pub use motion::MotionConfig;

mod color;
pub mod draw;
mod font;
mod motion;

pub type ComponentList = Vec<Component>;

pub type Font = font::Font;

#[skip_serializing_none]
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Text {
    pub x: i32,
    pub y: i32,
    pub font: Font,
    pub content: String,
    pub color: ColorSpec,
    pub alignment: Alignment,

    pub motion_config: Option<MotionConfig>,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Image {
    pub x: i32,
    pub y: i32,
    pub source: String,

    pub frame_slowdown: Option<usize>,
    pub motion_config: Option<MotionConfig>,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Line {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub stroke_width: u32,
    pub color: ColorSpec,

    pub motion_config: Option<MotionConfig>,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub border_color: ColorSpec,
    pub border_width: u32,

    pub fill_color: Option<ColorSpec>,
    pub motion_config: Option<MotionConfig>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Component {
    Image(Image),
    Text(Text),
    Line(Line),
    Rectangle(Rectangle),
}

impl Component {
    fn get_motion_config(&self) -> Option<&MotionConfig> {
        match self {
            Component::Image(c) => c.motion_config.as_ref(),
            Component::Text(c) => c.motion_config.as_ref(),
            Component::Line(c) => c.motion_config.as_ref(),
            Component::Rectangle(c) => c.motion_config.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics_component::color;

    use super::*;

    fn mk_static_colorspec(r: u8, g: u8, b: u8) -> color::ColorSpec {
        ColorSpec::Static(color::StaticColorSpec {
            color: color::Color::from_rgb(r, g, b),
        })
    }

    fn mk_animated_colorspec(duration: usize, keyframes: Vec<(u8, color::Color)>) -> color::ColorSpec {
        ColorSpec::Animated(color::AnimatedColorSpec { duration, keyframes })
    }

    fn assert_failed_deserialization<'a, T>(as_json: &'a str)
    where
        T: std::fmt::Debug + PartialEq + serde::Deserialize<'a>,
    {
        let result: Result<T, serde_json::Error> = serde_json::from_str(as_json);
        // dbg!(&result);
        assert!(result.is_err());
    }

    fn test_deserialization<'a, T>(as_json: &'a str, as_rust: &T)
    where
        T: std::fmt::Debug + PartialEq + serde::Deserialize<'a>,
    {
        let deserialized: T = serde_json::from_str(as_json).unwrap();
        assert_eq!(*as_rust, deserialized);
    }

    fn test_ser_des<T>(as_rust: &T)
    where
        T: std::fmt::Debug + PartialEq + for<'a> serde::Deserialize<'a> + serde::Serialize,
    {
        // Since JSON is hard to compare in tests, test serialization by
        // converting to json and back to rust again. Not the best test... so also
        // print out the serialized version for manual inspection if desired.
        println!("Rust structure: {:?}", as_rust);
        let serialized = serde_json::to_string(&as_rust).unwrap();
        println!("--> As Json: {}", serialized);

        // Don't want to serialize optional types as none, should just leave them out
        assert!(!serialized.contains("null"));

        let deserialized_again: T = serde_json::from_str(&serialized).unwrap();
        assert_eq!(*as_rust, deserialized_again);
    }

    #[test]
    fn text() {
        let as_json = r##"{
            "type": "text",
            "x": 1,
            "y": 2,
            "content": "Hello World",
            "font": "mono_default_5x7",
            "color": {
                "type": "static",
                "color": "#FF0001"
            },
            "alignment": "Left"
        }"##;

        let as_rust = Component::Text(Text {
            x: 1,
            y: 2,
            content: "Hello World".to_string(),
            font: font::Font::mono_default_5x7,
            color: mk_static_colorspec(255, 0, 1),
            alignment: Alignment::Left,
            motion_config: None,
        });

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }

    #[test]
    fn image() {
        let as_json = r#"{
            "type": "image",
            "x": 1,
            "y": 2,
            "source": "logo.png"
        }"#;

        let as_rust = Component::Image(Image {
            x: 1,
            y: 2,
            source: String::from("logo.png"),
            frame_slowdown: None,
            motion_config: None,
        });

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }

    #[test]
    fn line() {
        let as_json = r##"{
            "type": "line",
            "x1": 10,
            "y1": 2,
            "x2": 5,
            "y2": 8,
            "stroke_width": 1,
            "color": {
                "type": "static",
                "color": "#FF0001"
            }
        }"##;

        let as_rust = Component::Line(Line {
            x1: 10,
            y1: 2,
            x2: 5,
            y2: 8,
            stroke_width: 1,
            color: mk_static_colorspec(255, 0, 1),
            motion_config: None,
        });

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }

    #[test]
    fn line_rgb_css_spec() {
        let as_json = r##"{
            "type": "line",
            "x1": 10,
            "y1": 2,
            "x2": 5,
            "y2": 8,
            "stroke_width": 1,
            "color": {
                "type": "static",
                "color": "rgb(255 0 1)"
            }
        }"##;

        let as_rust = Component::Line(Line {
            x1: 10,
            y1: 2,
            x2: 5,
            y2: 8,
            stroke_width: 1,
            color: mk_static_colorspec(255, 0, 1),
            motion_config: None,
        });

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }

    #[test]
    fn color_static() {
        let as_json = r##"{
                "type": "static",
                "color": "rgb(255 0 1)"
            }
        "##;

        let as_rust = mk_static_colorspec(255, 0, 1);

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }

    #[test]
    fn color_animated() {
        let as_json = r##"{
            "type": "animated",
            "duration": 30,
            "keyframes": [
                [0, "#FF0000"],
                [20, "rgb(0, 120, 0)"],
                [90, "#00F"],
                [100, "#FF0000"]
            ]
        }"##;

        let as_rust = mk_animated_colorspec(
            30,
            vec![
                (0, color::Color::from_rgb(255, 0, 0)),
                (20, color::Color::from_rgb(0, 120, 0)),
                (90, color::Color::from_rgb(0, 0, 255)),
                (100, color::Color::from_rgb(255, 0, 0)),
            ],
        );

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }

    #[test]
    fn color_animated_invalid_keyframe_range() {
        assert_failed_deserialization::<color::ColorSpec>(
            r##"{
            "type": "animated",
            "duration": 30,
            "keyframes": [
                [0, "#FF0000"],
                [20, "rgb(0, 120, 0)"],
                [90, "#00F"],
                [100, "#FF0000"],
                [102, "#FF0000"]
            ]
            }"##,
        );
    }

    #[test]
    fn color_animated_invalid_missing_endpoints() {
        assert_failed_deserialization::<color::ColorSpec>(
            r##"{
            "type": "animated",
            "duration": 30,
            "keyframes": [
                [0, "#FF0000"],
                [20, "rgb(0, 120, 0)"],
                [90, "#00F"],
            ]
            }"##,
        );
        assert_failed_deserialization::<color::ColorSpec>(
            r##"{
            "type": "animated",
            "duration": 30,
            "keyframes": [
                [20, "rgb(0, 120, 0)"],
                [90, "#00F"],
                [100, "#FF0000"],
            ]
            }"##,
        );
    }

    #[test]
    fn color_animated_invalid_keyframe_not_increasing() {
        assert_failed_deserialization::<color::ColorSpec>(
            r##"{
            "type": "animated",
            "duration": 30,
            "keyframes": [
                [0, "#FF0000"],
                [90, "#00F"],
                [90, "rgb(0, 120, 0)"],
                [100, "#FF0000"],
            ]
            }"##,
        );
    }

    #[test]
    fn scrolling_rect() {
        let as_json = r##"{
            "type": "rectangle",
            "x": 10,
            "y": 2,
            "width": 50,
            "height": 9,
            "border_color": {
                "type": "static",
                "color": "#A05500"
            },
            "border_width": 3,
            "fill_color": {
                "type": "static",
                "color": "ffa09e"
            },
            "motion_config": {
                "direction_degrees": 45,
                "distance_per_tick": 2,
                "periodicity": 200
            }
        }"##;

        let as_rust = Component::Rectangle(Rectangle {
            x: 10,
            y: 2,
            width: 50,
            height: 9,
            border_width: 3,
            border_color: mk_static_colorspec(160, 85, 0),
            fill_color: Some(mk_static_colorspec(255, 160, 158)),
            motion_config: Some(MotionConfig {
                direction_degrees: 45,
                distance_per_tick: 2.0,
                periodicity: 200,
            }),
        });

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }
}
