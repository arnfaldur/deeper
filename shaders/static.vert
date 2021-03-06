#version 450

layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec3 a_Normal;
layout(location = 2) in vec2 a_TexCoord;

layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec3 v_Color;
layout(location = 2) out vec4 v_FragPos;
layout(location = 3) out vec4 v_Normal;

layout(set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
    vec4 u_Eye_Position;
};

layout(set = 1, binding = 0) uniform Locals {
    mat4 u_ModelMatrix;
    vec3 u_Color;
};

void main() {
    vec4 position = vec4(a_Pos, 1.0);

    v_FragPos = position;
    v_Normal = vec4(a_Normal, 0.0);

    v_Color = u_Color;
    v_TexCoord = a_TexCoord;

    gl_Position = u_ViewProj * position;
}
