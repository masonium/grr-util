use grr_util::vertex::GrrVertex;
use grr_util_derive::GrrVertex;
use nalgebra as na;

#[derive(GrrVertex)]
#[repr(C)]
pub struct VertexVecf32 {
    a: na::Vector2<f32>,
    b: na::Vector3<f32>,
    c: na::Vector4<f32>,
    d: na::Point2<f32>,
    e: na::Point3<f32>,
    f: na::Point4<f32>,
}

#[test]
pub fn test_attribs_vvf32() {
    let attrs = VertexVecf32::attribs(0, 0);
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
            format: grr::VertexFormat::Xyz32Float
        }
    );
    assert_eq!(
        attrs[2],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 2,
            offset: 20,
            format: grr::VertexFormat::Xyzw32Float
        }
    );
    assert_eq!(
        attrs[3],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 3,
            offset: 36,
            format: grr::VertexFormat::Xy32Float
        }
    );
    assert_eq!(
        attrs[4],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 4,
            offset: 44,
            format: grr::VertexFormat::Xyz32Float
        }
    );
    assert_eq!(
        attrs[5],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 5,
            offset: 56,
            format: grr::VertexFormat::Xyzw32Float
        }
    );
    assert_eq!(attrs.len(), 6);
}

#[derive(GrrVertex)]
#[repr(C)]
pub struct VertexVecf64 {
    a: na::Vector2<f64>,
    b: na::Vector3<f64>,
    c: na::Vector4<f64>,
    d: na::Point2<f64>,
    e: na::Point3<f64>,
    f: na::Point4<f64>,
}

#[test]
pub fn test_attribs_vvf64() {
    let attrs = VertexVecf64::attribs(0, 0);
    assert_eq!(
        attrs[0],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 0,
            offset: 0,
            format: grr::VertexFormat::Xy64Float
        }
    );
    assert_eq!(
        attrs[1],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 1,
            offset: 16,
            format: grr::VertexFormat::Xyz64Float
        }
    );
    assert_eq!(
        attrs[2],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 2,
            offset: 40,
            format: grr::VertexFormat::Xyzw64Float
        }
    );
    assert_eq!(
        attrs[3],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 3,
            offset: 72,
            format: grr::VertexFormat::Xy64Float
        }
    );
    assert_eq!(
        attrs[4],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 4,
            offset: 88,
            format: grr::VertexFormat::Xyz64Float
        }
    );
    assert_eq!(
        attrs[5],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 5,
            offset: 112,
            format: grr::VertexFormat::Xyzw64Float
        }
    );
    assert_eq!(attrs.len(), 6);
}

#[derive(GrrVertex)]
#[repr(C)]
pub struct VertexVecMix {
    a: na::Vector2<f64>,
    b: na::Point2<f32>,
}
#[test]
pub fn test_attribs_vvmix() {
    let attrs = VertexVecMix::attribs(0, 0);
    assert_eq!(
        attrs[0],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 0,
            offset: 0,
            format: grr::VertexFormat::Xy64Float
        }
    );
    assert_eq!(
        attrs[1],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 1,
            offset: 16,
            format: grr::VertexFormat::Xy32Float
        }
    );
    assert_eq!(attrs.len(), 2);
}

#[derive(GrrVertex)]
#[repr(C)]
pub struct VertexVecMixAlign {
    a: na::Vector3<f32>,
    b: na::Point2<f64>,
}
#[test]
pub fn test_attribs_vvmix_align() {
    let attrs = VertexVecMixAlign::attribs(0, 0);
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
            offset: 16,
            format: grr::VertexFormat::Xy64Float
        }
    );
    assert_eq!(attrs.len(), 2);
}
