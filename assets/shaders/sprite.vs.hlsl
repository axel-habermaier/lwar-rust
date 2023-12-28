// cbuffer PerFrameConstants : register(b0)
// { 
// 	column_major matrix Projection;
// };

// cbuffer PerObjectConstants : register(b1)
// {
// 	column_major matrix World;
// };

struct Input
{
	float4 position		: POSITION;
	// float2 texCoords	: TEXCOORD0;
	// float4 color		: COLOR0;
};

struct Output
{
	// float2 texCoords	: TEXCOORD0;
	// float4 color		: COLOR0;
	float4 position		: SV_Position;
};

Output main(Input input)
{
	Output output;

	//float4 position = mul(World, input.Position);
	//output.Position = mul(Projection, position);

	//output.Color = input.Color;
	//output.TexCoords = input.TexCoords;

	output.position = input.position;

	return output;
}