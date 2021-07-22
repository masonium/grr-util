use grr_util::vertex::GrrVertex;
use grr_util_derive::GrrVertex;

#[derive(GrrVertex)]
#[repr(C)]
pub struct V1 {
    a: f32,
    b: f32,
    c: f64,
}

#[test]
pub fn test_attribs_v1() {
    let attrs = V1::attribs(0, 0);
    assert_eq!(
        attrs[0],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 0,
            offset: 0,
            format: grr::VertexFormat::X32Float
        }
    );
    assert_eq!(
        attrs[1],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 1,
            offset: 4,
            format: grr::VertexFormat::X32Float
        }
    );
    assert_eq!(
        attrs[2],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 2,
            offset: 8,
            format: grr::VertexFormat::X64Float
        }
    );
}
