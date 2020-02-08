inline uint extract_8bit_value(uint bit_shift, uint package) {
    uint mask = 255;
    uint result = (package >> bit_shift) & mask;

    return result;
}

inline uint extract_16bit_value(uint bit_shift, uint package) {
    uint mask = 65535;
    uint result = (package >> bit_shift) & mask;

    return result;
}

typedef uint BBoxRef;
typedef uint SRGBColorRef;
typedef uint PietGlyphRef;
typedef uint PietCircleRef;
typedef uint PietItemRef;

struct BBoxPacked {
    uint x0_x1;
    uint y0_y1;
};

inline BBoxPacked BBox_read(ByteAddressBuffer buf, BBoxRef ref) {
    BBoxPacked result;

    uint x0_x1 = buf.Load(ref);
    result.x0_x1 = x0_x1;

    uint y0_y1 = buf.Load(ref + 4);
    result.y0_y1 = y0_y1;

    return result;
}

inline uint BBox_x0_x1(ByteAddressBuffer buf, BBoxRef ref) {
    uint x0_x1 = buf.Load(ref);
    return x0_x1;
}

inline uint BBox_y0_y1(ByteAddressBuffer buf, BBoxRef ref) {
    uint y0_y1 = buf.Load(ref + 4);
    return y0_y1;
}

inline uint BBox_unpack_x0(uint x0_x1) {
    uint result;

    result = extract_16bit_value(0, x0_x1);
    return result;
}

inline uint BBox_unpack_x1(uint x0_x1) {
    uint result;

    result = extract_16bit_value(16, x0_x1);
    return result;
}

inline uint BBox_unpack_y0(uint y0_y1) {
    uint result;

    result = extract_16bit_value(0, y0_y1);
    return result;
}

inline uint BBox_unpack_y1(uint y0_y1) {
    uint result;

    result = extract_16bit_value(16, y0_y1);
    return result;
}

struct BBox {
    uint x0;
    uint x1;
    uint y0;
    uint y1;
};

inline BBox BBox_unpack(BBoxPacked packed_form) {
    BBox result;

    result.x0 = BBox_unpack_x0(packed_form.x0_x1);
    result.x1 = BBox_unpack_x1(packed_form.x0_x1);
    result.y0 = BBox_unpack_y0(packed_form.y0_y1);
    result.y1 = BBox_unpack_y1(packed_form.y0_y1);

    return result;
}

struct SRGBColorPacked {
    uint r_g_b_a;
};

inline SRGBColorPacked SRGBColor_read(ByteAddressBuffer buf, SRGBColorRef ref) {
    SRGBColorPacked result;

    uint r_g_b_a = buf.Load(ref);
    result.r_g_b_a = r_g_b_a;

    return result;
}

inline uint SRGBColor_r_g_b_a(ByteAddressBuffer buf, SRGBColorRef ref) {
    uint r_g_b_a = buf.Load(ref);
    return r_g_b_a;
}

inline uint SRGBColor_unpack_r(uint r_g_b_a) {
    uint result;

    result = extract_8bit_value(0, r_g_b_a);
    return result;
}

inline uint SRGBColor_unpack_g(uint r_g_b_a) {
    uint result;

    result = extract_8bit_value(8, r_g_b_a);
    return result;
}

inline uint SRGBColor_unpack_b(uint r_g_b_a) {
    uint result;

    result = extract_8bit_value(16, r_g_b_a);
    return result;
}

inline uint SRGBColor_unpack_a(uint r_g_b_a) {
    uint result;

    result = extract_8bit_value(24, r_g_b_a);
    return result;
}

struct SRGBColor {
    uint r;
    uint g;
    uint b;
    uint a;
};

inline SRGBColor SRGBColor_unpack(SRGBColorPacked packed_form) {
    SRGBColor result;

    result.r = SRGBColor_unpack_r(packed_form.r_g_b_a);
    result.g = SRGBColor_unpack_g(packed_form.r_g_b_a);
    result.b = SRGBColor_unpack_b(packed_form.r_g_b_a);
    result.a = SRGBColor_unpack_a(packed_form.r_g_b_a);

    return result;
}

