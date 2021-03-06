//! A simple example that demonstrates using carbide within a basic `winit` window loop, using
//! `glium` to render the `carbide_core::render::Primitives` to screen.

#[macro_use]
extern crate carbide_core;
extern crate carbide_glium;
#[macro_use]
extern crate carbide_winit;
extern crate glium;
extern crate uuid;

use std::ops::{Deref, DerefMut};

use glium::Surface;
use uuid::Uuid;

use carbide_core::{Colorable, Point, Positionable, widget};
use carbide_core::color::{GREEN, LIGHT_BLUE, RED};
use carbide_core::event::event::Event;
use carbide_core::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use carbide_core::flags::Flags;
use carbide_core::layout::basic_layouter::BasicLayouter;
use carbide_core::layout::Layout;
use carbide_core::layout::layouter::Layouter;
use carbide_core::position::Dimensions;
use carbide_core::prelude::Environment;
use carbide_core::state::environment::Environment;
use carbide_core::state::state::{CommonState, LocalStateList};
use carbide_core::state::state_sync::NoLocalStateSync;
use carbide_core::text_old::font::Map;
use carbide_core::widget::{Ellipse, Frame, Image, Line, Rectangle, SCALE, Text, ZStack};
use carbide_core::widget::common_widget::CommonWidget;
use carbide_core::widget::complex::sync_test::SyncTest;
use carbide_core::widget::ellipse::Full;
use carbide_core::widget::primitive::edge_insets::EdgeInsets;
use carbide_core::widget::primitive::h_stack::HStack;
use carbide_core::widget::primitive::spacer::{Spacer, SpacerDirection};
use carbide_core::widget::primitive::v_stack::VStack;
use carbide_core::widget::primitive::Widget;
use carbide_core::widget::primitive::widget::WidgetExt;
use carbide_core::widget::render::ChildRender;
use carbide_core::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use carbide_glium::Window;

mod support;

const WIDTH: u32 = 750 / 2;
const HEIGHT: u32 = 1334 / 2;

fn main() {
    let mut window = Window::new("Hello world 2".to_string(), WIDTH, HEIGHT, GState {
        s: String::from("Hejsa")
    });

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_hover.png").unwrap();

    // Rectangle::new(params!(alignment: Alignment::Leading))

    let sync_state = CommonState::new_local("K", &"Hello".to_string());

    window.set_widgets(
        VStack::initialize(vec![
            Text::new("Hello".into(), vec![]),
            Text::new("world! \nHvad sker der i denne verden og vil den layoute rigtigt når der er en lang tekst".into(), vec![]),
            Image::new(rust_image, [100.0, 100.0], vec![]),
            Rectangle::initialize(vec![
                SyncTest::new(sync_state)
            ]).fill(GREEN),
            HStack::initialize(vec![
                Image::new(rust_image, [100.0, 100.0], vec![]),
                //ZStack::initialize(vec![
                Rectangle::initialize(vec![])
                    .fill(LIGHT_BLUE)
                    .frame(SCALE, 120.0),
                //Image::new(rust_image, [100.0,100.0], vec![])
                // ])
            ]),
            CustomWidget::new(),
        ])
    );
    window.draw()
}

#[derive(Clone, Debug)]
struct GState {
    pub s: String,
}

#[derive(Clone)]
pub struct CustomWidget {
    id: Uuid,
    child: Box<dyn Widget<GState>>,
    position: Point,
    dimension: Dimensions,
}

impl CustomWidget {
    pub fn new() -> Box<CustomWidget> {
        Box::new(CustomWidget {
            id: Uuid::new_v4(),
            child: HStack::initialize(vec![
                Spacer::new(SpacerDirection::Horizontal),
                Ellipse::new(vec![])
                    .fill(RED)
                    .padding(EdgeInsets::all(10.0))
                    .frame(150.0, 150.0),
                Spacer::new(SpacerDirection::Horizontal),
                Spacer::new(SpacerDirection::Horizontal)
            ]),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
        })
    }
}

impl CommonWidget<GState> for CustomWidget {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GState> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GState> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GState> {
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

impl Event<GState> for CustomWidget {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut GState) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut GState) {
        match event {
            KeyboardEvent::Text(st, _) => {
                global_state.s.push_str(st.as_str());
                println!("Global state says: {}", global_state.s);
            }
            _ => {}
        }
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }
}

impl NoLocalStateSync for CustomWidget {}

impl ChildRender for CustomWidget {}

impl Layout<GState> for CustomWidget {
    fn flexibility(&self) -> u32 {
        2
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, env);
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;
        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl WidgetExt<GState> for CustomWidget {}