//! A module encompassing the primitive 2D shape widgets.
use lyon::lyon_tessellation::path::path::Builder;
use lyon::math::Rect;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, Side, StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers};
use lyon::tessellation::path::Path;

use crate::{Point, Scalar};
use crate::draw::shape::triangle::Triangle;
use crate::widget::{CommonWidget, GlobalState};
use crate::widget::types::shape_style::ShapeStyle;
use crate::widget::types::stroke_style::StrokeStyle;
use crate::widget::types::triangle_store::TriangleStore;

pub mod ellipse;
pub mod polygon;
pub mod rectangle;
pub mod rounded_rectangle;
pub mod capsule;

pub trait Shape<GS>: CommonWidget<GS> where GS: GlobalState {
    fn get_triangle_store_mut(&mut self) -> &mut TriangleStore;
    fn get_stroke_style(&self) -> StrokeStyle;
    fn get_shape_style(&self) -> ShapeStyle;
}

pub fn tessellate<GS: GlobalState>(shape: &mut dyn Shape<GS>, rectangle: &Rect, path: &dyn Fn(&mut Builder, &Rect)) {
    match shape.get_shape_style() {
        ShapeStyle::Default | ShapeStyle::Fill => {
            fill(path, shape, rectangle);
        }
        ShapeStyle::Stroke => {
            stroke(path, shape, rectangle);
        }
        ShapeStyle::FillAndStroke => {
            fill(path, shape, rectangle);
            stroke(path, shape, rectangle);
        }
    }
}

pub fn fill<GS: GlobalState>(path: &dyn Fn(&mut Builder, &Rect), shape: &mut dyn Shape<GS>, rectangle: &Rect) {
    let position = shape.get_position();
    let dimension = shape.get_dimension();
    let triangle_store = shape.get_triangle_store_mut();

    if triangle_store.diff_fill(position, dimension) {
        let mut builder = Path::builder();

        // Let the caller decide the geometry
        path(&mut builder, rectangle);

        let path = builder.build();

        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();

        let mut tessellator = FillTessellator::new();

        let fill_options = FillOptions::default();



        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &fill_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    let point = vertex.position().to_array();
                    [point[0] as Scalar, point[1] as Scalar]
                }),
            ).unwrap();
        }



        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Point> = point_iter.collect();

        let triangles = Triangle::from_point_list(points);

        triangle_store.latest_fill_position = position;
        triangle_store.latest_fill_dimensions = dimension;
        triangle_store.set_fill_triangles(&triangles);
    }
}

pub fn stroke<GS: GlobalState>(path: &dyn Fn(&mut Builder, &Rect), shape: &mut dyn Shape<GS>, rectangle: &Rect) {
    let position = shape.get_position();
    let dimension = shape.get_dimension();
    let line_width = shape.get_stroke_style().get_line_width() as f32;
    let triangle_store = shape.get_triangle_store_mut();

    if triangle_store.diff_stroke(position, dimension) {
        let mut builder = Path::builder();

        // Let the caller decide the geometry
        path(&mut builder, &rectangle);

        let path = builder.build();

        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();

        let mut tessellator = StrokeTessellator::new();

        let mut stroke_options = StrokeOptions::default();
        stroke_options.line_width = line_width * 2.0;

        let filled_points: Vec<Point> = {
            let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();

            let mut tessellator = FillTessellator::new();

            let fill_options = FillOptions::default();

            {
                // Compute the tessellation.
                tessellator.tessellate_path(
                    &path,
                    &fill_options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        let point = vertex.position().to_array();
                        [point[0] as Scalar, point[1] as Scalar]
                    }),
                ).unwrap();
            }



            let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

            point_iter.collect()
        };

        // Todo: This is linear and should be optimized
        fn get_closest_point(point: Point, points: &Vec<Point>) -> Point {
            let mut closest = points[0];
            let mut dist = 1000000.0;
            for p in points {
                let cur_dist = ((point[0] - p[0]).powi(2) + (point[1] - p[1]).powi(2)).sqrt();
                if cur_dist < dist {
                    dist = cur_dist;
                    closest = *p;
                }
            }
            closest
        }


        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &stroke_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    let point = vertex.position().to_array();
                    if vertex.side() == Side::Left {
                        [point[0] as Scalar, point[1] as Scalar]
                    } else {

                        let p = [point[0] as Scalar, point[1] as Scalar];

                        get_closest_point(p, &filled_points)
                    }

                }),
            ).unwrap();
        }

        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Point> = point_iter.collect();

        let triangles = Triangle::from_point_list(points);

        triangle_store.latest_stroke_position = position;
        triangle_store.latest_stroke_dimensions = dimension;
        triangle_store.set_stroke_triangles(&triangles);
    }
}