#version 330 core

// Input vertex attributes
layout (location = 0) in vec3 vertexPosition;
layout (location = 1) in vec2 vertexTexCoord;
layout (location = 2) in vec3 vertexNormal;
layout (location = 3) in vec4 vertexColor;
layout (location = 4) in vec4 vertexTangent;
// layout (location = 5) in vec2 vertexTexCoord2; // Possibly useful

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
out vec4 fragTangent;
out mat3 fragTangentMatrix;

void main() {
    fragPosition = vec3(matModel * vec4(vertexPosition, 1.0));
    //fragPosition = vec3(inverse(view) * inverse(projection) * mvp * vec4(vertexPosition, 1.0));
    fragColor = colDiffuse;
    fragTexCoord = vertexTexCoord;
    fragTangent = vertexTangent;

    vec3 T = normalize(vec3(matModel * vertexTangent));
    vec3 N = normalize(vec3(mat3(matModel) * vertexNormal));
    // Make sure that tangent is orthogonal to normal
    T = normalize(T - dot(T, N) * N);
    vec3 B = normalize(cross(N, T));
    fragTangentMatrix = mat3(T, B, N);

    mat3 normalMatrix = transpose(inverse(mat3(matModel)));
    fragNormal = normalize(normalMatrix*vertexNormal);

    gl_Position = mvp * vec4(vertexPosition, 1.0);
}