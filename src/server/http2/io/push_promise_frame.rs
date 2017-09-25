use std::io::{Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use super::{read_padded_body, write_frame_header, FRAME_TYPE_PUSH_PROMISE, FrameError};
use server::http2::frames::Frame;
use super::flags;

pub fn read_push_promise_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if stream_id == 0x0 {
        return Err(FrameError::StreamIdRequired());
    }
    let pad_len = if flags::is_set(flags, flags::PADDED) {
        try_read!(reader.read_u8(), "Reading padding len")
    } else { 0 as u8 };

    let promised_stream_id = try_read!(reader.read_u32::<NetworkEndian>(), "Reading promised stream id");
    let fragment_len = frame_len - if flags::is_set(flags, flags::PADDED) { 1 } else { 0 } - 4;
    let header_block_fragment = try_read!(read_padded_body(reader, fragment_len, pad_len), "read_push_promise_frame:header_block_fragment");

    Ok(Frame::PushPromise {
        stream_id,
        promised_stream_id,
        is_end_of_headers: flags::is_set(flags, flags::END_HEADERS),
        header_block_fragment,
    })
}

pub fn write_push_promise_frame(writer: &mut Write, stream_id: u32, is_end_of_headers: bool, promised_stream_id: u32, header_block_fragment: Vec<u8>) -> Result<(), FrameError> {
    let flags = if is_end_of_headers { flags::END_HEADERS } else { 0x00 };
    write_frame_header(writer, (4 + header_block_fragment.len()) as u32, FRAME_TYPE_PUSH_PROMISE, flags, stream_id)?;
    try_write!(writer.write_u32::<NetworkEndian>(promised_stream_id), "write_push_promise_frame:promised stream id");
    try_write!(writer.write(header_block_fragment.as_ref()), "write_push_promise_frame:header fragment");
    Ok(())
}
