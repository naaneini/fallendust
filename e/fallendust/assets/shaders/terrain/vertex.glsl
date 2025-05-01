# Vertex Shader for Terrain Rendering

# This shader transforms the vertex positions and passes data to the fragment shader.

# Vertex Shader Code
#version 330 core

layout(location = 0) in vec3 aPosition; // Vertex position
layout(location = 1) in vec3 aNormal;   // Vertex normal

uniform mat4 uMVP;          // Model-View-Projection matrix
uniform mat4 lightSpaceMatrix; // Light space matrix

out vec3 FragPos;          // Fragment position
out vec3 Normal;           // Fragment normal
out vec4 FragPosLightSpace; // Fragment position in light space

void main()
{
    FragPos = aPosition; // Pass the vertex position to the fragment shader
    Normal = aNormal;    // Pass the vertex normal to the fragment shader

    // Transform the vertex position into clip space
    gl_Position = uMVP * vec4(aPosition, 1.0);

    // Transform the vertex position into light space
    FragPosLightSpace = lightSpaceMatrix * vec4(aPosition, 1.0);
}