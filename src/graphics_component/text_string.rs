use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;
use serde_with::{DeserializeFromStr, SerializeDisplay};

use std::{assert_matches, str::FromStr};

#[derive(Parser)]
#[grammar = "graphics_component/text_string_spec.pest"]
struct StringParser;

#[derive(Debug, PartialEq, Clone)]
pub enum TextTemplatePart {
    Str(String),
    Time(Vec<chrono::format::Item<'static>>),
}

impl TextTemplatePart {
    fn from_pest_component(component: pest::iterators::Pair<Rule>) -> Result<Self> {
        //dbg!(&component);
        let component_variant = component.into_inner().next().unwrap();
        //dbg!(&component_variant);
        let parsed_template = match component_variant.as_rule() {
            Rule::plain_text_component => {
                let raw_string = component_variant.as_str().to_string();
                let unescaped_string = raw_string.replace("$$", "$");
                Self::Str(unescaped_string)
            }
            Rule::special_component => {
                let body = component_variant.into_inner().next().unwrap();
                assert_matches!(body.as_rule(), Rule::special_component_body);

                let body_variant = body.into_inner().next().unwrap();
                match body_variant.as_rule() {
                    Rule::time_string => {
                        // body_variant has format: "TIME <arg>"
                        // arg is the strftime format string
                        let arg = body_variant.into_inner().next().unwrap();
                        assert_matches!(arg.as_rule(), Rule::time_string_args);

                        Self::Time(chrono::format::StrftimeItems::new(arg.as_str()).parse_to_owned()?)
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };
        Ok(parsed_template)
    }
}

#[derive(Debug, PartialEq, Clone, SerializeDisplay, DeserializeFromStr)]
pub struct TextStringTemplate {
    original: String,
    parts: Vec<TextTemplatePart>,
}

// For displaying the format template. To actually perform the pattern substitution and get the text to display on the LED, call
// make_string_from_template.
impl std::fmt::Display for TextStringTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}

impl FromStr for TextStringTemplate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parsed_parts = Vec::<TextTemplatePart>::new();

        let mut pairs = StringParser::parse(Rule::text_spec, s)?;
        let full_text = pairs.next().unwrap();

        for component in full_text.into_inner() {
            match component.as_rule() {
                Rule::EOI => (),
                Rule::component => {
                    parsed_parts.push(TextTemplatePart::from_pest_component(component)?);
                }
                _ => unreachable!(),
            }
        }

        Ok(Self {
            original: s.to_string(),
            parts: parsed_parts,
        })
    }
}

pub fn make_string_from_template<F: FnMut() -> chrono::DateTime<chrono::Local>>(
    template: &TextStringTemplate,
    mut time_getter: F,
) -> String {
    use std::fmt::Write;
    let mut out_str = String::new();

    for part in &template.parts {
        match part {
            TextTemplatePart::Str(s) => write!(out_str, "{}", s).unwrap(),
            TextTemplatePart::Time(items) => {
                let formatted_time = time_getter().format_with_items(items.as_slice().iter());
                write!(out_str, "{}", formatted_time).unwrap()
            }
        };
    }

    out_str
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;
    use std::assert_matches;

    fn get_test_time() -> chrono::DateTime<chrono::Local> {
        chrono::Local
            .from_local_datetime(&chrono::NaiveDateTime::default())
            .unwrap()
    }

    #[test]
    fn parse_plain_string() {
        let test_string: String = "Hello world".into();

        let result = TextStringTemplate::from_str(&test_string);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.original, test_string);

        assert_eq!(template.parts.len(), 1);
        let TextTemplatePart::Str(s) = &template.parts[0] else {
            panic!()
        };

        assert_eq!(s, "Hello world");
    }

    #[test]
    fn parse_time_string() {
        let test_string: String = "${TIME %Y-%m-%d %H:%M:%S}".into();

        let result = TextStringTemplate::from_str(&test_string);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.original, test_string);

        assert_eq!(template.parts.len(), 1);
        assert_matches!(template.parts[0], TextTemplatePart::Time(_));

        assert_eq!(
            make_string_from_template(&template, get_test_time),
            "1970-01-01 00:00:00"
        );
    }

    #[test]
    fn parse_invalid_strftime_string() {
        let test_string: String = "${TIME %o%O%`}".into();

        let result = TextStringTemplate::from_str(&test_string);
        assert!(result.is_err());
    }

    #[test]
    fn parse_time_string_plus_text_with_escaped_dollar() {
        let test_string: String = "uuuu${TIME %Y-%m-%d %H:%M:%S}  asdf }{}$$ q".into();

        let result = TextStringTemplate::from_str(&test_string);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.original, test_string);

        assert_eq!(template.parts.len(), 3);
        // segment 1 - text
        let TextTemplatePart::Str(s) = &template.parts[0] else {
            panic!()
        };
        assert_eq!(s, "uuuu");
        // segment 2 - time
        assert_matches!(template.parts[1], TextTemplatePart::Time(_));
        // segment 3 - text
        let TextTemplatePart::Str(s) = &template.parts[2] else {
            panic!()
        };
        assert_eq!(s, "  asdf }{}$ q");

        assert_eq!(
            make_string_from_template(&template, get_test_time),
            "uuuu1970-01-01 00:00:00  asdf }{}$ q"
        );
    }

    #[test]

    fn parse_empty_string() {
        let test_string: String = "".into();

        let result = TextStringTemplate::from_str(&test_string);
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.original, test_string);

        assert_eq!(make_string_from_template(&template, get_test_time), "");
    }
}
