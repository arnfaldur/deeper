#version 450

// TODO: Inject into shader
#define MAX_NR_OF_POINT_LIGHTS 10

struct DirectionalLight {
    vec4 direction;
    vec4 ambient;
    vec4 color;
};

struct PointLight {
    vec4 position;
    vec4 color;
};

struct Material {
    vec4 diffuse;
    vec4 specular;

    float shininess;
};

layout(location = 0) in vec2 v_TexCoord;
layout(location = 1) in vec3 v_Color;
layout(location = 2) in vec4 v_FragPos;
layout(location = 3) in vec4 v_Normal;

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 0) uniform Globals {
    mat4 u_ViewProj;
    vec4 u_Eye_Position;
};

layout(set = 0, binding = 1) uniform Lights {
    DirectionalLight uDirectionalLight;
    PointLight uPointLights[MAX_NR_OF_POINT_LIGHTS];
};

float fLightFalloff(float distance, float lightRadius, float scale) {
    //
    //           saturate(1 - (distance/lightRadius)^4)^2
    // falloff = ----------------------------------------       (9)
    //                      distance^2 + 1
    //
    // Note(j):
    // Apparently "saturate" is just clamp(x, 0.0, 1.0) and is a HLSL term
    //
    distance = distance / scale;
    return pow(clamp(1 - pow(distance/lightRadius, 4), 0.0, 1.0),2) / (pow(distance, 2) + 1);
}

float fLambert(vec4 normal, vec4 lightDir) {
    return max(dot(normal, lightDir), 0.0); // lambert
}

float fPhong(vec4 normal, vec4 lightDir, float shininess) {
    vec4 reflectDir = reflect(-lightDir, normal);
    return pow(max(dot(normal, reflectDir), 0.0), shininess); // phong
}

float fBlinnPhong(vec4 normal, vec4 lightDir, vec4 viewDir, float shininess) {
    vec4 halfway = normalize(lightDir + viewDir);
    return pow(max(dot(normal, halfway), 0.0), 3*shininess); // blinn-phong
}

vec4 fPointLightFactor(PointLight light, vec4 normal, Material material) {
    vec4 toLight  = light.position - v_FragPos;
    vec4 lightDir = normalize(toLight);

    float intensity = fLightFalloff(length(toLight), 20.0, 3.0);

    vec4 diffuse  = material.diffuse  * fLambert(normal, lightDir);
    vec4 specular = material.specular * fPhong(normal, lightDir, material.shininess);
    //vec4 specular = material.specular * fBlinnPhong(normal, lightDir, viewDir, material.shininess);

    //return intensity * light.color * (diffuse + specular);
    return intensity * light.color * (diffuse + specular);
}

vec4 fDirectionalLightFactor(DirectionalLight light, vec4 normal, Material material) {
    vec4 lightDir = normalize(light.direction);
    vec4 diffuse  = material.diffuse  * fLambert(normal, lightDir);
    vec4 specular = material.specular * fPhong(normal, lightDir, 64);
    // TODO: fix
    //vec4 specular = material.specular * fBlinnPhong(normal, light.direction, viewDir, material.shininess);
    return light.ambient * material.diffuse + light.color * (diffuse + specular);
}

void main() {
    vec4 finalColor = vec4(0.0);

    vec4 normal = normalize(v_Normal);

    Material mat;
    mat.diffuse = vec4(v_Color, 1.0);
    mat.specular = vec4(1.0);
    mat.shininess = 64;

    for(int i = 0; i < MAX_NR_OF_POINT_LIGHTS; i++) {
        finalColor += fPointLightFactor(
            uPointLights[i],
            normal,
            mat
        );
    }

    finalColor += fDirectionalLightFactor(uDirectionalLight, normal, mat);

    //finalColor += fPointLightFactor(uPointLights[0], normal, mat);

    o_Target = finalColor;
}
