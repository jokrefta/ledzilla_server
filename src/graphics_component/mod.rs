use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub use color::ColorSpec;
pub use draw::ComponentDrawer;
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

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Component {
    Image(Image),
    Text(Text),
    Line(Line),
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

    // TODO add tests for scroll when implemneted

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
}
