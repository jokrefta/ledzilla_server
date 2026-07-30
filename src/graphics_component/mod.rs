use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

pub use color::ColorSpec;
pub use font::Alignment;
pub use draw::ComponentDrawer;

mod color;
mod font;
pub mod draw;

pub type ComponentList = Vec<Component>;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[skip_serializing_none]
pub struct CommonProperties {
    pub x: u32,
    pub y: u32,
    pub scroll: Option<()>,
}

pub type Font = font::Font;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Text {
    pub common_properties: CommonProperties,
    pub font: Font,
    pub content: String,
    pub color: ColorSpec,
    pub alignment: Alignment,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Image {
    pub common_properties: CommonProperties,
    pub source: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Line {
    pub common_properties: CommonProperties,
    pub delta_x: i32,
    pub delta_y: i32,
    pub stroke_width: u32,
    pub color: ColorSpec,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Component {
    Image(Image),
    Text(Text),
    Line(Line),
}

pub fn is_static(_c: Component) -> bool {
    true // Once we support animated components, this will change
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let serialized = serde_json::to_string(&as_rust).unwrap();
        println!("{}", serialized);
        let deserialized_again: T = serde_json::from_str(&serialized).unwrap();
        assert_eq!(*as_rust, deserialized_again);
    }

    // TODO add tests for scroll when implemneted

    #[test]
    fn text() {
        let as_json = r##"{
            "type": "text",
            "common_properties": {
                "x": 1,
                "y": 2
            },
            "content": "Hello World",
            "font": "mono_default_5x7",
            "color": "#FF0001",
            "alignment": "Left"
        }"##;

        let as_rust = Component::Text(Text {
            common_properties: CommonProperties {
                x: 1,
                y: 2,
                scroll: None,
            },
            content: "Hello World".to_string(),
            font: font::Font::mono_default_5x7,
            color: ColorSpec::from_rgb(255, 0, 1),
            alignment: Alignment::Left,
        });

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }

    #[test]
    fn image() {
        let as_json = r#"{
            "type": "image",
            "common_properties": {
                "x": 1,
                "y": 2
            },
            "source": "logo.png"
        }"#;

        let as_rust = Component::Image(Image {
            common_properties: CommonProperties {
                x: 1,
                y: 2,
                scroll: None,
            },
            source: String::from("logo.png"),
        });

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }

    #[test]
    fn line() {
        let as_json = r##"{
            "type": "line",
            "common_properties": {
                "x": 10,
                "y": 2
            },
            "delta_x": 5,
            "delta_y": -8,
            "stroke_width": 1,
            "color": "#FF0001"
        }"##;

        let as_rust = Component::Line(Line {
            common_properties: CommonProperties {
                x: 10,
                y: 2,
                scroll: None,
            },
            delta_x: 5,
            delta_y: -8,
            stroke_width: 1,
            color: ColorSpec::from_rgb(255, 0, 1),
        });

        test_deserialization(as_json, &as_rust);
        test_ser_des(&as_rust);
    }
}
