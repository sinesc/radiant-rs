#version 140

uniform sampler2D tex;
uniform vec4 global_color;
uniform float time;
uniform bool horizontal;
in vec2 v_tex_coords;
out vec4 f_color;

//const float weight[13] = float[] (0.45, 0.15, 0.125, 0.1, 0.09, 0.08, 0.07, 0.006, 0.005, 0.004, 0.003, 0.002, 0.001);
//const float weight[14] = float[] (0.38, 0.32, 0.25, 0.125, 0.1, 0.09, 0.08, 0.07, 0.006, 0.005, 0.0045, 0.004, 0.0035, 0.003);
const float weight[13] = float[] (0.23, 0.19, 0.10, 0.085, 0.08, 0.075, 0.07, 0.006, 0.005, 0.0045, 0.004, 0.0035, 0.003);

void main()
{
    vec2 tex_offset = 1.0 / textureSize(tex, 0); // gets size of single texel
    const float size = 3.1415;
    vec3 result = texture(tex, v_tex_coords).rgb * weight[0]; // current fragment's contribution

    if (horizontal) {
        for(int i = 1; i < 13; ++i)
        {
            result += texture(tex, v_tex_coords + vec2(tex_offset.x * i * size, 0.0)).rgb * weight[i];
            result += texture(tex, v_tex_coords - vec2(tex_offset.x * i * size, 0.0)).rgb * weight[i];
        }
    } else {
        for(int i = 1; i < 13; ++i)
        {
            result += texture(tex, v_tex_coords + vec2(0.0, tex_offset.y * i * size)).rgb * weight[i];
            result += texture(tex, v_tex_coords - vec2(0.0, tex_offset.y * i * size)).rgb * weight[i];
        }
    }

    f_color = vec4(result, 1.0);
}
