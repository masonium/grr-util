//! GrrVertex
use grr::VertexFormat;
use nalgebra::{
    Matrix2, Matrix3, Matrix4, Point1, Point2, Point3, Point4, Quaternion, Vector1, Vector2,
    Vector3, Vector4,
};

/// `GrrVertex` contains utility methods that make it easier to create
/// vertex arrays and a vertex buffer of this vertex.
///
/// `GrrVertex` can be automatically derived if every vertex
pub trait GrrVertex: Sized {
    /// Create a sequence of `grr::VertexAttributeDesc` objects that
    /// map to the fields in `GrrVertex`.
    fn attribs(binding: u32, location_start: u32) -> Vec<grr::VertexAttributeDesc>;

    fn view(buffer: grr::Buffer, input_rate: grr::InputRate) -> grr::VertexBufferView {
        grr::VertexBufferView {
            buffer,
            input_rate,
            offset: 0,
            stride: std::mem::size_of::<Self>() as _,
        }
    }
}

pub trait GrrVertexField: Sized {
    const COMPONENT_SIZE: u32 = std::mem::size_of::<Self>() as _;

    fn size() -> u32 {
        std::mem::size_of::<Self>() as _
    }
    fn format() -> (VertexFormat, usize);
}

/// Implement `GrrVertexField` for a simple field, mapping to only one
/// location.
macro_rules! impl_field {
    ($ty: ty, $format: ident) => {
        impl GrrVertexField for $ty {
            fn format() -> (VertexFormat, usize) {
                (VertexFormat::$format, 1)
            }
        }
    };
}

/// Implement `GrrVertexField` for a matrix field, which spans
/// multiple locations.
macro_rules! impl_matrix_field {
    ($ty: ty, $format: ident, $comp_size: literal, $len: literal) => {
        impl GrrVertexField for $ty {
            const COMPONENT_SIZE: u32 = $comp_size;
            fn format() -> (VertexFormat, usize) {
                (VertexFormat::$format, $len)
            }
        }
    };
}

impl_field!(f32, X32Float);
impl_field!(Vector1<f32>, Xy32Float);
impl_field!(Vector2<f32>, Xy32Float);
impl_field!(Vector3<f32>, Xyz32Float);
impl_field!(Vector4<f32>, Xyzw32Float);
impl_field!(Point1<f32>, Xy32Float);
impl_field!(Point2<f32>, Xy32Float);
impl_field!(Point3<f32>, Xyz32Float);
impl_field!(Point4<f32>, Xyzw32Float);
impl_field!([f32; 1], X32Float);
impl_field!([f32; 2], Xy32Float);
impl_field!([f32; 3], Xyz32Float);
impl_field!([f32; 4], Xyzw32Float);

impl_field!(Quaternion<f32>, Xyzw32Float);

impl_field!(f64, X64Float);
impl_field!(Vector1<f64>, Xy64Float);
impl_field!(Vector2<f64>, Xy64Float);
impl_field!(Vector3<f64>, Xyz64Float);
impl_field!(Vector4<f64>, Xyzw64Float);
impl_field!(Point1<f64>, Xy64Float);
impl_field!(Point2<f64>, Xy64Float);
impl_field!(Point3<f64>, Xyz64Float);
impl_field!(Point4<f64>, Xyzw64Float);
impl_field!([f64; 1], X64Float);
impl_field!([f64; 2], Xy64Float);
impl_field!([f64; 3], Xyz64Float);
impl_field!([f64; 4], Xyzw64Float);

impl_matrix_field!(Matrix2<f32>, Xy32Float, 8, 2);
impl_matrix_field!(Matrix3<f32>, Xyz32Float, 12, 3);
impl_matrix_field!(Matrix4<f32>, Xyzw32Float, 16, 4);
impl_matrix_field!([[f32; 2]; 2], Xy32Float, 8, 2);
impl_matrix_field!([[f32; 3]; 3], Xyz32Float, 12, 3);
impl_matrix_field!([[f32; 4]; 4], Xyzw32Float, 16, 4);

impl<S: palette::rgb::RgbStandard> GrrVertexField for palette::rgb::Rgb<S, u8> {
    fn format() -> (VertexFormat, usize) {
        (VertexFormat::Xyz8Unorm, 1)
    }
}
impl<S: palette::rgb::RgbStandard> GrrVertexField for palette::rgb::Rgba<S, u8> {
    fn format() -> (VertexFormat, usize) {
        (VertexFormat::Xyzw8Unorm, 1)
    }
}

impl<S: palette::rgb::RgbStandard> GrrVertexField for palette::rgb::Rgb<S, f32> {
    fn format() -> (VertexFormat, usize) {
        (VertexFormat::Xyz32Float, 1)
    }
}
impl<S: palette::rgb::RgbStandard> GrrVertexField for palette::rgb::Rgba<S, f32> {
    fn format() -> (VertexFormat, usize) {
        (VertexFormat::Xyzw32Float, 1)
    }
}
