#version 330 core

in vec3 fragPosition;
in vec2 fragTexCoord;
in vec4 fragColor;
in vec3 fragNormal;

uniform vec3 eyePosition;

// Final output color
out vec4 finalColor;

// modified equation (9) from 'Real Shading in Unreal Engine 4' by Brian Karis
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

struct DirectionalLight {
    vec4 direction;
    vec4 ambient;
    vec4 color;
};

struct Material {
    vec4 diffuse;
    vec4 specular;
    float shininess;
};

vec4 fDirectionalLightFactor(DirectionalLight light, vec4 normal, Material material) {
    vec4 lightDir = normalize(light.direction);
    vec4 diffuse  = material.diffuse  * fLambert(normal, lightDir);
    vec4 specular = material.specular * fPhong(normal, lightDir, 64);
    //vec4 specular = material.specular * fBlinnPhong(normal, light.direction, viewDir, material.shininess);
    return light.ambient * material.diffuse + light.color * (diffuse + specular);
}

struct PointLight {
    int is_lit;
    float radius;

    vec3 position;
    vec4 color;
};

#define MAX_NR_OF_POINT_LIGHTS 10
uniform PointLight uPointLights[MAX_NR_OF_POINT_LIGHTS];

vec4 fPointLightFactor(PointLight light, vec4 normal, vec4 viewDir, Material material) {
    vec4 toLight  = vec4(light.position - fragPosition, 0.0);
    vec4 lightDir = normalize(toLight);

    float intensity = fLightFalloff(length(toLight), light.radius, 1.0);

    vec4 diffuse  = material.diffuse  * fLambert(normal, lightDir);
    vec4 specular = material.specular * fPhong(normal, lightDir, material.shininess);
    //vec4 specular = material.specular * fBlinnPhong(normal, lightDir, viewDir, material.shininess);

    //return intensity * light.color * (diffuse + specular);
    return intensity * light.color * (diffuse + specular);
}

void main() {
    DirectionalLight dl;
    dl.direction = vec4(1.0, 0.8, 0.8, 0.0);
    //dl.ambient   = vec4(0.2, 0.2, 0.2, 1.0) ;
    dl.color     = vec4(0.3, 0.3, 0.2, 1.0);

    Material mat;
    mat.diffuse   = fragColor;
    mat.specular  = vec4(vec3(0.0), 1.0);
    mat.shininess = 0.0;

    vec4 viewDir = vec4(eyePosition - fragPosition, 0.0);
    vec4 normal = vec4(fragNormal, 0.0);

    finalColor = fDirectionalLightFactor(dl, normal, mat);

    PointLight test;
    test.is_lit = 1;
    test.radius = 150.0;
    test.position = vec3(0.0, 1.0, 0.0);
    test.color = vec4(1.0);

    //finalColor += fPointLightFactor(test, normal, viewDir, mat);

    for (int i = 0; i < MAX_NR_OF_POINT_LIGHTS; i++) {
        finalColor += fPointLightFactor(uPointLights[i], normal, viewDir, mat);
    }

    finalColor = vec4(vec3(finalColor), 1.0);
}