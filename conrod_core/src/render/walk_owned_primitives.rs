use widget::triangles::{Triangle, ColoredPoint};
use ::{Point, text};
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use render::text::Text;
use render::primitive_walker::PrimitiveWalker;
use render::owned_primitive_kind::OwnedPrimitiveKind;
use render::owned_primitive::OwnedPrimitive;
use render::owned_text::OwnedText;

/// An iterator-like type for yielding `Primitive`s from an `OwnedPrimitives`.
pub struct WalkOwnedPrimitives<'a> {
    pub(crate) primitives: std::slice::Iter<'a, OwnedPrimitive>,
    pub(crate) triangles_single_color: &'a [Triangle<Point>],
    pub(crate) triangles_multi_color: &'a [Triangle<ColoredPoint>],
    pub(crate) line_infos: &'a [text::line::Info],
    pub(crate) texts_str: &'a str,
    pub(crate) positioned_glyphs: Vec<text::PositionedGlyph>,
}

impl<'a> WalkOwnedPrimitives<'a> {

    /// Yield the next `Primitive` in order or rendering depth, bottom to top.
    pub fn next(&mut self) -> Option<Primitive> {
        let WalkOwnedPrimitives {
            ref mut primitives,
            ref mut positioned_glyphs,
            triangles_single_color,
            triangles_multi_color,
            line_infos,
            texts_str,
        } = *self;

        primitives.next().map(move |&OwnedPrimitive { id, rect, scizzor, ref kind }| {
            let new = |kind| Primitive {
                id: id,
                rect: rect,
                scizzor: scizzor,
                kind: kind,
            };

            match *kind {

                OwnedPrimitiveKind::Rectangle { color } => {
                    let kind = PrimitiveKind::Rectangle { color: color };
                    new(kind)
                },

                OwnedPrimitiveKind::TrianglesSingleColor { color, ref triangle_range } => {
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color: color,
                        triangles: triangles_single_color[triangle_range.clone()].to_owned(),
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::TrianglesMultiColor { ref triangle_range } => {
                    let kind = PrimitiveKind::TrianglesMultiColor {
                        triangles: &triangles_multi_color[triangle_range.clone()],
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::Text { color, font_id, ref text } => {
                    let OwnedText {
                        ref str_byte_range,
                        ref line_infos_range,
                        ref font,
                        window_dim,
                        font_size,
                        rect,
                        justify,
                        y_align,
                        line_spacing,
                    } = *text;

                    let text_str = &texts_str[str_byte_range.clone()];
                    let line_infos = &line_infos[line_infos_range.clone()];

                    let text = Text {
                        positioned_glyphs: positioned_glyphs,
                        window_dim: window_dim,
                        text: text_str,
                        line_infos: line_infos,
                        font: font,
                        font_size: font_size,
                        rect: rect,
                        justify: justify,
                        y_align: y_align,
                        line_spacing: line_spacing,
                    };

                    let kind = PrimitiveKind::Text {
                        color: color,
                        font_id: font_id,
                        text: text,
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::Image { image_id, color, source_rect } => {
                    let kind = PrimitiveKind::Image {
                        image_id: image_id,
                        color: color,
                        source_rect: source_rect,
                    };
                    new(kind)
                },
            }
        })
    }

}


impl<'a> PrimitiveWalker for WalkOwnedPrimitives<'a> {
    fn next_primitive(&mut self) -> Option<Primitive> {
        self.next()
    }
}





