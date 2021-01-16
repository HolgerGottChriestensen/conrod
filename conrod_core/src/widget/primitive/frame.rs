use std::ops::Neg;

use uuid::Uuid;

use crate::{Point, Scalar};
use crate::{Rect, text};
use crate::event::event::NoEvents;
use crate::flags::Flags;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::Layout;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::state::environment::Environment;
use crate::state::state_sync::NoLocalStateSync;
use crate::widget::Rectangle;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::state::global_state::GlobalState;

pub static SCALE: f64 = -1.0;


#[derive(Debug, Clone)]
pub struct Frame<S> where S: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    position: Point,
    dimension: Dimensions
}

impl<S: GlobalState> Frame<S> {
    pub fn init(width: Scalar, height: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, height]
        })
    }

    pub fn init_width(width: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, -1.0]
        })
    }

    pub fn init_height(height: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [-1.0, height]
        })
    }
}

impl<S: GlobalState> Widget<S> for Frame<S> {}

impl<S: GlobalState> WidgetExt<S> for Frame<S> {}

impl<S: GlobalState> NoEvents for Frame<S> {}

impl<S: GlobalState> NoLocalStateSync for Frame<S> {}

impl<S: GlobalState> CommonWidget<S> for Frame<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::single(&mut self.child)
    }


    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        [self.dimension[0].abs(), self.dimension[1].abs()]
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<S: GlobalState> Layout<S> for Frame<S> {
    fn flexibility(&self) -> u32 {
        9
    }

    fn calculate_size(&mut self, dimension: Dimensions, env: &Environment<S>) -> Dimensions {
        let dimensions = self.dimension;
        let abs_dimensions = match (dimensions[0], dimensions[1]) {
            (x, y) if x < 0.0 && y < 0.0 => [dimension[0], dimension[1]],
            (x, _y) if x < 0.0 => [dimension[0], self.dimension[1]],
            (_x, y) if y < 0.0 => [self.dimension[0], dimension[1]],
            (x, y) => [x, y]
        };

        let child_dimensions = self.child.calculate_size(abs_dimensions, env);

        if dimensions[0] < 0.0 {
            self.dimension = [child_dimensions[0].abs().neg(), dimensions[1]]
        }

        if dimensions[1] < 0.0 {
            self.dimension = [self.dimension[0], child_dimensions[1].abs().neg()]
        }

        [self.dimension[0].abs(), self.dimension[1].abs()]
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = [self.dimension[0].abs(), self.dimension[1].abs()];


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S: GlobalState> Render<S> for Frame<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, [self.dimension[0].abs(), self.dimension[1].abs()]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(fonts);
        prims.extend(children);

        return prims;
    }
}