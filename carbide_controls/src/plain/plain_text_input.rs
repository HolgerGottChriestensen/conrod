use carbide_core::widget::*;
use carbide_core::color::{RED, GREEN};
use carbide_core::event_handler::{KeyboardEvent, MouseEvent};
use crate::plain::cursor::{Cursor, CursorIndex};
use carbide_core::state::environment::Environment;
use carbide_core::draw::shape::vertex::Vertex;
use carbide_core::widget::text::Wrap;
use crate::plain::text_input_key_commands::TextInputKeyCommand;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use carbide_core::text::PositionedGlyph;


#[derive(Clone, Widget)]
#[event(handle_keyboard_event, handle_mouse_event)]
pub struct PlainTextInput<GS> where GS: GlobalState {
    id: Id,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    cursor: Cursor,
    grapheme_split_cache: (String, Vec<f32>),
    #[state] text: State<String, GS>,
    #[state] cursor_x: State<f64, GS>,
    #[state] selection_x: State<f64, GS>,
    #[state] selection_width: State<f64, GS>,
    #[state] text_offset: State<f64, GS>,
}

impl<GS: GlobalState> PlainTextInput<GS> {
    pub fn new() -> Box<Self> {

        let text_state = State::new_local_with_key(&String::from("Hello World!"));

        let cursor_x = State::new_local_with_key(&0.0);
        let selection_x = State::new_local_with_key(&0.0);

        let selection_width = State::new_local_with_key(&4.0);

        let text_offset = State::new_local_with_key(&0.0);

        Box::new(PlainTextInput {
            id: Id::new_v4(),
            child: HStack::initialize( vec![
                ZStack::initialize(vec![
                    Rectangle::initialize(vec![])
                        .fill(GREEN)
                        .frame(selection_width.clone(), 40.0.into())
                        .offset(selection_x.clone(), 0.0.into()),
                    Text::initialize(text_state.clone())
                        .font_size(40.into()).wrap_mode(Wrap::None),
                    Rectangle::initialize(vec![])
                        .fill(RED)
                        .frame(4.0.into(), 40.0.into())
                        .offset(cursor_x.clone(), 0.0.into())
            ]).alignment(BasicLayouter::TopLeading)
                    .offset(text_offset.clone(), 0.0.into()),
                   Spacer::new(SpacerDirection::Horizontal)
            ]),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            text: text_state,
            grapheme_split_cache: ("".to_string(), vec![]),
            cursor: Cursor::Single(CursorIndex{ line: 0, char: 0 }),
            cursor_x,
            selection_width,
            selection_x,
            text_offset
        })
    }

    fn len_in_graphemes(text: &String) -> usize {
        text.graphemes(true).count()
    }

    fn byte_index_from_graphemes(index: usize, text: &str) -> usize {
        if text.len() == 0 { return 0 }
        let grapheme_byte_offset = match text.grapheme_indices(true).skip(index).next() {
            None => text.len(),
            Some((g, _)) => g
        };
        grapheme_byte_offset
    }

    fn insert_str(&mut self, index: usize, string: &str, global_state: &mut GS) {
        let offset = Self::byte_index_from_graphemes(index, self.text.get_value(global_state));
        self.text.get_value_mut(global_state).insert_str(offset, string);
    }

    fn remove(&mut self, index: usize, global_state: &mut GS) {
        let offset = Self::byte_index_from_graphemes(index, self.text.get_value(global_state));
        self.text.get_value_mut(global_state).remove(offset);
    }

    fn remove_range(&mut self, index: Range<usize>, global_state: &mut GS) {
        let text = self.text.get_value(global_state);

        let offset_start = Self::byte_index_from_graphemes(index.start, text);
        let offset_end = Self::byte_index_from_graphemes(index.end, text);
        self.text.get_value_mut(global_state).replace_range(offset_start..offset_end, "");
    }

