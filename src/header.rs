use crate::RomVersion;

pub struct HeaderData {
    map: u32,
    area: u32,
    tile_offset: u32,
    palette_set_table_loc: u32,
    c0_table_loc: u32,
    a1_c0_table_loc: u32,
    c1_table_loc: u32,
    c2_table_loc: u32,
    swap_base: u32,
    palette_change_base: u32,
    area1_swap_base: u32,
    global_tile_set_table_loc: u32,
    gfx_source_base: u32,
    global_meta_tile_set_table_loc: u32,
    global_tile_data_table_loc: u32
}

pub const EU_HEADER_DATA: HeaderData = HeaderData {
    map: 0x11d95c,
    area: 0x0d4828,
    tile_offset: 0x5a23d0,
    palette_set_table_loc: 0xfed88,
    c0_table_loc: 0x107aec,
    a1_c0_table_loc: 0x1077ac,
    c1_table_loc: 0x107b02,
    c2_table_loc: 0x107b18,
    swap_base: 0x107b5c,
    palette_change_base: 0x107940,
    area1_swap_base: 0x107800,
    global_tile_set_table_loc: 0x101bc8,
    gfx_source_base: 0x323fec,
    global_meta_tile_set_table_loc: 0x1027f8,
    global_tile_data_table_loc: 0x1070e4,    
};

pub fn get_header(region: RomVersion) -> &'static HeaderData {
    match region {
        RomVersion::EU => &EU_HEADER_DATA,
        _ => unimplemented!()
    }
}