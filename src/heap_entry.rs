#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Entry {
    heap_offset: u8,
    bit_offset: u8,
}
