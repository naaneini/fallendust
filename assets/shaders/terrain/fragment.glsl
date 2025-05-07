#version 330 core
out vec4 FragColor;

in VS_OUT {
    vec3 FragPos;
    vec3 Normal;
    vec4 FragPosLightSpace;
} fs_in;

uniform sampler2D uGrassTex;
uniform sampler2D uGrassNormal;
uniform sampler2D uRockTex;
uniform sampler2D uRockNormal;

uniform sampler2D shadowMap;

uniform vec3 lightPos;
uniform vec3 viewPos;
uniform vec3 lightDir;
uniform float normalStrength = 1.0;

float ShadowCalculation(vec4 fragPosLightSpace)
{
    // perform perspective divide
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    // transform to [0,1] range
    projCoords = projCoords * 0.5 + 0.5;

    // get closest depth value from light's perspective (using [0,1] range fragPosLight as coords)
    float closestDepth = texture(shadowMap, projCoords.xy).r; 
    // get depth of current fragment from light's perspective
    float currentDepth = projCoords.z;
    // calculate bias (based on depth map resolution and slope)
    vec3 normal = normalize(fs_in.Normal);
    vec3 lightDir = normalize(lightPos - fs_in.FragPos);
    float lightDistance = length(lightPos - fs_in.FragPos);
    float bias = max(0.005 * (1.0 - dot(normal, lightDir)) / lightDistance, 0.005);

    // PCF
    float shadow = 0.0;
    int kernelSize = 15; // Increased kernel size for more samples
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    int sampleCount = kernelSize * kernelSize;
    int halfKernelSize = kernelSize / 2;

    for(int x = -halfKernelSize; x <= halfKernelSize; ++x)
    {
        for(int y = -halfKernelSize; y <= halfKernelSize; ++y)
        {
            float pcfDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r;
            // Smooth falloff using smoothstep
            float depthDifference = currentDepth - bias - pcfDepth;
            float shadowFactor = smoothstep(0.0, 0.04, depthDifference); // Adjust the 0.0 and 0.05 values to control the smoothness
            shadow += shadowFactor;
        }    
    }
    shadow /= float(sampleCount);
    
    // keep the shadow at 0.0 when outside the far_plane region of the light's frustum.
    if(projCoords.z > 1.0)
        shadow = 0.0;
        
    return shadow;
}

// Perturb normal with normal map
vec3 perturbNormal(vec3 normal, vec3 normalSample) {
    vec3 normalTex = normalize(normalSample * 2.0 - 1.0);
    normalTex = mix(vec3(0.0, 0.0, 1.0), normalTex, normalStrength);
    return normalize(normal + normalTex);
}

void main()
{
    // --- Chunk-Aligned Tiling Parameters ---
    const float CHUNK_SIZE = 64.0;       // Chunk size in world units
    const float ROCK_TILE_SCALE = 8.0;   // Rock texture repeats every 8 units (64/8 = 8 tiles per chunk)
    const float GRASS_TILE_SCALE = 4.0;  // Grass texture repeats every 4 units (64/4 = 16 tiles per chunk)

    // --- Normalize Interpolated Normal ---
    vec3 normal = normalize(fs_in.Normal);
    
    // --- Blend Factors (Top vs. Sides) ---
    float upFactor = -normal.y; // 1=up, -1=down
    float topBlend = smoothstep(0.7, 0.9, upFactor);
    
    // --- Tri-Planar Blending (Favor Y-Axis) ---
    vec3 blending = abs(normal);
    blending.y *= 4.0;          // Boost top projection influence
    blending = normalize(max(blending, 0.00001));

    // --- Rock Texture Sampling (Chunk-Aligned UVs) ---
    vec2 RockUVX = mod(fs_in.FragPos.zy, CHUNK_SIZE) / ROCK_TILE_SCALE;
    vec2 RockUVY = mod(fs_in.FragPos.xz, CHUNK_SIZE) / ROCK_TILE_SCALE; // Dominant top projection
    vec2 RockUVZ = mod(fs_in.FragPos.xy, CHUNK_SIZE) / ROCK_TILE_SCALE;
    
    vec4 RockX = texture(uRockTex, RockUVX);
    vec4 RockY = texture(uRockTex, RockUVY);
    vec4 RockZ = texture(uRockTex, RockUVZ);
    vec4 RockColor = RockX * blending.x + RockY * blending.y + RockZ * blending.z;
    
    // --- Rock Normal Mapping ---
    vec3 RockNormalX = texture(uRockNormal, RockUVX).xyz;
    vec3 RockNormalY = texture(uRockNormal, RockUVY).xyz;
    vec3 RockNormalZ = texture(uRockNormal, RockUVZ).xyz;
    vec3 RockNormalTex = RockNormalX * blending.x + RockNormalY * blending.y + RockNormalZ * blending.z;

    // --- Grass Texture Sampling (Chunk-Aligned UVs) ---
    vec2 grassUV = mod(fs_in.FragPos.xz, CHUNK_SIZE) / GRASS_TILE_SCALE;
    vec4 grassColor = texture(uGrassTex, grassUV);
    vec3 grassNormalTex = texture(uGrassNormal, grassUV).xyz;

    // --- Final Blending (Rock vs. Grass) ---
    vec4 finalColor = mix(RockColor, grassColor, topBlend);
    vec3 finalNormalTex = mix(RockNormalTex, grassNormalTex, topBlend);
    vec3 finalNormal = perturbNormal(normal, finalNormalTex);

    // --- Lighting Calculation ---
    float diffuse = max(dot(finalNormal, lightDir), 0.0);
    float ambient = 0.2;
    float lighting = ambient + (1.0 - ambient) * diffuse;

    float shadow = ShadowCalculation(fs_in.FragPosLightSpace);                      
    
    FragColor = finalColor * lighting * (1.0 - shadow * 0.5);  
}