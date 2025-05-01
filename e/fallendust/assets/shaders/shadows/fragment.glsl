#version 330 core

in vec2 TexCoords;
in float ShadowCoord;

out vec4 FragColor;

uniform sampler2D shadowMap;
uniform vec3 lightPos;
uniform vec3 viewPos;

void main()
{
    // Calculate the shadow intensity
    float shadow = texture(shadowMap, vec2(ShadowCoord)).r;

    // Simple shadow calculation
    float shadowIntensity = shadow < 0.5 ? 0.5 : 1.0;

    // Output the final color with shadow effect
    FragColor = vec4(vec3(shadowIntensity), 1.0);
}