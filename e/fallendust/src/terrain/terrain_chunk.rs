#version 330 core

in vec2 TexCoords;
in float ShadowCoord;

out vec4 FragColor;

uniform sampler2D shadowMap;
uniform vec3 lightPos;

void main()
{
    // Calculate shadow intensity
    float shadow = 0.0;
    float closestDepth = texture(shadowMap, TexCoords).r; // Get the depth from the shadow map
    float currentDepth = ShadowCoord; // Current fragment depth

    // Compare the current depth with the closest depth
    if (currentDepth > closestDepth + 0.005) {
        shadow = 1.0; // In shadow
    }

    // Output color with shadow effect
    FragColor = vec4(vec3(1.0 - shadow), 1.0); // White color with shadow
}