use grr_util::vertex::GrrVertex;
use grr_util_derive::GrrVertex;
use palette::Srgb;

#[derive(GrrVertex)]
#[repr(C)]
pub struct C1 {
    a: Srgb<u8>,
}

#[test]
pub fn test_attribs_c1() {
    let attrs = C1::attribs(0, 0);
    assert_eq!(
        attrs[0],
        grr::VertexAttributeDesc {
            binding: 0,
            location: 0,
            offset: 0,
            format: grr::VertexFormat::Xyz8Unorm
        }
    );
}
