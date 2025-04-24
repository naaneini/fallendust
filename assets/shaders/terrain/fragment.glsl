#version 330 core

in vec3 vNormal;
in vec3 vWorldPos;

uniform sampler2D uGrassTex;
uniform sampler2D uStoneTex;
uniform vec3 uLightDir; // Should be normalized (direction from surface to light)

out vec4 FragColor;

void main()
{
    // Normalize the normal (interpolation can make it not unit length)
    vec3 normal = normalize(vNormal);
    
    // Calculate how "up-facing" the normal is (0 = side, 1 = top)
    float topBlend = smoothstep(0.6, 0.9, -normal.y);
    
    // Tri-planar mapping for stone texture
    vec3 blending = abs(normal);
    blending = normalize(max(blending, 0.00001)); // Ensure weights don't get zero
    float b = (blending.x + blending.y + blending.z);
    blending /= vec3(b, b, b);
    
    // Sample stone texture from three projections
    vec2 stoneUVX = vWorldPos.zy * 0.1; // X-facing plane
    vec2 stoneUVY = vWorldPos.xz * 0.1; // Y-facing plane (top/bottom)
    vec2 stoneUVZ = vWorldPos.xy * 0.1; // Z-facing plane
    
    vec4 stoneX = texture(uStoneTex, stoneUVX);
    vec4 stoneY = texture(uStoneTex, stoneUVY);
    vec4 stoneZ = texture(uStoneTex, stoneUVZ);
    
    // Blend the stone textures based on normal direction
    vec4 stoneColor = stoneX * blending.x + stoneY * blending.y + stoneZ * blending.z;
    
    // Sample grass texture (only on top)
    vec2 grassUV = vWorldPos.xz * 0.5;
    vec4 grassColor = texture(uGrassTex, grassUV);
    
    // Blend based on normal direction
    vec4 finalColor = mix(stoneColor, grassColor, topBlend);
    
    // Simple diffuse lighting (lambertian)
    float diffuse = max(dot(normal, -uLightDir), 0.0);
    
    // Add some ambient light so shadows aren't completely black
    float ambient = 0.2;
    float lighting = ambient + (1.0 - ambient) * diffuse;
    
    // Apply lighting to the final color
    FragColor = finalColor * lighting;
}