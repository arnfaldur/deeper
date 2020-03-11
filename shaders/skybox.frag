#version 330 core
out vec4 FragColor;

in float height;

void main() {

    float y = height + 0.5;
    vec4 color1 = vec4(0.0, 0.1, 0.4, 1.0);
    vec4 color2 = vec4(0.2, 0.0, 0.0, 1.0);
    FragColor = mix(color2, color1, clamp(y, 0.2, 0.8));

}
