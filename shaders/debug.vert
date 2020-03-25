#version 450

layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec2 a_TexCoord;

layout(location = 0) out vec2 v_TexCoord;

layout(set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
};

layout(set = 1, binding = 0) uniform Locals {
    mat4 u_ModelMatrix;
};

void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = u_ViewProj * vec4(a_Pos, 1.0);
}
