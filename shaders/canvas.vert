#version 450

layout(location = 0) in vec2 a_Pos;
layout(location = 1) in vec2 a_TexCoord;

layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec3 v_Color;

layout(set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
    vec4 u_Eye_Position;
};

layout(set = 1, binding = 0) uniform Locals {
    mat4 u_ModelMatrix;
    vec3 u_Color;
};

const vec2 positions[3] = vec2[3](
    vec2(0.0, 0.5),
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5)
);

void main() {
    vec4 position = vec4(a_Pos, 0.0, 1.0);

    v_Color = u_Color;
    v_TexCoord = a_TexCoord;

    gl_Position = u_ViewProj * u_ModelMatrix * vec4(a_Pos, 0.0, 1.0);
    gl_Position = u_ViewProj * u_ModelMatrix * vec4(a_Pos, 0.0, 1.0);
}
