inline uint extract_8bit_value(uint bit_shift, uint package) {
    uint mask = 255 << bit_shift;
    uint result = (package >> bit_shift) & mask;

    return result;
}

inline uint extract_16bit_value(uint bit_shift, uint package) {
    uint mask = 65535 << bit_shift;
    uint result = (package >> bit_shift) & mask;

    return result;
}

typedef uint InSceneBboxRef;
typedef uint PietGlyphRef;
typedef uint PietCircleRef;
typedef uint PietItemRef;

struct InSceneBboxPacked {
    uint1 x_lims;
    uint1 y_lims;
};

inline InSceneBboxPacked InSceneBbox_read(ByteAddressBuffer buf, InSceneBboxRef ref) {
    InSceneBboxPacked result;

    uint1 x_lims = buf.Load(ref);
    result.x_lims = x_lims;

    uint1 y_lims = buf.Load(ref + 4);
    result.y_lims = y_lims;

    return result;
}

inline uint1 InSceneBbox_x_lims(ByteAddressBuffer buf, InSceneBboxRef ref) {
    uint1 x_lims = buf.Load(ref);
    return x_lims;
}

inline uint1 InSceneBbox_y_lims(ByteAddressBuffer buf, InSceneBboxRef ref) {
    uint1 y_lims = buf.Load(ref + 4);
    return y_lims;
}

struct PietGlyphPacked {
    uint tag;
    uint2 in_atlas_bbox;
    uint1 color;
};

inline PietGlyphPacked PietGlyph_read(ByteAddressBuffer buf, PietGlyphRef ref) {
    PietGlyphPacked result;

    uint2 in_atlas_bbox = buf.Load2(ref + 4);
    result.in_atlas_bbox = in_atlas_bbox;

    uint1 color = buf.Load(ref + 12);
    result.color = color;

    return result;
}

inline uint2 PietGlyph_in_atlas_bbox(ByteAddressBuffer buf, PietGlyphRef ref) {
    uint2 in_atlas_bbox = buf.Load2(ref + 4);
    return in_atlas_bbox;
}

inline uint1 PietGlyph_color(ByteAddressBuffer buf, PietGlyphRef ref) {
    uint1 color = buf.Load(ref + 12);
    return color;
}

struct PietCirclePacked {
    uint tag;
    uint radius;
    uint1 color;
};

inline PietCirclePacked PietCircle_read(ByteAddressBuffer buf, PietCircleRef ref) {
    PietCirclePacked result;

    uint radius = buf.Load(ref + 4);
    result.radius = radius;

    uint1 color = buf.Load(ref + 8);
    result.color = color;

    return result;
}

inline uint PietCircle_radius(ByteAddressBuffer buf, PietCircleRef ref) {
    uint radius = buf.Load(ref + 4);
    return radius;
}

inline uint1 PietCircle_color(ByteAddressBuffer buf, PietCircleRef ref) {
    uint1 color = buf.Load(ref + 8);
    return color;
}

inline uint PietCircle_unpack_radius(uint radius) {
    uint result;

    result = extract_16bit_value(16, radius);
    return result;
}

inline uint4 PietCircle_unpack_color(uint1 color) {
    uint4 result;

    result[0] = extract_8bit_value(24, color);
    result[1] = extract_8bit_value(16, color);
    result[2] = extract_8bit_value(8, color);
    result[3] = extract_8bit_value(0, color);
    return result;
}

struct PietItem {
    uint tag;
    uint body[4];
};
inline uint PietItem_tag(ByteAddressBuffer buf, PietItemRef ref) {
    uint result = buf.Load(ref);
    return result;
}

#define PietItem_Circle 0
#define PietItem_Glyph 1
