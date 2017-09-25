pub const END_STREAM: u8 = 0x1; //Used by DATA, HEADERS
pub const ACK: u8 = 0x1; //Used by SETTINGS, PING

pub const END_HEADERS: u8 = 0x4; //Used by HEADERS, PUSH_PROMISE, CONTINUATION

pub const PADDED: u8 = 0x8; //Used by DATA, HEADERS, PUSH_PROMISE

pub const PRIORITY: u8 = 0x20; //Used by HEADERS

pub fn is_set(flags: u8, flag: u8) -> bool {
    (flags & flag) != 0
}