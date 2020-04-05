#version 450 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec3 aTangent;
layout (location = 3) in vec3 aBitangent;
layout (location = 4) in vec2 aTexCoords;

uniform mat4 uModelMatrix;
uniform mat4 uViewMatrix;
uniform mat4 uProjectionMatrix;

out vec2 vTexCoords;
out vec4 vFragPos;
out vec4 vNormal;
out mat3 vTangentMatrix;

void main()
{
    vTexCoords = aTexCoords;

    vec4 position = vec4(aPos, 1.0);
    vec3 T = normalize(vec3(uModelMatrix * vec4(aTangent,   0.0)));
    vec3 N = normalize(vec3(uModelMatrix * vec4(aNormal,    0.0)));
	// Make sure tangent is orthogonal to the normal
    T = normalize(T - dot(T, N) * N);
    vec3 B = normalize(vec3(uModelMatrix * vec4(aBitangent, 0.0)));

	// Make sure that the bitangent points in the right direction
    if(dot(cross(N.xyz, T.xyz), B.xyz) < 0.0) {
      T = T * -1;
    }
    B = cross(T, N);
    vTangentMatrix = mat3(T,B,N);

    vNormal = normalize(uModelMatrix * vec4(aNormal, 0.0));
    vFragPos = uModelMatrix * position;

    gl_Position = uProjectionMatrix * uViewMatrix * uModelMatrix * position;
}
