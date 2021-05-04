#version 330 core
in vec3 vertexPosition;

uniform mat4 projection;
uniform mat4 view;

out float height;

void main(void) {

     mat4 t_view = mat4(mat3(view));
     vec4 position = projection * t_view * vec4(vertexPosition, 1.0);

     height = vertexPosition.y;

     gl_Position = position.xyww;
}

