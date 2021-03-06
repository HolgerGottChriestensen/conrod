//use crate::text::paragraph::Paragraph;
use crate::{Color, OldRect, Point, Scalar};
use crate::color::BLACK;
use crate::draw::{Dimension, Position, Rect};
use crate::text::{Font, FontId};
use crate::text::font_family::FontFamily;
use crate::text::font_style::FontStyle;
use crate::text::font_weight::FontWeight;
use crate::text::glyph::Glyph;
use crate::text::markup::{parse_polar_bear_markup, PolarItem};
use crate::text::text_decoration::TextDecoration;
use crate::text::text_style::TextStyle;
use crate::widget::{Environment, GlobalState, Widget};
use crate::widget::types::justify::Justify;
use crate::widget::types::text_wrap::Wrap;

#[derive(Debug, Clone)]
pub enum TextSpan<GS> where GS: GlobalState {
    Text {
        style: Option<TextStyle>,
        text: String,
        glyphs: Vec<Glyph>,
        widths: Vec<Scalar>,
        ascend: f64,
        descend: f64,
        line_gap: f64,
    },
    Widget(Box<dyn Widget<GS>>),
    NewLine,
}

impl<GS: GlobalState> TextSpan<GS> {
    pub fn new(string: &str, env: &mut Environment<GS>) -> Vec<TextSpan<GS>> {
        let default_font_family_name = env.get_first_font_family().name.clone();

        let scale_factor = env.get_scale_factor();

        let mut res = vec![];

        for (index, line) in string.split('\n').enumerate() {
            let font_style = if index % 3 == 0 {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            };

            let style = TextStyle {
                font_family: default_font_family_name.clone(),
                font_size: 14,
                font_style,
                font_weight: FontWeight::Normal,
                text_decoration: TextDecoration::None,
                color: None,
            };

            let font = style.get_font(env);

            let ascend = font.ascend(style.font_size, scale_factor);
            let descend = font.descend(style.font_size, scale_factor);
            let line_gap = font.line_gap(style.font_size, scale_factor);
            let (widths, glyphs) = font.get_glyphs(line, style.font_size, scale_factor, env);

            res.push(TextSpan::Text {
                style: Some(style.clone()),
                text: line.to_string(),
                glyphs,
                widths,
                ascend,
                descend,
                line_gap,
            });
            res.push(TextSpan::NewLine);
        }

        res.pop();
        res
    }

    // https://bear.app/faq/Markup%20:%20Markdown/Polar%20Bear%20markup%20language/
    pub fn new_polar_bear_markup(string: &str, env: &mut Environment<GS>) -> Vec<TextSpan<GS>> {
        let default_font_family_name = env.get_first_font_family().name.clone();
        let scale_factor = env.get_scale_factor();
        let polars = parse_polar_bear_markup(string).unwrap().1;

        let mut spans = vec![];

        for polar in polars {
            match polar {
                PolarItem::Header1(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 30,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Bold,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) = font.get_glyphs(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                    spans.push(TextSpan::NewLine)
                }
                PolarItem::Header2(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 20,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) = font.get_glyphs(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);

                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                    spans.push(TextSpan::NewLine)
                }
                PolarItem::Italic(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Italic,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) = font.get_glyphs(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);

                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Bold(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Bold,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) = font.get_glyphs(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Paragraph(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::None,
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) = font.get_glyphs(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Underline(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::Underline(vec![]),
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) = font.get_glyphs(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Strike(text) => {
                    let style = TextStyle {
                        font_family: default_font_family_name.clone(),
                        font_size: 14,
                        font_style: FontStyle::Normal,
                        font_weight: FontWeight::Normal,
                        text_decoration: TextDecoration::StrikeThrough(vec![]),
                        color: None,
                    };
                    let font = style.get_font(env);

                    let (widths, glyphs) = font.get_glyphs(&text, style.font_size, scale_factor, env);
                    let ascending_pixels = font.ascend(style.font_size, scale_factor);
                    let line_height = font.descend(style.font_size, scale_factor);
                    let line_gap = font.line_gap(style.font_size, scale_factor);

                    let span = TextSpan::Text {
                        style: Some(style.clone()),
                        text: text.to_string(),
                        glyphs,
                        widths,
                        ascend: ascending_pixels,
                        descend: line_height,
                        line_gap,
                    };

                    spans.push(span);
                }
                PolarItem::Newline => {
                    spans.push(TextSpan::NewLine)
                }
                _ => ()
            }
        }

        spans
    }
}