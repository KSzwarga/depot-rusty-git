#[repr(C)]
#[derive(Debug)]
pub struct CacheEntry {
    pub created: u128,
    pub modified: u128,
    pub size: u64,
    pub sha1: [u8; 20],
    pub namelen: u16,
    pub name: String,
}