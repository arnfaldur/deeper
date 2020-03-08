#version 330 core

// Input vertex attributes
in vec3 vertexPosition;
in vec2 vertexTexCoord;
in vec3 vertexNormal;
in vec4 vertexColor;

uniform vec4 colDiffuse;

// Input uniform values
uniform mat4 mvp;
uniform mat4 projection;      // VS: Projection matrix
uniform mat4 view;            // VS: View matrix
uniform mat4 matModel;

// Output vertex attributes (to fragment shader)
out vec3 fragPosition;
out vec2 fragTexCoord;
out vec4 fragColor;
out vec3 fragNormal;

void main() {
    fragPosition = vec3(matModel * vec4(vertexPosition, 1.0));

    fragColor = colDiffuse;
    fragTexCoord = vertexTexCoord;

    mat3 normalMatrix = transpose(inverse(mat3(matModel)));
    fragNormal = normalize(normalMatrix*vertexNormal);

    gl_Position = mvp * vec4(vertexPosition, 1.0);
}