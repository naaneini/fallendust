#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;

uniform mat4 uMVP;

out vec3 vNormal;
out vec3 vWorldPos; // Pass world position for texture mapping

void main()
{
    gl_Position = uMVP * vec4(aPos, 1.0);
    vNormal = normalize(aNormal);
    vWorldPos = aPos; // Assuming model is not transformed
}