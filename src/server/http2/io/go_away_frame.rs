use std::io::{Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use super::FrameError;
use super::{write_frame_header, FRAME_TYPE_GO_AWAY};
use server::http2::frames::Frame;

pub fn read_go_away_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if frame_len < 8 {
        return Err(FrameError::InvalidFrameSize());
    } else if stream_id != 0x0 {
        return Err(FrameError::StreamIdForbidden());
    }
    let last_stream_id = try_read!(reader.read_u32::<NetworkEndian>(), "read_go_away_frame: last stream ID");
    let error_code = try_read!(reader.read_u32::<NetworkEndian>(), "read_go_away_frame: error code");
    let mut additional_debug_data:Vec<u8> = Vec::with_capacity((frame_len - 8) as usize);
    try_read!(reader.take((frame_len - 8) as u64).read_to_end(&mut additional_debug_data), "read_go_away_frame:additional_debug_data");

    Ok(Frame::GoAway {
        last_stream_id,
        error_code,
        additional_debug_data
    })
}

pub fn write_go_away_frame(writer: &mut Write, last_stream_id: u32, error_code: u32, additional_debug_data: Vec<u8>) -> Result<(), FrameError> {
    write_frame_header(writer, (8 + additional_debug_data.len()) as u32, FRAME_TYPE_GO_AWAY, 0x0, 0x0)?;
    try_write!(writer.write_u32::<NetworkEndian>(last_stream_id), "write_go_away_frame:last stream ID");
    try_write!(writer.write_u32::<NetworkEndian>(error_code), "write_go_away_frame: error code");
    try_write!(writer.write_all(&additional_debug_data), "write_go_away_frame: debug data");
    Ok(())
}
