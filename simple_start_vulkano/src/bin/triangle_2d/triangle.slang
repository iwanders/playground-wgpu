#language slang 2026


struct FramePush
{
     float4x4 camera;
};
//const FramePush our_push;
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
    [[vk::location(0)]] float3 position;
    [[vk::location(1)]] float4 normal;
    [[vk::location(2)]] float4 color;
};


struct VertexOut
{
	float4 position : SV_POSITION;
	[[vk::location(0)]] float4  color;
};

[shader("vertex")]
VertexOut vertexMain(VertexIn input, uniform FramePush   push_data)
{
	VertexOut output;
	output.color =  input.color ;
	// output.color[0]  =1.0;
	output.position = mul(push_data.camera,  float4(input.position, 1.0));
	// output.pos = mul(input.pos,  push_data.camera);
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
    return input.color ;
}
