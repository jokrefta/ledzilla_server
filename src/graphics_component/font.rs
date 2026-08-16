use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::Rgb888};
use embedded_graphics::{mono_font::ascii as eg_mono_ascii, text as eg_text};
use serde::{Deserialize, Serialize};
use u8g2_fonts::U8g2TextStyle;
use u8g2_fonts::fonts::*; // The all have u8g2 prefix in the name anyway

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl From<Alignment> for eg_text::Alignment {
    fn from(value: Alignment) -> Self {
        match value {
            Alignment::Left => eg_text::Alignment::Left,
            Alignment::Center => eg_text::Alignment::Center,
            Alignment::Right => eg_text::Alignment::Right,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EgCharStyle {
    EgMono(MonoTextStyle<'static, Rgb888>),
    U8G2(U8g2TextStyle<Rgb888>),
}

impl EgCharStyle {
    fn from_eg_mono(font: &'static embedded_graphics::mono_font::MonoFont<'static>, color: Rgb888) -> Self {
        EgCharStyle::EgMono(MonoTextStyle::new(font, color))
    }

    fn from_u8g2<F: u8g2_fonts::Font>(font: F, color: Rgb888) -> Self {
        EgCharStyle::U8G2(U8g2TextStyle::new(font, color))
    }
}

// The wrapped types implement TextRenderer, but we must reimplement it for the wrapper to allow it to be used as
// a text style.
impl eg_text::renderer::TextRenderer for EgCharStyle {
    type Color = Rgb888;

    fn draw_string<D>(
        &self,
        text: &str,
        position: embedded_graphics::prelude::Point,
        baseline: eg_text::Baseline,
        target: &mut D,
    ) -> Result<embedded_graphics::prelude::Point, D::Error>
    where
        D: embedded_graphics::prelude::DrawTarget<Color = Self::Color>,
    {
        match self {
            EgCharStyle::EgMono(style) => style.draw_string(text, position, baseline, target),
            EgCharStyle::U8G2(style) => style.draw_string(text, position, baseline, target),
        }
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: embedded_graphics::prelude::Point,
        baseline: eg_text::Baseline,
        target: &mut D,
    ) -> Result<embedded_graphics::prelude::Point, D::Error>
    where
        D: embedded_graphics::prelude::DrawTarget<Color = Self::Color>,
    {
        match self {
            EgCharStyle::EgMono(style) => style.draw_whitespace(width, position, baseline, target),
            EgCharStyle::U8G2(style) => style.draw_whitespace(width, position, baseline, target),
        }
    }

    fn measure_string(
        &self,
        text: &str,
        position: embedded_graphics::prelude::Point,
        baseline: eg_text::Baseline,
    ) -> eg_text::renderer::TextMetrics {
        match self {
            EgCharStyle::EgMono(style) => style.measure_string(text, position, baseline),
            EgCharStyle::U8G2(style) => style.measure_string(text, position, baseline),
        }
    }

    fn line_height(&self) -> u32 {
        match self {
            EgCharStyle::EgMono(style) => style.line_height(),
            EgCharStyle::U8G2(style) => style.line_height(),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize, Serialize, strum_macros::VariantNames)]
pub enum FontName {
    mono_default_4x6,
    mono_default_5x7,
    mono_default_5x8,
    mono_default_6x10,
    mono_default_6x12,
    mono_default_6x13,
    mono_default_6x13_bold,
    mono_default_6x13_italic,
    mono_default_6x9,
    mono_default_7x13,
    mono_default_7x13_bold,
    mono_default_7x13_italic,
    mono_default_7x14,
    mono_default_7x14_bold,
    mono_default_8x13,
    mono_default_8x13_bold,
    mono_default_8x13_italic,
    mono_default_9x15,
    mono_default_9x15_bold,
    mono_default_9x18,
    mono_default_9x18_bold,
    mono_default_10x20,

    // u8g2 - bitfontmaker2
    adventurer_11,
    fancypixels_9,
    smallsimple_6,
    simple1_7,
    bpixel_7,
    bpixel_12_bold,
    sticker100_11,
    greenbloodserif_12,
    commodore64_12,

    // u8g2 - lucida
    lucida_serif_9,
    lucida_serif_10,
    lucida_serif_13,
    lucida_serif_15,
    lucida_serif_18,
    lucida_serif_25,
    lucida_serif_9_bold,
    lucida_serif_10_bold,
    lucida_serif_13_bold,
    lucida_serif_15_bold,
    lucida_serif_18_bold,
    lucida_serif_25_bold,
    lucida_serif_9_italic,
    lucida_serif_10_italic,
    lucida_serif_13_italic,
    lucida_serif_15_italic,
    lucida_serif_18_italic,
    lucida_serif_25_italic,

    lucida_sans_9,
    lucida_sans_10,
    lucida_sans_13,
    lucida_sans_15,
    lucida_sans_18,
    lucida_sans_25,
    lucida_sans_9_bold,
    lucida_sans_10_bold,
    lucida_sans_13_bold,
    lucida_sans_15_bold,
    lucida_sans_18_bold,
    lucida_sans_25_bold,
    lucida_sans_9_italic,
    lucida_sans_10_italic,
    lucida_sans_13_italic,
    lucida_sans_15_italic,
    lucida_sans_18_italic,
    lucida_sans_25_italic,

    // u8g2 - spleen
    spleen_mono_6,
    spleen_mono_8,
    spleen_mono_15,
    spleen_mono_20,

    // u8g2 - fontstruct
    trixel_sq_5,
    maniac_23,
    bubble_18,

    // u8g2 - pentacom
    bittypewriter_7,
    helvetipixeloutline_12,
    unnameddosfont_12,

    // u8g2 - profont
    profont_6,
    profont_8,
    profont_9,
    profont_14,
    profont_19,

    // u8g2- gilesbooth
    sirclive_7,
}

impl FontName {
    pub fn get_eg_font(&self, color: Rgb888) -> EgCharStyle {
        #[rustfmt::skip]
        // Intermediate variable b/c we can't disable rustfmt on expressions, only statements
        let font = match self {
            Self::mono_default_4x6            => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_4X6, color),
            Self::mono_default_5x7            => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_5X7, color),
            Self::mono_default_5x8            => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_5X8, color),
            Self::mono_default_6x10           => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_6X10, color),
            Self::mono_default_6x12           => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_6X12, color),
            Self::mono_default_6x13           => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_6X13, color),
            Self::mono_default_6x13_bold      => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_6X13_BOLD, color),
            Self::mono_default_6x13_italic    => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_6X13_ITALIC, color),
            Self::mono_default_6x9            => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_6X9, color),
            Self::mono_default_7x13           => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_7X13, color),
            Self::mono_default_7x13_bold      => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_7X13_BOLD, color),
            Self::mono_default_7x13_italic    => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_7X13_ITALIC, color),
            Self::mono_default_7x14           => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_7X14, color),
            Self::mono_default_7x14_bold      => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_7X14_BOLD, color),
            Self::mono_default_8x13           => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_8X13, color),
            Self::mono_default_8x13_bold      => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_8X13_BOLD, color),
            Self::mono_default_8x13_italic    => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_8X13_ITALIC, color),
            Self::mono_default_9x15           => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_9X15, color),
            Self::mono_default_9x15_bold      => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_9X15_BOLD, color),
            Self::mono_default_9x18           => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_9X18, color),
            Self::mono_default_9x18_bold      => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_9X18_BOLD, color),
            Self::mono_default_10x20          => EgCharStyle::from_eg_mono(&eg_mono_ascii::FONT_10X20, color),

            // u8g2 - bitfontmaker2
            Self:: adventurer_11              => EgCharStyle::from_u8g2(u8g2_font_adventurer_tf, color),
            Self::fancypixels_9               => EgCharStyle::from_u8g2(u8g2_font_fancypixels_tr, color),
            Self::smallsimple_6               => EgCharStyle::from_u8g2(u8g2_font_smallsimple_tr, color),
            Self::simple1_7                   => EgCharStyle::from_u8g2(u8g2_font_simple1_tr, color),
            Self::bpixel_7                    => EgCharStyle::from_u8g2(u8g2_font_bpixel_tr, color),
            Self::bpixel_12_bold              => EgCharStyle::from_u8g2(u8g2_font_bpixeldouble_tr, color),
            Self::sticker100_11               => EgCharStyle::from_u8g2(u8g2_font_sticker100complete_tr, color),
            Self::greenbloodserif_12          => EgCharStyle::from_u8g2(u8g2_font_greenbloodserif2_tr, color),
            Self::commodore64_12              => EgCharStyle::from_u8g2(u8g2_font_commodore64_tr, color),

            // u8g2 - lucida
            Self::lucida_serif_9              => EgCharStyle::from_u8g2(u8g2_font_lubR08_tf, color),
            Self::lucida_serif_10             => EgCharStyle::from_u8g2(u8g2_font_lubR10_tf, color),
            Self::lucida_serif_13             => EgCharStyle::from_u8g2(u8g2_font_lubR12_tf, color),
            Self::lucida_serif_15             => EgCharStyle::from_u8g2(u8g2_font_lubR14_tf, color),
            Self::lucida_serif_18             => EgCharStyle::from_u8g2(u8g2_font_lubR18_tf, color),
            Self::lucida_serif_25             => EgCharStyle::from_u8g2(u8g2_font_lubR24_tf, color),
            Self::lucida_serif_9_bold         => EgCharStyle::from_u8g2(u8g2_font_lubB08_tf, color),
            Self::lucida_serif_10_bold        => EgCharStyle::from_u8g2(u8g2_font_lubB10_tf, color),
            Self::lucida_serif_13_bold        => EgCharStyle::from_u8g2(u8g2_font_lubB12_tf, color),
            Self::lucida_serif_15_bold        => EgCharStyle::from_u8g2(u8g2_font_lubB14_tf, color),
            Self::lucida_serif_18_bold        => EgCharStyle::from_u8g2(u8g2_font_lubB18_tf, color),
            Self::lucida_serif_25_bold        => EgCharStyle::from_u8g2(u8g2_font_lubB24_tf, color),
            Self::lucida_serif_9_italic       => EgCharStyle::from_u8g2(u8g2_font_lubI08_tf, color),
            Self::lucida_serif_10_italic      => EgCharStyle::from_u8g2(u8g2_font_lubI10_tf, color),
            Self::lucida_serif_13_italic      => EgCharStyle::from_u8g2(u8g2_font_lubI12_tf, color),
            Self::lucida_serif_15_italic      => EgCharStyle::from_u8g2(u8g2_font_lubI14_tf, color),
            Self::lucida_serif_18_italic      => EgCharStyle::from_u8g2(u8g2_font_lubI18_tf, color),
            Self::lucida_serif_25_italic      => EgCharStyle::from_u8g2(u8g2_font_lubI24_tf, color),

            Self::lucida_sans_9               => EgCharStyle::from_u8g2(u8g2_font_luRS08_tf, color),
            Self::lucida_sans_10              => EgCharStyle::from_u8g2(u8g2_font_luRS10_tf, color),
            Self::lucida_sans_13              => EgCharStyle::from_u8g2(u8g2_font_luRS12_tf, color),
            Self::lucida_sans_15              => EgCharStyle::from_u8g2(u8g2_font_luRS14_tf, color),
            Self::lucida_sans_18              => EgCharStyle::from_u8g2(u8g2_font_luRS18_tf, color),
            Self::lucida_sans_25              => EgCharStyle::from_u8g2(u8g2_font_luRS24_tf, color),
            Self::lucida_sans_9_bold          => EgCharStyle::from_u8g2(u8g2_font_luBS08_tf, color),
            Self::lucida_sans_10_bold         => EgCharStyle::from_u8g2(u8g2_font_luBS10_tf, color),
            Self::lucida_sans_13_bold         => EgCharStyle::from_u8g2(u8g2_font_luBS12_tf, color),
            Self::lucida_sans_15_bold         => EgCharStyle::from_u8g2(u8g2_font_luBS14_tf, color),
            Self::lucida_sans_18_bold         => EgCharStyle::from_u8g2(u8g2_font_luBS18_tf, color),
            Self::lucida_sans_25_bold         => EgCharStyle::from_u8g2(u8g2_font_luBS24_tf, color),
            Self::lucida_sans_9_italic        => EgCharStyle::from_u8g2(u8g2_font_luIS08_tf, color),
            Self::lucida_sans_10_italic       => EgCharStyle::from_u8g2(u8g2_font_luIS10_tf, color),
            Self::lucida_sans_13_italic       => EgCharStyle::from_u8g2(u8g2_font_luIS12_tf, color),
            Self::lucida_sans_15_italic       => EgCharStyle::from_u8g2(u8g2_font_luIS14_tf, color),
            Self::lucida_sans_18_italic       => EgCharStyle::from_u8g2(u8g2_font_luIS18_tf, color),
            Self::lucida_sans_25_italic       => EgCharStyle::from_u8g2(u8g2_font_luIS24_tf, color),

            // u8g2 - spleen
            Self::spleen_mono_6               => EgCharStyle::from_u8g2(u8g2_font_spleen5x8_mf, color),
            Self::spleen_mono_8               => EgCharStyle::from_u8g2(u8g2_font_spleen6x12_mf, color),
            Self::spleen_mono_15              => EgCharStyle::from_u8g2(u8g2_font_spleen12x24_mf, color),
            Self::spleen_mono_20              => EgCharStyle::from_u8g2(u8g2_font_spleen16x32_mf, color),

            // u8g2 - fontstruct
            Self::trixel_sq_5                 => EgCharStyle::from_u8g2(u8g2_font_trixel_square_tr, color),
            Self::maniac_23                   => EgCharStyle::from_u8g2(u8g2_font_maniac_tf, color),
            Self::bubble_18                   => EgCharStyle::from_u8g2(u8g2_font_bubble_tr, color),

            // u8g2 - pentacom
            Self::bittypewriter_7             => EgCharStyle::from_u8g2(u8g2_font_BitTypeWriter_tr, color),
            Self::helvetipixeloutline_12      => EgCharStyle::from_u8g2(u8g2_font_HelvetiPixelOutline_tr, color),
            Self::unnameddosfont_12           => EgCharStyle::from_u8g2(u8g2_font_UnnamedDOSFontIV_tr, color),

            // u8g2 - profont
            Self::profont_6                   => EgCharStyle::from_u8g2(u8g2_font_profont10_tf, color),
            Self::profont_8                   => EgCharStyle::from_u8g2(u8g2_font_profont12_tf, color),
            Self::profont_9                   => EgCharStyle::from_u8g2(u8g2_font_profont15_tf, color),
            Self::profont_14                  => EgCharStyle::from_u8g2(u8g2_font_profont22_tf, color),
            Self::profont_19                  => EgCharStyle::from_u8g2(u8g2_font_profont29_tf, color),

            // u8g2- gilesbooth
            Self::sirclive_7                  => EgCharStyle::from_u8g2(u8g2_font_sirclive_tr, color),
        };

        font
    }
}
