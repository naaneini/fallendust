// filepath: fallendust/assets/shaders/debug_quad/fragment.glsl
#version 330 core

in vec2 TexCoords;
out vec4 FragColor;

uniform sampler2D screenTexture;

void main() {
    // Sample the texture
    FragColor = texture(screenTexture, TexCoords);
}