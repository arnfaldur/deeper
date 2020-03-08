#version 330 core
layout (location = 0) in vec3 aPos;

uniform mat4 uModelMatrix;
uniform mat4 uViewMatrix;
uniform mat4 uProjectionMatrix;

out float height;

void main(void) {

     mat4 view = mat4(mat3(uViewMatrix));
     vec4 position = uProjectionMatrix * view * vec4(aPos, 1.0);

     height = aPos.y;

     gl_Position = position.xyww;
}

