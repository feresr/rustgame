#version 330 core

in vec2 TexCoord;
uniform sampler2D u_texture;
uniform vec2 u_lightPosition;

layout(location = 0) out vec4 FragColor;
uniform ivec2 u_resolution;

// Draws the ouline of the tiles (only the ones facing the ligh source) 
// This shader takes an complete outline of the map and discard the edges that point away from the light source
void main() {
    vec4 pre_normal = (texture(u_texture, TexCoord));
    if (pre_normal.a <= 0.0) {
        discard;
    }
    
    // map normal color from 0 to 1 to -1 to 1
    vec3 normal = normalize(pre_normal.xyz * 2.0 - vec3(1.0)); 
    vec2 pixelCoor = vec2(gl_FragCoord.x, gl_FragCoord.y);
    vec2 lightPos = vec2(u_lightPosition.x, u_resolution.y - u_lightPosition.y);
    
    vec3 lightDir = normalize(vec3(lightPos - pixelCoor, 0.0));

    float dot =  dot(lightDir.xy, normal.xy);

    // Discard fragments that do not point towards the light source (with some tolerance)
    if (dot <= -0.25) {
        discard;
    }
}
