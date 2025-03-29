#version 330 core
in vec2 TexCoord;
in vec4 a_color;
in vec4 a_type; 
layout(location = 0) out vec4 FragColor;

uniform vec2 u_light_position;
uniform float u_light_radius;

uniform sampler2D u_texture;
uniform ivec2 u_resolution;

void main()
{
    float frag_to_light = distance(gl_FragCoord.xy, u_light_position);
    if (length(frag_to_light) > u_light_radius) {
        discard; 
    } 
    float f = mix(1.0, 0.7, step(0.8, frag_to_light / u_light_radius));
    // float f = smoothstep(1.5, 0.8, frag_to_light / u_light_radius); 
    FragColor = vec4(f, f, f, 1.0);
}