use carbide_core::{Point, Scalar};
use carbide_core::text::PositionedGlyph;

#[derive(Debug, Clone, Copy)]
pub enum Cursor {
    Single(CursorIndex),
    Selection {start: CursorIndex, end: CursorIndex}
}

impl Cursor {
    pub fn get_width(&self, text: &str, positioned_glyphs: &Vec<PositionedGlyph>) -> Scalar {
        if let Cursor::Selection {start, end} = self {
            let start_point = start.get_position(text, positioned_glyphs);
            let end_point = end.get_position(text, positioned_glyphs);
            end_point[0] - start_point[0]
        } else {
            0.0
        }
    }

    pub fn get_char_index(relative_offset: f64, _text: &str, positioned_glyphs: &Vec<PositionedGlyph>) -> usize {
        let splits = vec![0.0].into_iter().chain(positioned_glyphs.iter().map(|val| {
            let middle = val.position().x + val.unpositioned().h_metrics().advance_width;
            middle
        }));

        let collected: Vec<f32> = splits.collect();

        let mut closest_iter = collected.iter().enumerate().skip(1).skip_while(|(i, s)|{
            let distance_to_before = (relative_offset as f32 - collected[i-1]).abs();

            let distance_to_after = (relative_offset as f32 - **s).abs();

            distance_to_after < distance_to_before

        });

        let closest = match closest_iter.next() {
            None => {positioned_glyphs.len()}
            Some((i, _)) => i-1
        };



        closest
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CursorIndex {
    /// The index of the line upon which the cursor is situated.
    pub line: usize,
    /// The index within all possible cursor positions for the line.
    ///
    /// For example, for the line `foo`, a `char` of `1` would indicate the cursor's position
    /// as `f|oo` where `|` is the cursor.
    pub char: usize,
}


impl CursorIndex {
    pub fn get_position(&self, text: &str, positioned_glyphs: &Vec<PositionedGlyph>) -> Point {
        if self.line == 0 {
            if self.char == 0 {
                return [0.0, 0.0]
            }
            if self.char <= positioned_glyphs.len() {
                let positioned = &positioned_glyphs[self.char-1];

                let point = positioned.position();

                let width = positioned.unpositioned().h_metrics().advance_width;

                [point.x as f64 + width as f64, point.y as f64]

            } else {
                panic!("The char index is outside of the letters({}): {} > {}", text, self.char, positioned_glyphs.len()-1)
            }
        } else {
            panic!("For now only operate on single line things")
        }
    }


}