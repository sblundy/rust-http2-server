use std::io::{Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use super::{write_frame_header, FRAME_TYPE_RST_STREAM, FrameError};
use server::http2::frames::Frame;

pub fn read_rst_stream_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if frame_len != 4 {
        Err(FrameError::InvalidFrameSize())
    } else if stream_id == 0x0 {
        return Err(FrameError::StreamIdRequired());
    } else {
        let error_code = try_read!(reader.read_u32::<NetworkEndian>(), "read_rst_stream_frame:error code");
        Ok(Frame::RstStream { stream_id, error_code })
    }
}

pub fn write_rst_stream_frame(writer: &mut Write, stream_id: u32, error_code: u32) -> Result<(), FrameError> {
    write_frame_header(writer, 4, FRAME_TYPE_RST_STREAM, 0x0, stream_id)?;
    try_write!(writer.write_u32::<NetworkEndian>(error_code), "write_rst_stream_frame: error code");
    Ok(())
}
