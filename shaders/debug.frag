#version 450

// TODO: Inject into shader
#define MAX_NR_OF_POINT_LIGHTS 10

const float PI = 3.14159265359;

struct DirectionalLight {
    vec4 direction;
    vec4 ambient;
    vec4 color;
};

struct PointLight {
    float radius;
    vec4 position;
    vec4 color;
};

struct Material {
    vec4  albedo;
    float metallic;
    float roughness;
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

layout(set = 1, binding = 0) uniform Locals {
    mat4 u_ModelMatrix;
    Material material;
};


// The following color space functions are from
// http://www.chilliant.com/rgb2hsv.html
const float Epsilon = 1e-10;
const vec3 HCYwts = vec3(0.299, 0.587, 0.114);

vec3 HUEtoRGB(in float H)
{
    float R = abs(H * 6 - 3) - 1;
    float G = 2 - abs(H * 6 - 2);
    float B = 2 - abs(H * 6 - 4);
    return clamp(vec3(R,G,B), 0.0, 1.0);
}

vec3 RGBtoHCV(vec3 RGB)
{
    // Based on work by Sam Hocevar and Emil Persson
    vec4 P = (RGB.g < RGB.b) ? vec4(RGB.bg, -1.0, 2.0/3.0) : vec4(RGB.gb, 0.0, -1.0/3.0);
    vec4 Q = (RGB.r < P.x) ? vec4(P.xyw, RGB.r) : vec4(RGB.r, P.yzx);
    float C = Q.x - min(Q.w, Q.y);
    float H = abs((Q.w - Q.y) / (6 * C + Epsilon) + Q.z);
    return vec3(H, C, Q.x);
}

vec3 RGBtoHCY(vec3 RGB)
{
    // Corrected by David Schaeffer
    vec3 HCV = RGBtoHCV(RGB);
    float Y = dot(RGB, HCYwts);
    float Z = dot(HUEtoRGB(HCV.x), HCYwts);
    if (Y < Z)
    {
        HCV.y *= Z / (Epsilon + Y);
    }
    else
    {
        HCV.y *= (1 - Z) / (Epsilon + 1 - Y);
    }
    return vec3(HCV.x, HCV.y, Y);
}
// The weights of RGB contributions to luminance.
// Should sum to unity.

vec3 HCYtoRGB(vec3 HCY) {
    vec3 RGB = HUEtoRGB(HCY.x);
    float Z = dot(RGB, HCYwts);
    if (HCY.z < Z) {
        HCY.y *= HCY.z / Z;
    } else if (Z < 1) {
        HCY.y *= (1 - HCY.z) / (1 - Z);
    }
    return (RGB - Z) * HCY.y + HCY.z;
}

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

// https://learnopengl.com/PBR/Lighting

vec3 fFresnelSchlick(float cos_theta, vec3 F_0) {
    return F_0 + (1.0 - F_0) * pow(1.0 - cos_theta, 5.0);
}

float fDistributionGGX(vec3 N, vec3 H, float roughness) {
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float fGeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r*r) / 16.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

float fGeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = fGeometrySchlickGGX(NdotV, roughness);
    float ggx1  = fGeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
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

float contrast(float a, float x) {
    return clamp(a * (cos(x * PI + PI) + 1) / 2.0 + ((1-a)*x), 0.0, 1.0);
}

vec3 fLightFactor(vec3 normal, float distance, float radius, vec3 color, vec3 lightDir, vec3 viewDir, vec3 F_0, Material mat) {
    vec3 halfway = normalize(lightDir + viewDir);

    float attenuation = fLightFalloff(distance, radius, 3.0);
    vec3 radiance = color * attenuation;

    float NDF = fDistributionGGX(normal, halfway, mat.roughness);
    float G = fGeometrySmith(normal, viewDir, lightDir, mat.roughness);
    vec3 F = fFresnelSchlick(max(dot(halfway, viewDir), 0.0), F_0);

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - mat.metallic;

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(normal, viewDir), 0.0) * max(dot(normal, lightDir), 0.0);
    vec3 specular = numerator / max(denominator, 0.001);

    float specular_falloff = fLightFalloff(distance, radius, 5.0);
    float NdotL = max(dot(normal, lightDir), 0.0);

    return (kD * vec3(mat.albedo) / PI + specular_falloff * specular) * radiance * NdotL;
}

void main() {
    vec4 finalColor = vec4(0.0);

    vec3 normal = normalize(v_Normal.xyz);
    vec3 viewDir = normalize(u_Eye_Position.xyz - v_FragPos.xyz);

    Material mat = material;

    vec3 F_0 = vec3(0.1);
    F_0 = mix(F_0, vec3(mat.albedo), mat.metallic);

    vec3 Lo = vec3(0.0);

    for(int i = 0; i < MAX_NR_OF_POINT_LIGHTS; i++) {
        PointLight light = uPointLights[i];
        vec3 to_light = light.position.xyz - v_FragPos.xyz;
        vec3 lightDir = normalize(to_light);

        Lo += fLightFactor(
            normal,
            length(to_light),
            light.radius,
            light.color.xyz,
            lightDir,
            viewDir,
            F_0,
            mat
        );
    }

    vec3 ambient = uDirectionalLight.ambient.rgb * vec3(mat.albedo);
    vec3 color = ambient + Lo;

    vec3 lightDir = normalize(uDirectionalLight.direction.xyz);
    vec3 halfway = normalize(lightDir + viewDir);
    vec3 radiance = uDirectionalLight.color.xyz;

    float NDF = fDistributionGGX(normal, halfway, mat.roughness);
    float G = fGeometrySmith(normal, viewDir, lightDir, mat.roughness);
    vec3 F = fFresnelSchlick(max(dot(halfway, viewDir), 0.0), F_0);

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - mat.metallic;

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(normal, viewDir), 0.0) * max(dot(normal, lightDir), 0.0);
    vec3 specular = numerator / max(denominator, 0.001);

    float NdotL = max(dot(normal, lightDir), 0.0);
    color += (kD * vec3(mat.albedo) / PI + specular) * NdotL * radiance;


    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));

    color = RGBtoHCY(color);
    color.z += 0.1;
    color.z = contrast(1.6, color.z);
    color = HCYtoRGB(color);

    o_Target = vec4(color, 0.2);
}
