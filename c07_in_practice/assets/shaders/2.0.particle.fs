#version 100

precision mediump float;

uniform vec4 u_color;
uniform sampler2D s_texture; 

varying float v_lifetime;

void main() { 
    vec4 texColor;
    texColor = texture2D(s_texture, gl_PointCoord);
    gl_FragColor = vec4(u_color) * texColor;
    gl_FragColor.a *= v_lifetime;
}