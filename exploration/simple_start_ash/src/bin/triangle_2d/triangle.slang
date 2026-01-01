
/*
Vertex was:
layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 color;


layout (location = 0) out vec4 o_color;
void main() {
    o_color = color;
    gl_Position = pos;
*/
struct VertexIn
{
    [[vk::location(0)]] float4 pos;
    [[vk::location(1)]] float4 color;
};

struct VertexOut
{
	float4 pos : SV_POSITION;
	[[vk::location(0)]] float4  color;
};

[shader("vertex")]
VertexOut vertexMain(VertexIn input)
{
	VertexOut output;
	output.color = input.color;
	output.pos = input.pos;
	return output;
}

/*

Fragment was:
layout (location = 0) in vec4 o_color;
layout (location = 0) out vec4 uFragColor;

void main() {
    uFragColor = o_color;
}

*/
[shader("fragment")]
float4 fragmentMain(VertexOut input)
{
    return input.color;
}
