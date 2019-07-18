RWTexture2D<float4> canvas : register(u1);

float4 PSMain(float4 position: SV_Position) : SV_TARGET
{
    uint2 pos = position.xy;
    return canvas[pos.xy];
}
