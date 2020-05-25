pub mod shader_manager;
pub mod window;

pub use shader_manager::{ShaderDesc, ShaderManager};
pub use window::GrrWindow;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
