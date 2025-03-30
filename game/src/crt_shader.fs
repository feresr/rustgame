#version 330 core
in vec2 TexCoord;
in vec4 a_color;
in vec4 a_type; 

layout(location = 0) out vec4 FragColor;

uniform sampler2D u_color_texture;
uniform sampler2D u_light_texture;

uniform float u_light_radius;

uniform ivec2 u_resolution;

void main()
{
    vec4 color = texture(u_color_texture, TexCoord); 
    vec4 light = texture(u_light_texture, TexCoord); 

    color = color * (light.x * 0.9 + 0.50);

    float crtIntensity = 0.60;  // 0 = max 1 = min
    float crt = (sin(gl_FragCoord.y * 3.14) + 1.0) * 0.5; 
    crt = (crt * (1.0 - (crtIntensity))) + crtIntensity; 
    crt = mix(crt, 1.0, light.x); 
    FragColor = vec4(color.rgb, 1.0) * vec4(crt, crt, crt, 1.0); 
}