    fn prev_word_range(text: String, start_index: usize) -> Range<usize> {
        let mut has_hit_space = false;

        let number_left = text.chars().rev().skip(Self::len_in_graphemes(&text) - start_index).skip_while(|cur| {
            if *cur == ' ' {
                has_hit_space = true;
                true
            } else {
                !has_hit_space
            }
        }).count();

        number_left..start_index
    }

    fn next_word_range(text: String, start_index: usize) -> Range<usize> {
        let mut has_hit_space = false;

        let number_left = text.chars().skip(start_index).skip_while(|cur| {
            if *cur == ' ' {
                has_hit_space = true;
                true
            } else {
                !has_hit_space
            }
        }).count();

        let new_index = Self::len_in_graphemes(&text) - number_left;

        start_index..new_index
    }

    fn word_index_range(text: String, start_index: usize) -> Range<usize> {
        let mut max_iter = text.chars().enumerate().skip(start_index).skip_while(|(_, cur)|{
           *cur != ' '
        });

        let mut min_iter = text.chars().rev().enumerate().skip(Self::len_in_graphemes(&text) - start_index).skip_while(|(_, cur)|{
            *cur != ' '
        });

        let max = match max_iter.next() {
            None => {Self::len_in_graphemes(&text)}
            Some((u, _)) => u
        };

        let min = match min_iter.next() {
            None => 0,
            Some((u, _)) => Self::len_in_graphemes(&text) - u
        };

        min..max
    }

    fn get_positioned_glyphs(&mut self, text: &String, env: &Environment<GS>) -> Vec<PositionedGlyph> {
        let mut text_scaler: Box<carbide_core::widget::Text<GS>> = Text::initialize(text.clone().into())
            .font_size(40.into()).wrap_mode(Wrap::None);

        text_scaler.set_position([0.0, 0.0]);
        text_scaler.set_dimension(self.dimension.add([100.0,100.0]));

        let positioned_glyphs = text_scaler.get_positioned_glyphs(env.get_fonts_map(), 1.0); //Todo: save dpi in env stack
        positioned_glyphs
    }

