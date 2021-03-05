#version 450


struct Material {
    vec4  albedo;
    float metallic;
    float roughness;
};

layout(location = 0) in vec2 v_TexCoord;
layout(location = 1) in vec3 v_Color;

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
    vec4 u_Eye_Position;
};

layout(set = 1, binding = 0) uniform Locals {
    mat4 u_ModelMatrix;
    Material material;
};

//layout(set = 2, binding = 0) uniform texture2D t_Diffuse;
//layout(set = 2, binding = 1) uniform sampler s_Diffuse;

void main() {
    //o_Target = texture(sampler2D(t_Diffuse, s_Diffuse), v_TexCoord);
    o_Target = vec4(v_Color, 1.0);
}
