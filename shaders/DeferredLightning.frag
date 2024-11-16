#version 450

layout (set = 0, binding = 0) uniform sampler2D samplerColor;
layout (set = 0, binding = 1) uniform sampler2D samplerNormal;
layout (set = 0, binding = 2) uniform sampler2D samplerPosition;
layout (set = 0, binding = 3) uniform sampler2D samplerShadowMap;

layout (push_constant) uniform Push {
    mat4 lightSpace;
    vec3 view;
} push;

layout (location = 0) in vec2 inPos;

layout (location = 0) out vec4 outColor;

const float PI = 3.14159265359;

vec3 Specular(float N, float G, vec3 F, float NoV, float NoL);

float NormalDistribution(vec3 N, vec3 VhL, float roughness);
float GeometricShadowing(vec3 N, vec3 V, vec3 L, float roughness);
float SchlickGGX(vec3 N, vec3 R, float roughness);
vec3 Fresnel(vec3 F0, vec3 V, vec3 VhL);

float Shadow(vec4 worldPosition);

void main()
{
    vec4 sampledColor = texture(samplerColor, inPos);
    vec4 sampledNormal = texture(samplerNormal, inPos);
    vec4 sampledPosition = texture(samplerPosition, inPos);
    vec4 sampledShadowMap = texture(samplerShadowMap, inPos);

    vec3 normal = normalize(sampledNormal.xyz);

    float metallic = sampledNormal.w;
    float roughness = sampledPosition.w;

    vec3 light = normalize(vec3(0.0, 0.0, -1.0));

    float NoV = dot(normal, push.view);
    float NoL = dot(normal, light);

    vec3 VhL = normalize(push.view + light);

    vec3 F0 = mix(vec3(0.04), sampledColor.xyz, metallic);

    float normalDistribution = NormalDistribution(normal, VhL, roughness);
    float geometricShadowing = GeometricShadowing(normal, push.view, light, roughness);
    vec3 fresnel = Fresnel(F0, push.view, VhL);

    vec3 specular = Specular(normalDistribution, geometricShadowing, fresnel, NoV, NoL);

    vec3 kD = mix(vec3(1.0) - fresnel, vec3(0.0), metallic);

    vec3 diffuse = sampledColor.xyz / PI;

    float shadow = Shadow(sampledPosition);
    vec3 finalColor = (kD * diffuse + specular) * NoL * (1.0 - shadow);

    finalColor = finalColor / (finalColor + vec3(1.0));
    finalColor = pow(finalColor, vec3(1.0/2.2));

    outColor = vec4(finalColor, 1.0);

    //outColor = vec4(shadow, shadow, shadow, 1.0);
}

vec3 Specular(float N, float G, vec3 F, float NoV, float NoL)
{
    vec3 numerator = N * G * F;
    float denominator = 4.0 * NoV * NoL;

    return numerator / denominator;
}

// GGX (Trowbridge-Reitz)
// N = Normal
// VhL = Half (View, Light)
float NormalDistribution(vec3 N, vec3 VhL, float roughness)
{
    float a = roughness * roughness;
    float a2 = a * a;
    float NoH = max(dot(N, VhL), 0.0);
    float NoH2 = NoH * NoH;

    float denominator = a2 - 1.0;
    denominator = NoH2 * denominator + 1.0;
    denominator = denominator * denominator;
    denominator = PI * denominator;

    return a2 / denominator;
}


// Smith
// L = Light
// V = View
// VhL = Half (View, Light)
float GeometricShadowing(vec3 N, vec3 V, vec3 L, float roughness)
{
    float g1 = SchlickGGX(N, V, roughness);
    float g2 = SchlickGGX(N, L, roughness);

    return g1 * g2;
}

// N = Normal
// R = View/Light
float SchlickGGX(vec3 N, vec3 R, float roughness)
{
    float remappedRoughness = roughness + 1.0;
    float k = (remappedRoughness * remappedRoughness) / 8.0;

    float NoR = max(dot(N, R), 0.0);

    float denominator = NoR * (1.0 - k) + k;

    return NoR / denominator;
}

// Schlick
// F0 = Normal Incidence
// V = View
// VhL = Half (View, Light)
vec3 Fresnel(vec3 F0, vec3 V, vec3 VhL)
{
    float VoH = dot(V, VhL);
    return F0 + (1.0 - F0) * pow(1.0 - VoH, 5.0);
}

float Shadow(vec4 worldPosition) {
    //return 0.0;
    // roughness is stored in worldPosition.w
    vec4 lightSpacePosition2 = push.lightSpace * vec4(worldPosition.rgb, 1.0);
    vec3 lightSpacePosition = lightSpacePosition2.rgb/* / lightSpacePosition2.w*/;
    lightSpacePosition = lightSpacePosition * 0.5 + 0.5;
    float closestDepth = texture(samplerShadowMap, lightSpacePosition.xy).r;
    float currentDepth = lightSpacePosition2.z;

    // TODO: dot bias
    float bias = 0.005;
    float shadow = currentDepth - bias > closestDepth ? 1.0 : 0.0;

    //return closestDepth;
    return shadow;
}