    fn check_for_cache_updates(&mut self, text: &String, env: &Environment<GS>) {
        let (cache_string, _) = &self.grapheme_split_cache;

        if text != cache_string {
            // Position the cursor
            let positioned_glyphs = self.get_positioned_glyphs(text, env);

            let new_splits = Cursor::get_char_index_split_points(&positioned_glyphs);
            let new_cache = (text.clone(), new_splits);

            self.grapheme_split_cache = new_cache;
        }
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment<GS>, global_state: &mut GS) {
        let text_offset = *self.text_offset.get_value(global_state);

        match event {
            MouseEvent::Press(_, position, _) => {
                let text = self.text.get_value(global_state).clone();

                self.check_for_cache_updates(&text, env);
                let (_, cache_split) = &self.grapheme_split_cache;


                let relative_offset = position[0] - self.position[0] - text_offset;
                let char_index = Cursor::get_char_index(relative_offset, &text, &cache_split);

                self.cursor = Cursor::Single(CursorIndex{ line: 0, char: char_index });
            }
            MouseEvent::NClick(_, position, _, n) => {
                if n % 2 == 1 {
                    self.cursor = Cursor::Selection {start: CursorIndex{line: 0, char: 0}, end: CursorIndex {line: 0, char: Self::len_in_graphemes(self.text.get_value(global_state))}};
                } else {
                    let text = self.text.get_value(global_state).clone();

                    self.check_for_cache_updates(&text, env);

                    let (_, cache_split) = &self.grapheme_split_cache;

                    let relative_offset = position[0] - self.position[0] - text_offset;

                    let char_index = Cursor::get_char_index(relative_offset, &text, &cache_split);

                    let range = Self::word_index_range(text.clone(), char_index);

                    self.cursor = Cursor::Selection { start: CursorIndex { line: 0, char: range.start }, end: CursorIndex { line: 0, char: range.end } }
                }

            }
            MouseEvent::Drag { to, .. } => {
                let text = self.text.get_value(global_state).clone();

                let mouse_scroll_threshold = 30.0;

                if to[0] < self.get_x() + mouse_scroll_threshold {
                    let offset = self.text_offset.get_value(global_state) + 10.0;
                    *self.text_offset.get_value_mut(global_state) = offset.min(0.0);
                } else if to[0] > self.get_x() + self.get_width() - mouse_scroll_threshold {
                    let offset = self.text_offset.get_value(global_state) - 10.0;
                    let text = self.text.get_value(global_state).clone();
                    let positioned_glyphs = self.get_positioned_glyphs(&text, env);

                    let start = CursorIndex {line: 0, char: 0};
                    let end = CursorIndex {line: 0, char: Self::len_in_graphemes(self.text.get_value(global_state))};

                    let max_offset = Cursor::Selection{start, end}.get_width(&text, &positioned_glyphs);

                    *self.text_offset.get_value_mut(global_state) = offset.max(-(max_offset - self.get_width()));
                }


                self.check_for_cache_updates(&text, env);

                let (_, cache_split) = &self.grapheme_split_cache;

                let current_relative_offset = to[0] - self.position[0] - text_offset;

                let current_char_index = Cursor::get_char_index(current_relative_offset, &text, &cache_split);

                match self.cursor {
                    Cursor::Single(index) => {
                        self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: current_char_index } }
                    }
                    Cursor::Selection { start, .. } => {
                        self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: current_char_index } }
                    }
                }
            }
            _ => ()
        }

        self.reposition_cursor(env, global_state);
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        match event {
            KeyboardEvent::Press(key, modifier) => {
                let (current_movable_cursor_index, _is_selection) = match self.cursor {
                    Cursor::Single(cursor_index) => {
                        (cursor_index, false)
                    }
                    Cursor::Selection { end, .. } => {
                        (end, true)
                    }
                };

                match (key, modifier).into() {
                    TextInputKeyCommand::MoveLeft => {
                        let current_char = current_movable_cursor_index.char;
                        let moved_char = if current_char == 0 {0} else {current_char - 1};
                        let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(self.text.get_value(global_state)));

                        self.cursor = Cursor::Single(CursorIndex{ line: 0, char: clamped });
                    }
                    TextInputKeyCommand::MoveRight => {
                        let current_char = current_movable_cursor_index.char;
                        let moved_char = current_char + 1;
                        let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(self.text.get_value(global_state)));

                        self.cursor = Cursor::Single(CursorIndex{ line: 0, char: clamped });
                    }
                    TextInputKeyCommand::RemoveLeft => {

                        match self.cursor {
                            Cursor::Single(index) => {
                                if index.char > 0 {
                                    self.remove(index.char - 1, global_state);
                                    self.cursor = Cursor::Single(CursorIndex{ line: 0, char: index.char -1 });
                                }
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                self.remove_range(min..max, global_state);

                                self.cursor = Cursor::Single(CursorIndex{ line: 0, char: min });
                            }
                        }
                    }
                    TextInputKeyCommand::RemoveRight => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                let mut_text = self.text.get_value_mut(global_state);
                                if index.char < Self::len_in_graphemes(mut_text) {
                                    self.remove(index.char, global_state);
                                    self.cursor = Cursor::Single(CursorIndex{ line: 0, char: index.char });
                                }
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);
                                self.remove_range(min..max, global_state);

                                self.cursor = Cursor::Single(CursorIndex{ line: 0, char: min });
                            }
                        }
                    }
                    TextInputKeyCommand::Undefined => {}
                    TextInputKeyCommand::Copy => {
                        let mut ctx = ClipboardContext::new().unwrap();
                        let text = self.text.get_value(global_state).clone();


                        match self.cursor {
                            Cursor::Single(_) => {
                                ctx.set_contents(text).unwrap();
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                let s = text[min..max].to_string();
                                ctx.set_contents(s).unwrap();
                            }
                        }
                    }
                    TextInputKeyCommand::Paste => {
                        let mut ctx = ClipboardContext::new().unwrap();

                        let mut content = ctx.get_contents().unwrap();

                        // Remove newlines from the pasted text
                        content.retain(|c| {c != '\n'});

                        match self.cursor {
                            Cursor::Single(index) => {
                                self.insert_str(index.char, &content, global_state);
                                self.cursor = Cursor::Single(CursorIndex{ line: 0, char: index.char + Self::len_in_graphemes(&content) });
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);
                                self.remove_range(min..max, global_state);

                                self.insert_str(min, &content, global_state);
                                self.cursor = Cursor::Single(CursorIndex{ line: 0, char: min + Self::len_in_graphemes(&content) });

                            }
                        }
                    }
                    TextInputKeyCommand::Clip => {
                        let mut ctx = ClipboardContext::new().unwrap();
                        let text = self.text.get_value(global_state).clone();
                        match self.cursor {
                            Cursor::Single(_) => {
                                ctx.set_contents(text).unwrap();
                                self.text.get_value_mut(global_state).clear();

                                self.cursor = Cursor::Single(CursorIndex{ line: 0, char: 0 })
                            }
                            Cursor::Selection { start, end } => {
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);
                                let s = text[min..max].to_string();
                                ctx.set_contents(s).unwrap();
                                self.remove_range(min..max, global_state);

                                self.cursor = Cursor::Single(CursorIndex{ line: 0, char: min })
                            }
                        }
                    }
                    TextInputKeyCommand::SelectLeft => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                let moved_char = if index.char == 0 {0} else {index.char - 1};
                                let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(self.text.get_value(global_state)));

                                self.cursor = Cursor::Selection {start: index, end: CursorIndex {line: 0, char: clamped}}

                            }
                            Cursor::Selection { start, end } => {
                                let moved_char = if end.char == 0 {0} else {end.char - 1};
                                let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(self.text.get_value(global_state)));

                                if start.char == clamped {
                                    self.cursor = Cursor::Single(start)
                                } else {
                                    self.cursor = Cursor::Selection {start, end: CursorIndex {line: 0, char: clamped}}
                                }


                            }
                        }
                    }
                    TextInputKeyCommand::SelectRight => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                let moved_char = index.char + 1;
                                let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(self.text.get_value(global_state)));

                                self.cursor = Cursor::Selection {start: index, end: CursorIndex {line: 0, char: clamped}}

                            }
                            Cursor::Selection { start, end } => {
                                let moved_char = end.char + 1;
                                let clamped = carbide_core::utils::clamp(moved_char, 0, Self::len_in_graphemes(self.text.get_value(global_state)));

                                if start.char == clamped {
                                    self.cursor = Cursor::Single(start)
                                } else {
                                    self.cursor = Cursor::Selection {start, end: CursorIndex {line: 0, char: clamped}}
                                }
                            }
                        }

                    }
                    TextInputKeyCommand::SelectAll => {
                        self.cursor = Cursor::Selection {start: CursorIndex{line: 0, char: 0}, end: CursorIndex {line: 0, char: Self::len_in_graphemes(self.text.get_value(global_state))}}
                    }
                    TextInputKeyCommand::JumpWordLeft => {
                        let text = self.text.get_value(global_state).clone();
                        let start_index = current_movable_cursor_index.char;

                        let range = Self::prev_word_range(text, start_index);

                        self.cursor = Cursor::Single(CursorIndex {line: 0, char: range.start})

                    }
                    TextInputKeyCommand::JumpWordRight => {
                        let text = self.text.get_value(global_state).clone();
                        let start_index = current_movable_cursor_index.char;

                        let range = Self::next_word_range(text, start_index);

                        self.cursor = Cursor::Single(CursorIndex {line: 0, char: range.end})
                    }
                    TextInputKeyCommand::JumpSelectWordLeft => {
                        let text = self.text.get_value(global_state).clone();
                        let start_index = current_movable_cursor_index.char;

                        let range = Self::prev_word_range(text, start_index);

                        match self.cursor {
                            Cursor::Single(_) => {
                                self.cursor = Cursor::Selection{ start: CursorIndex { line: 0, char: start_index }, end: CursorIndex { line: 0, char: range.start } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection{ start, end: CursorIndex { line: 0, char: range.start } }
                            }
                        }
                    }
                    TextInputKeyCommand::JumpSelectWordRight => {
                        let text = self.text.get_value(global_state).clone();
                        let start_index = current_movable_cursor_index.char;

                        let range = Self::next_word_range(text, start_index);

                        match self.cursor {
                            Cursor::Single(_) => {
                                self.cursor = Cursor::Selection{ start: CursorIndex { line: 0, char: start_index }, end: CursorIndex { line: 0, char: range.end } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection{ start, end: CursorIndex { line: 0, char: range.end } }
                            }
                        }


                    }
                    TextInputKeyCommand::RemoveAll => {
                        self.text.get_value_mut(global_state).clear();
                        self.cursor = Cursor::Single (CursorIndex{line: 0, char: 0})
                    }
                    TextInputKeyCommand::RemoveWordLeft => {
                        if let Cursor::Single(index) = self.cursor {
                            let text = self.text.get_value(global_state).clone();
                            let start_index = index.char;

                            let range = Self::prev_word_range(text, start_index);
                            let start = range.start;

                            self.remove_range(range, global_state);

                            self.cursor = Cursor::Single (CursorIndex{line: 0, char: start})

                        }
                    }
                    TextInputKeyCommand::RemoveWordRight => {
                        if let Cursor::Single(index) = self.cursor {
                            let text = self.text.get_value(global_state).clone();
                            let start_index = index.char;

                            let range = Self::next_word_range(text, start_index);
                            let start = range.start;

                            self.remove_range(range, global_state);

                            self.cursor = Cursor::Single (CursorIndex{line: 0, char: start})

                        }
                    }
                    TextInputKeyCommand::DuplicateLeft => {
                        match self.cursor {
                            Cursor::Single(_) => {
                                let text = self.text.get_value(global_state).clone();
                                self.text.get_value_mut(global_state).push_str(&text);

                            }
                            Cursor::Selection { start, end } => {
                                let text = self.text.get_value(global_state).clone();
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                self.insert_str(max, &text[min..max], global_state);
                            }
                        }
                    }
                    TextInputKeyCommand::DuplicateRight => {
                        match self.cursor {
                            Cursor::Single(_) => {
                                let text = self.text.get_value(global_state).clone();
                                self.text.get_value_mut(global_state).push_str(&text);

                                self.cursor = Cursor::Single (CursorIndex{line: 0, char: Self::len_in_graphemes(&text) * 2})
                            }
                            Cursor::Selection { start, end } => {
                                let text = self.text.get_value(global_state).clone();
                                let min = start.char.min(end.char);
                                let max = start.char.max(end.char);

                                self.insert_str(max, &text[min..max], global_state);

                                self.cursor = Cursor::Selection { start: CursorIndex {line: 0, char: end.char}, end: CursorIndex {line: 0, char: end.char + (min..max).count()} }
                            }
                        }
                    }
                    TextInputKeyCommand::JumpToLeft => {
                        self.cursor = Cursor::Single(CursorIndex{line: 0, char: 0})
                    }
                    TextInputKeyCommand::JumpToRight => {
                        self.cursor = Cursor::Single(CursorIndex{line: 0, char: Self::len_in_graphemes(self.text.get_value(global_state))})
                    }
                    TextInputKeyCommand::JumpSelectToLeft => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: 0 } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: 0 } }
                            }
                        }
                    }
                    TextInputKeyCommand::JumpSelectToRight => {
                        match self.cursor {
                            Cursor::Single(index) => {
                                self.cursor = Cursor::Selection { start: index, end: CursorIndex { line: 0, char: Self::len_in_graphemes(self.text.get_value(global_state)) } }
                            }
                            Cursor::Selection { start, .. } => {
                                self.cursor = Cursor::Selection { start, end: CursorIndex { line: 0, char: Self::len_in_graphemes(self.text.get_value(global_state)) } }
                            }
                        }
                    }
                }
            }
            KeyboardEvent::Text(string, _modifiers) => {
                if Self::len_in_graphemes(&string) == 0 || string.chars().next().unwrap().is_control() { return }

                match self.cursor {
                    Cursor::Single(index) => {
                        self.insert_str(index.char, string, global_state);

                        self.cursor = Cursor::Single(CursorIndex{ line: 0, char: index.char + Self::len_in_graphemes(&string) });
                    }
                    Cursor::Selection { start, end } => {
                        let min = start.char.min(end.char);
                        let max = start.char.max(end.char);
                        self.remove_range(min..max, global_state);
                        self.insert_str(min, string, global_state);
                        self.cursor = Cursor::Single(CursorIndex{ line: 0, char: min + Self::len_in_graphemes(&string) });
                    }
                }
            }
            _ => ()
        }

        self.reposition_cursor(env, global_state);
        self.recalculate_offset_to_make_cursor_visible(env, global_state);
    }

    fn reposition_cursor(&mut self, env: &mut Environment<GS>, global_state: &mut GS) {
        let text = self.text.get_value(global_state).clone();

        // Position the cursor
        let mut text_scaler: Box<carbide_core::widget::Text<GS>> = Text::initialize(text.clone().into())
            .font_size(40.into()).wrap_mode(Wrap::None);

        text_scaler.set_position([0.0, 0.0]);
        text_scaler.set_dimension(self.dimension.add([100.0, 100.0]));

        let positioned_glyphs = text_scaler.get_positioned_glyphs(env.get_fonts_map(), 1.0); //Todo: save dpi in env stack

        let index = match self.cursor {
            Cursor::Single(index) => index,
            Cursor::Selection { end, .. } => end
        };

        let point = index.get_position(&text, &positioned_glyphs);

        *self.cursor_x.get_value_mut(global_state) = point[0];
        *self.selection_x.get_value_mut(global_state) = point[0];

        let selection_width = self.cursor.get_width(&text, &positioned_glyphs);

        if selection_width < 0.0 {
            *self.selection_width.get_value_mut(global_state) = selection_width.abs();
        } else {
            *self.selection_x.get_value_mut(global_state) -= selection_width;
            *self.selection_width.get_value_mut(global_state) = selection_width;
        }
    }

    fn recalculate_offset_to_make_cursor_visible(&mut self, env: &mut Environment<GS>, global_state: &mut GS) {
        let cursor_x = *self.cursor_x.get_value(global_state);
        let cursor_width = 4.0;
        let current_text_offset = *self.text_offset.get_value(global_state);

        if cursor_x + cursor_width > self.get_width() && -current_text_offset < cursor_x + cursor_width - self.get_width() {
            let new_text_offset = -(cursor_x + cursor_width - self.get_width());

            *self.text_offset.get_value_mut(global_state) = new_text_offset;
        } else if cursor_x + current_text_offset < 0.0 {
            let new_text_offset = -(cursor_x);

            *self.text_offset.get_value_mut(global_state) = new_text_offset;
        }


    }
}


impl<GS: GlobalState> CommonWidget<GS> for PlainTextInput<GS> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<GS: GlobalState> ChildRender for PlainTextInput<GS> {}

impl<GS: GlobalState> Layout<GS> for PlainTextInput<GS> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], env: &Environment<GS>) -> [f64; 2] {
        let mut dimensions = [0.0, 0.0];
        if let Some(child) = self.get_children_mut().next() {
            dimensions = child.calculate_size(requested_size, env);
        }

        self.set_dimension([requested_size[0], dimensions[1]]);

        self.get_dimension()
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.get_position();
        let dimension = self.get_dimension();

        if let Some(child) = self.get_children_mut().next() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl<GS: GlobalState> WidgetExt<GS> for PlainTextInput<GS> {}