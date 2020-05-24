mod shader_manager;

pub use shader_manager::{ShaderManager, ShaderDesc};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
