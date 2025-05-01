# Fragment Shader for Terrain Rendering with Shadows

# This shader handles the color output and texture sampling while incorporating shadow calculations.

# Define the precision for floating-point numbers
#version 330 core

# Input from the vertex shader
in vec2 TexCoords; // Texture coordinates
in vec3 FragPos;   // Fragment position in world space
in vec3 Normal;    // Normal vector

# Output color
out vec4 FragColor;

# Uniforms
uniform sampler2D uGrassTex;       // Grass texture
uniform sampler2D uRockTex;        // Rock texture
uniform sampler2D shadowMap;       // Depth texture for shadows
uniform vec3 lightPos;              // Light position
uniform vec3 viewPos;               // Camera position
uniform vec3 lightColor;            // Light color
uniform vec3 ambientColor;          // Ambient light color

// Function to calculate shadow intensity
float ShadowCalculation(vec4 fragPosLightSpace) {
    // Transform fragment position to light space
    // Get the current fragment's depth value from the shadow map
    float closestDepth = texture(shadowMap, fragPosLightSpace.xy).r; 
    // Get the current fragment's depth value
    float currentDepth = fragPosLightSpace.z; 

    // Calculate shadow factor
    float shadow = currentDepth > closestDepth + 0.005 ? 1.0 : 0.0; // Simple shadow comparison
    return shadow;
}

void main() {
    // Ambient lighting
    vec3 ambient = ambientColor * 0.1;

    // Diffuse lighting
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(lightPos - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = lightColor * diff;

    // Shadow calculations
    vec4 fragPosLightSpace = /* Transform FragPos to light space */;
    float shadow = ShadowCalculation(fragPosLightSpace);

    // Combine results
    vec3 result = (ambient + (1.0 - shadow) * diffuse) * texture(uGrassTex, TexCoords).rgb;
    FragColor = vec4(result, 1.0);
}