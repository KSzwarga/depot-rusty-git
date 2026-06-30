#[repr(C, packed)]
pub struct CacheHeader {
    pub signature: u32,
    pub version: i32,
    pub nr_entries: i32,
    pub sha1: [u8; 20],
}


