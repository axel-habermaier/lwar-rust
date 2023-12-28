struct Input
{
	// float2 texCoords	: TEXCOORD0;
	// float4 color		: COLOR0;
};

// Texture2D Tex : register(t0);

// SamplerState TexSampler : register(s0)
// {
// 	Filter = MIN_MAG_MIP_LINEAR;
// 	AddressU = Wrap;
// 	AddressV = Wrap;
// };

float4 main(Input input) : SV_Target
{
	// return Tex.Sample(TexSampler, input.TexCoords) * input.Color;
    return float4(1, 0, 0, 1);
}