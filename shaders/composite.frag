#version 450

layout(push_constant) uniform Push {
    mat4 model;
    mat4 view;
    mat4 projection;
    vec3 viewPosition;
} push;

layout (input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput inputColor;
layout (input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput inputPosition;
layout (input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput inputNormal;

layout (location = 0) out vec4 outColor;

void main()
{
    vec3 lightDirection = normalize(vec3(1.0, -1.0, 1.0));
    vec3 lightColor = vec3(1.0, 1.0, 1.0);

    vec3 color = subpassLoad(inputColor).rgb;
    vec3 position = subpassLoad(inputPosition).rgb;
    vec3 normal = normalize(normalize(subpassLoad(inputNormal).rgb));

    float ambientValue = 0.1;

    float diffuseValue = max(0.0, dot(normal, -lightDirection));

    vec3 viewDirection = normalize(push.viewPosition - position);
    vec3 reflectDirection = reflect(lightDirection, normal);

    float specularAngle = max(0.0, dot(viewDirection, reflectDirection));
    float specularValue = pow(specularAngle, 16.0);

    vec3 finalColor = (ambientValue + diffuseValue + specularValue) * lightColor * color;
    outColor = vec4(finalColor, subpassLoad(inputColor).a);
}