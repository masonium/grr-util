pub mod image_format;
pub mod image_manager;
pub mod mesh;
pub mod screenshot;
pub mod shader_manager;
pub mod window;

pub use image_manager::ImageManager;
pub use num_traits::Zero;
pub use shader_manager::{ShaderDesc, ShaderManager};
pub use window::{GrrBuilder, GrrHeadless, GrrWindow};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
