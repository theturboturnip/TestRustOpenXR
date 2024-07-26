#[macro_export]
macro_rules! spv_shader_bytes {
    ($shader_name:expr) => {
        include_bytes!(concat!("shaders/spv/", $shader_name, ".spv"))[..]
    }
}