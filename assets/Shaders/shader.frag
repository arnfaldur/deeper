#version 450 core
out vec4 FragColor;

layout (location = 0) uniform sampler2D uTextureDiffuse;
layout (location = 1) uniform sampler2D uTextureSpecular;
layout (location = 2) uniform sampler2D uTextureNormal;

uniform mat4 uModelMatrix;
uniform mat4 uViewMatrix;
uniform mat4 uProjectionMatrix;

uniform vec4 uEyePosition;

// Hacky bullshit
uniform bool uSelected;

uniform bool uTextured;
uniform bool uNormaled;
uniform bool uSpecmapped;

// Varying parameters from vertex shader
in vec2 vTexCoords;
in vec4 vFragPos;
in vec4 vNormal;
in mat3 vTangentMatrix;

in vec4 vTangent;
in vec4 vBitangent;

struct Material {
    vec4 diffuse;
    vec4 specular;

    float shininess;
};

uniform Material uMaterial;

struct DirectionalLight {
    int is_lit;
    vec4 direction;

    vec4 ambient;
    vec4 color;
};

// TODO: make programmable in shader
#define MAX_NR_OF_DIRECTIONAL_LIGHTS 3
uniform DirectionalLight uDirectionalLights[MAX_NR_OF_DIRECTIONAL_LIGHTS];

struct PointLight {
    int is_lit;
    float radius;

    vec4 position;
    vec4 color;
};

// TODO: make programmable in shader
#define MAX_NR_OF_POINT_LIGHTS 5
uniform PointLight uPointLights[MAX_NR_OF_POINT_LIGHTS];

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

vec4 fdirectionallightfactor(directionallight light, vec4 normal, vec4 viewdir, material material) {
    vec4 lightdir = normalize(light.direction);
    vec4 diffuse  = material.diffuse  * flambert(normal, lightdir);
    vec4 specular = material.specular * fphong(normal, lightdir, 64);
    //vec4 specular = material.specular * fblinnphong(normal, light.direction, viewdir, material.shininess);
    return light.ambient * material.diffuse + light.color * (diffuse + specular);
}

vec4 fPointLightFactor(PointLight light, vec4 normal, vec4 viewDir, Material material) {
    vec4 toLight  = light.position - vFragPos;
    vec4 lightDir = normalize(toLight);

    float intensity = fLightFalloff(length(toLight), light.radius, 1.0);

    vec4 diffuse  = material.diffuse  * fLambert(normal, lightDir);
    vec4 specular = material.specular * fPhong(normal, lightDir, material.shininess);
    //vec4 specular = material.specular * fBlinnPhong(normal, lightDir, viewDir, material.shininess);

    return intensity * light.color * (diffuse + specular);
}

void main()
{
    FragColor = vec4(0.0);

    vec4 normal  = normalize(vNormal);
    vec4 viewDir = normalize(uEyePosition - vFragPos);

    Material material = uMaterial;

	if (uTextured) {
        material.diffuse = texture(uTextureDiffuse, vTexCoords);
    }

    if(uNormaled) {
		normal = vec4(normalize(vTangentMatrix * normalize(texture(uTextureNormal, vTexCoords).rgb * 2 - 1)),0.0);
    }

    if(uSpecmapped) {
		material.specular = texture(uTextureSpecular, vTexCoords);
    }

    for (int i = 0; i < MAX_NR_OF_POINT_LIGHTS; i++) {
        if (uPointLights[i].is_lit == 1) {
            FragColor += fPointLightFactor(uPointLights[i], normal, viewDir, material);
        }
    }

    for (int i = 0; i < MAX_NR_OF_DIRECTIONAL_LIGHTS; i++) {
        if (uDirectionalLights[i].is_lit == 1) {
            FragColor += fDirectionalLightFactor(uDirectionalLights[i], normal, viewDir, material);
        }
    }

    // Fresnel attempt
    if (uSelected) {
		float R = 0.0 + 0.2 * pow(1.0 + dot(viewDir, normal), 4);
		FragColor = mix(vec4(1.0,1.0,0.2,1.0), FragColor, R);
    }
}
