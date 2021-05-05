use graphics::data::LocalUniforms;

pub struct StaticModelRequest {
    pub label: String,
    pub uniforms: LocalUniforms,
}

impl StaticModelRequest {
    pub fn new(label: &str, uniforms: LocalUniforms) -> Self {
        Self {
            label: label.to_string(),
            uniforms,
        }
    }
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
