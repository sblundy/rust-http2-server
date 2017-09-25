use std::io::{Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use super::FrameError;
use super::{write_frame_header, FRAME_TYPE_WINDOW_UPDATE};
use server::http2::frames::Frame;

pub fn read_window_update_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if frame_len != 4 {
        return Err(FrameError::InvalidFrameSize());
    }
    let size_increment = try_read!(reader.read_u32::<NetworkEndian>(), "read_window_update_frame: window_size_increment");

    Ok(Frame::WindowUpdate {
        stream_id: if stream_id == 0x0 { None } else { Some(stream_id) },
        size_increment
    })
}

pub fn write_window_update_frame(writer: &mut Write, stream_id: Option<u32>, size_increment: u32) -> Result<(), FrameError> {
    write_frame_header(writer, 4, FRAME_TYPE_WINDOW_UPDATE, 0x0, stream_id.unwrap_or(0x0))?;
    try_write!(writer.write_u32::<NetworkEndian>(size_increment), "write_window_update_frame: size_increment");
    Ok(())
}
