use std::io::{Read, Write};
use server::http2::frames::Frame;
use byteorder::{ReadBytesExt};
use super::{read_padded_body, write_frame_header, FRAME_TYPE_DATA, FrameError};
use super::flags;

pub fn read_data_frame(mut frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if stream_id == 0x0 {
        return Err(FrameError::StreamIdRequired());
    }

    let is_padded = flags::is_set(flags, flags::PADDED);
    let is_end_stream = flags::is_set(flags, flags::END_STREAM);

    let pad_length = if !is_padded { 0 } else {
        frame_len = frame_len - 1;
        try_read!(reader.read_u8(), "read_data_frame:pad length")
    };

    let data = try_read!(read_padded_body(reader, frame_len, pad_length), "read_data_frame:data");
    Ok(Frame::Data { stream_id, data, is_end_stream })
}

pub fn write_data_frame(writer: &mut Write, stream_id: u32, payload: &Vec<u8>, is_terminated: bool) -> Result<(), FrameError> {
    write_frame_header(writer, payload.len() as u32, FRAME_TYPE_DATA, if is_terminated { flags::END_STREAM } else { 0x0 }, stream_id)?;
    try_write!(writer.write_all(payload.as_ref()), "Writing payload");

    Ok(())
}