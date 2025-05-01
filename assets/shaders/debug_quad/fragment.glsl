#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    float texColor = texture(screenTexture, TexCoords).r;
    
    // Map grayscale value to hue (0-1 corresponds to 0-360 degrees in HSV)
    // Using 0.66 (240 degrees) to 0.0 (0 degrees) for a nice blue-to-red spectrum
    float hue = 0.66 * (1.0 - texColor);
    vec3 hsvColor = vec3(hue, 1.0, 1.0); // Full saturation and value
    
    FragColor = vec4(hsv2rgb(hsvColor), 1.0);
}