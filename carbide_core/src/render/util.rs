use crate::OldRect;
use crate::render::primitive::Primitive;
use crate::render::primitive_kind::PrimitiveKind;

/// Simplify the constructor for a `Primitive`.
pub fn new_primitive(kind: PrimitiveKind, rect: OldRect) -> Primitive {
    Primitive {
        kind,
        rect,
    }
}
