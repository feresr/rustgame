// input texture
#version 330 core
uniform sampler2D u_texture;
uniform vec2 u_texelSize; // (1.0 / textureWidth, 1.0 / textureHeight)

layout(location = 0) out vec4 FragColor;

void main() {
    vec2 uv = gl_FragCoord.xy * u_texelSize;
    vec4 normals = texture(u_texture, uv);
    float c = normals.a;

    float up    = texture(u_texture, uv + vec2(0, u_texelSize.y)).a;
    float down  = texture(u_texture, uv - vec2(0, u_texelSize.y)).a;
    float left  = texture(u_texture, uv - vec2(u_texelSize.x, 0)).a;
    float right = texture(u_texture, uv + vec2(u_texelSize.x, 0)).a;

    bool isEdge = c > 0.5 && (up < 0.5 || down < 0.5 || left < 0.5 || right < 0.5);

    if (isEdge) {
        FragColor = vec4(normals.xyz, 1);
    } else {
        discard;
    }

}
