#version 330 core
layout (location = 0) in vec3 aPos;

uniform mat4 uModelMatrix;
uniform mat4 uViewMatrix;
uniform mat4 uProjectionMatrix;

void main()
{
    vec4 position = vec4(aPos, 1.0);
    gl_Position = uProjectionMatrix * uViewMatrix * uModelMatrix * position;
}
