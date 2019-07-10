#define BLOCK_SIZE {{TILE_SIZE_SQUARED}}

cbuffer Constants
{
	uint num_circles;
};

struct Circle
{
    float radius;
    float2 center;
    float4 color;
    float pad;
};
StructuredBuffer <Circle> circle_buffer;

RWTexture2D<float4> canvas;

float circle_shader(uint2 pixel_pos, uint2 center_pos, float radius, float err) {
    float d = distance(pixel_pos, center_pos);

    if (d > (1.0f + err)*radius) {
        return 0.0f;
    }

    if (d < (1.0f - err)*radius) {
        return 1.0f;
    }

    // linear interpolation
    return 1.0f - (d - (1.0f - err)*radius)/(2*err*radius);
}

float4 calculate_pixel_color_due_to_circle(uint2 pixel_pos, Circle circle) {
    float position_based_alpha = circle_shader(pixel_pos, circle.center, circle.radius, 2.0f);

    float4 pixel_color = {circle.color[0], circle.color[1], circle.color[2], circle.color[3]*position_based_alpha};
    return pixel_color;
}

float4 blend_pd_over(float4 bg, float4 fg) {
    float fga = fg[3];
    float bga = bg[3];
    float fgax = 1.0f - fga;
    float bgax = 1.0f - bga;
    float x = bga*fgax;

    float denominator = fga + bga*fgax;

    float r = fg[0]*fga + bg[0]*x;
    float g = fg[1]*fga + bg[1]*x;
    float b = fg[2]*fga + bg[2]*x;
    float a = fga*fga + bga*x;

    float4 result = {r, g, b, a};
    return result;
}

[numthreads({{TILE_SIZE}}, {{TILE_SIZE}}, 1)]
void CSMain(uint3 DTid : SV_DispatchThreadID) {
    float4 bg = {0.0f, 0.0f, 0.0f, 0.0f};
    float4 fg = {0.0f, 0.0f, 0.0f, 0.0f};

    uint2 pixel_pos = DTid.xy;

    for (uint i = 0; i < num_circles; i++) {
        Circle c = circle_buffer.Load(i);

        float4 fg = calculate_pixel_color_due_to_circle(pixel_pos, c);
        bg = blend_pd_over(bg, fg);
    }

    canvas[DTid.xy] = bg;
}

float4 VSMain(float4 position: POSITION) : SV_Position
{
    return position;
}

float4 PSMain(float4 position: SV_Position) : SV_TARGET
{
    uint2 pos = position.xy;
    return canvas[pos.xy];
}
