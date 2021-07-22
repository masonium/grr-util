use grr_util::vertex::GrrVertex;
use grr_util_derive::GrrVertex;
use nalgebra as na;

#[derive(GrrVertex)]
#[repr(C)]
pub struct V_M2 {
    a: na::Matrix2<f32>,
}

#[test]
pub fn test_attribs_m2() {
    let attrs = V_M2::attribs(0, 0);
    assert_eq!(
        attrs[0],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 0,
            offset: 0,
            format: grr::VertexFormat::Xy32Float
        }
    );
    assert_eq!(
        attrs[1],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 1,
            offset: 8,
            format: grr::VertexFormat::Xy32Float
        }
    );
    assert_eq!(attrs.len(), 2);
}

#[derive(GrrVertex)]
#[repr(C)]
pub struct V_M3 {
    a: na::Matrix3<f32>,
}

#[test]
pub fn test_attribs_m3() {
    let attrs = V_M3::attribs(0, 0);
    assert_eq!(
        attrs[0],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 0,
            offset: 0,
            format: grr::VertexFormat::Xyz32Float
        }
    );
    assert_eq!(
        attrs[1],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 1,
            offset: 12,
            format: grr::VertexFormat::Xyz32Float
        }
    );
    assert_eq!(
        attrs[2],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 2,
            offset: 24,
            format: grr::VertexFormat::Xyz32Float
        }
    );
    assert_eq!(attrs.len(), 3);
}

#[derive(GrrVertex)]
#[repr(C)]
pub struct V_M4 {
    a: na::Matrix4<f32>,
}

#[test]
pub fn test_attribs_m4() {
    let attrs = V_M4::attribs(0, 0);
    assert_eq!(
        attrs[0],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 0,
            offset: 0,
            format: grr::VertexFormat::Xyzw32Float
        }
    );
    assert_eq!(
        attrs[1],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 1,
            offset: 16,
            format: grr::VertexFormat::Xyzw32Float
        }
    );
    assert_eq!(
        attrs[2],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 2,
            offset: 32,
            format: grr::VertexFormat::Xyzw32Float
        }
    );
    assert_eq!(
        attrs[3],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 3,
            offset: 48,
            format: grr::VertexFormat::Xyzw32Float
        }
    );
    assert_eq!(attrs.len(), 4);
}
