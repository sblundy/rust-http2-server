use std::io::{Read, Write};
use super::FrameError;
use super::{write_frame_header, FRAME_TYPE_CONTINUATION};
use server::http2::frames::Frame;
use super::flags;

pub fn read_continuation_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if stream_id == 0x0 {
        return Err(FrameError::StreamIdRequired());
    }
    let mut header_block_fragment = Vec::with_capacity(frame_len as usize);
    try_read!(reader.take(frame_len as u64).read_to_end(&mut header_block_fragment),     "read_continuation_frame:header_block_fragment");
    Ok(Frame::Continuation {
        stream_id,
        is_end_headers: flags::is_set(flags, flags::END_HEADERS),
        header_block_fragment
    })
}

pub fn write_continuation_frame(writer: &mut Write, stream_id: u32, is_end_headers: bool, header_block_fragment: Vec<u8>) -> Result<(), FrameError> {
    write_frame_header(writer, header_block_fragment.len() as u32, FRAME_TYPE_CONTINUATION, if is_end_headers { flags::END_HEADERS} else { 0x0}, stream_id)?;
    try_write!(writer.write_all(header_block_fragment.as_ref()), "write_continuation_frame: header_block_fragment");
    Ok(())
}
