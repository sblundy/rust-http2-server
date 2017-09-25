use std::io::{Read, Write};
use super::{write_frame_header, FRAME_TYPE_PING};
use server::http2::frames::Frame;
use super::FrameError;
use super::flags;

pub fn read_ping_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if frame_len != 8 {
        return Err(FrameError::InvalidFrameSize());
    }
    if stream_id != 0x0 {
        return Err(FrameError::StreamIdForbidden())
    }
    let mut opaque_data = [0x00 as u8; 8];
    match reader.read_exact(&mut opaque_data) {
        Ok(()) => {},
        Err(e) => return Err(FrameError::ReadFailed("opaque data", e))
    }

    Ok(Frame::Ping {
        is_ack: flags::is_set(flags, flags::ACK),
        opaque_data
    })
}

pub fn write_ping_frame(writer: &mut Write, is_ack: bool, opaque_data: [u8; 8]) -> Result<(), FrameError> {
    write_frame_header(writer, 8, FRAME_TYPE_PING, if is_ack { flags::ACK } else {0}, 0x0)?;
    try_write!(writer.write(&opaque_data), "write_ping_frame: opaque data");
    Ok(())
}
