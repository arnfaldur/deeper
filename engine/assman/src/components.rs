use graphics::data::LocalUniforms;

pub struct StaticModelRequest {
    pub label: String,
    pub uniforms: LocalUniforms,
}

pub struct DynamicModelRequest {
    pub label: String,
}

impl DynamicModelRequest {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }
}
