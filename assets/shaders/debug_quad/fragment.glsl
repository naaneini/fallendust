#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;

void main() {
    float texColor = texture(screenTexture, TexCoords).r;
    
    // Map grayscale value to hue (0-1 corresponds to 0-360 degrees in HSV)
    // Using 0.66 (240 degrees) to 0.0 (0 degrees) for a nice blue-to-red spectrum
    
    FragColor = vec4(0.0, 0.0, texColor, 1.0);
}