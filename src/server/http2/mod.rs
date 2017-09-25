pub use self::io::{read_frame, write_frame, FrameError};
pub use self::frames::{Frame, PriorityInfo, StreamId};

/**
 * HTTP/2 error codes
 */

#[allow(dead_code)]
pub mod error_codes {
    pub const NO_ERROR: u32 = 0x0000;
    pub const PROTOCOL_ERROR: u32 = 0x0001;
    pub const INTERNAL_ERROR: u32 = 0x0002;
    pub const FLOW_CONTROL_ERROR: u32 = 0x0003;
    pub const SETTINGS_TIMEOUT: u32 = 0x0004;
    pub const STREAM_CLOSED: u32 = 0x0005;
    pub const FRAME_SIZE_ERROR: u32 = 0x0006;
    pub const REFUSED_STREAM: u32 = 0x0007;
    pub const CANCEL: u32 = 0x0008;
    pub const COMPRESSION_ERROR: u32 = 0x0009;
    pub const CONNECT_ERROR: u32 = 0x000a;
    pub const ENHANCE_YOUR_CALM: u32 = 0x000b;
    pub const INADEQUATE_SECURITY: u32 = 0x000c;
    pub const HTTP_1_1_REQUIRED: u32 = 0x000d;
}

mod frames;
mod io;