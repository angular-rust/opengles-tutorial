#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;

void main()
{
    // simple
    // vec3 col = texture(screenTexture, TexCoords).rgb;

    // inversion
    // vec3 col = vec3(1.0 - texture(screenTexture, TexCoords)).rgb;

    // simple grayscale 
    // vec3 texcolor = texture(screenTexture, TexCoords).rgb;
    // float average = (texcolor.r + texcolor.g + texcolor.b) / 3.0;
    // vec3 col = vec3(average, average, average);

    // nature grayscale
    vec3 texcolor = texture(screenTexture, TexCoords).rgb;
    float average = 0.2126 * texcolor.r + 0.7152 * texcolor.g + 0.0722 * texcolor.b;
    vec3 col = vec3(average, average, average);

    FragColor = vec4(col, 1.0);
}
