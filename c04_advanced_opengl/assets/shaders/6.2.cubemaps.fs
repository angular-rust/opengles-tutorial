#version 330 core
out vec4 FragColor;

in vec3 Normal;
in vec3 Position;

uniform vec3 camera_pos;
uniform samplerCube skybox;

void main()
{
    vec3 I = normalize(Position - camera_pos);
    vec3 R = reflect(I, normalize(Normal));
    FragColor = vec4(texture(skybox, R).rgb, 1.0);
}