struct PietGlyphPacked {
    uint tag;
    BBoxPacked scene_bbox;
    BBoxPacked atlas_bbox;
    SRGBColorPacked color;
};

inline PietGlyphPacked PietGlyph_read(ByteAddressBuffer buf, PietGlyphRef ref) {
    PietGlyphPacked result;

    BBoxPacked scene_bbox = BBox_read(buf, ref + 4);
    result.scene_bbox = scene_bbox;

    BBoxPacked atlas_bbox = BBox_read(buf, ref + 12);
    result.atlas_bbox = atlas_bbox;

    SRGBColorPacked color = SRGBColor_read(buf, ref + 20);
    result.color = color;

    return result;
}

inline BBoxPacked PietGlyph_scene_bbox(ByteAddressBuffer buf, PietGlyphRef ref) {
    BBoxPacked scene_bbox = BBox_read(buf, ref + 4);
    return scene_bbox;
}

inline BBoxPacked PietGlyph_atlas_bbox(ByteAddressBuffer buf, PietGlyphRef ref) {
    BBoxPacked atlas_bbox = BBox_read(buf, ref + 12);
    return atlas_bbox;
}

inline SRGBColorPacked PietGlyph_color(ByteAddressBuffer buf, PietGlyphRef ref) {
    SRGBColorPacked color = SRGBColor_read(buf, ref + 20);
    return color;
}

struct PietGlyph {
    BBox scene_bbox;
    BBox atlas_bbox;
    SRGBColor color;
};

inline PietGlyph PietGlyph_unpack(PietGlyphPacked packed_form) {
    PietGlyph result;

    result.scene_bbox = BBox_unpack(packed_form.scene_bbox);
    result.atlas_bbox = BBox_unpack(packed_form.atlas_bbox);
    result.color = SRGBColor_unpack(packed_form.color);

    return result;
}

struct PietCirclePacked {
    uint tag;
    BBoxPacked scene_bbox;
    SRGBColorPacked color;
};

inline PietCirclePacked PietCircle_read(ByteAddressBuffer buf, PietCircleRef ref) {
    PietCirclePacked result;

    BBoxPacked scene_bbox = BBox_read(buf, ref + 4);
    result.scene_bbox = scene_bbox;

    SRGBColorPacked color = SRGBColor_read(buf, ref + 12);
    result.color = color;

    return result;
}

inline BBoxPacked PietCircle_scene_bbox(ByteAddressBuffer buf, PietCircleRef ref) {
    BBoxPacked scene_bbox = BBox_read(buf, ref + 4);
    return scene_bbox;
}

inline SRGBColorPacked PietCircle_color(ByteAddressBuffer buf, PietCircleRef ref) {
    SRGBColorPacked color = SRGBColor_read(buf, ref + 12);
    return color;
}

struct PietCircle {
    BBox scene_bbox;
    SRGBColor color;
};

inline PietCircle PietCircle_unpack(PietCirclePacked packed_form) {
    PietCircle result;

    result.scene_bbox = BBox_unpack(packed_form.scene_bbox);
    result.color = SRGBColor_unpack(packed_form.color);

    return result;
}

struct PietItem {
    uint tag;
    uint body[5];
};
inline uint PietItem_tag(ByteAddressBuffer buf, PietItemRef ref) {
    uint result = buf.Load(ref);
    return result;
}

inline void PietItem_copy(ByteAddressBuffer src, uint src_ref, RWByteAddressBuffer dst, uint dst_ref) {
    uint4 group0 = src.Load4(src_ref);
    dst.Store4(dst_ref, group0);

    uint2 group1 = src.Load2(src_ref + 16);
    dst.Store2(dst_ref + 16, group1);
}

#define BBOX_SIZE 8
#define SRGBCOLOR_SIZE 4
#define PIET_ITEM_SIZE 24
#define PietItem_Circle 0
#define PietItem_Glyph 1
