#version 140

uniform sampler2D tex;
uniform vec4 global_color;
uniform float time;

in vec2 v_tex_coords;

out vec4 f_color;

const int gaussRadius = 11;
const float gaussFilter[gaussRadius] = float[gaussRadius](
	0.0402, 0.0623, 0.0877, 0.1120, 0.1297, 0.1362, 0.1297, 0.1120, 0.0877, 0.0623, 0.0402
);

void main() {

    vec2 uShift = vec2(0.001 * sin(time), 0.001 * cos(time));

	vec2 texCoord = v_tex_coords - float(int(gaussRadius/2)) * uShift;
	vec3 color = vec3(0.0, 0.0, 0.0);
	for (int i = 0; i < gaussRadius; ++i) {
		color += gaussFilter[i] * texture2D(tex, texCoord).xyz;
		texCoord += uShift;
	}
	f_color = vec4(color,1.0);
/*
    float fx = sin(time) * 0.1;
    float fy = cos(time) * 0.1;

    vec2 coords = vec2(v_tex_coords.x, v_tex_coords.y);
    coords.x += fy;
    coords.y += fx;

    vec4 color = texture(tex, coords) * global_color;

    f_color = color;

    f_color = vec4(
        fx * color.r + fy * color.b,
        fy * color.g + fx * color.r,
        fx * color.b + fy * color.g,
        color.a
    );

*/
    //f_color = f_color + vec4(1.0, 0.0, 0.0, 0.5);
}
