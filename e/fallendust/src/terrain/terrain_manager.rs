#version 330 core

in vec2 TexCoords;
out vec4 FragColor;

uniform sampler2D shadowMap;
uniform vec3 lightPos;
uniform vec3 viewPos;

void main()
{
    // Calculate the shadow intensity
    float shadow = texture(shadowMap, TexCoords).r;

    // Simple lighting calculation
    vec3 lightDir = normalize(lightPos - FragColor.rgb);
    float diff = max(dot(normalize(FragColor.rgb), lightDir), 0.0);

    // Combine shadow with diffuse lighting
    FragColor = vec4(diff * (1.0 - shadow), 1.0);
}