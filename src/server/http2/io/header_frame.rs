use std::io::{Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use super::{read_padded_body, write_frame_header, FRAME_TYPE_HEADERS, FrameError};
use server::http2::frames::{Frame, PriorityInfo};
use super::flags;

pub fn read_header_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if stream_id == 0x0 {
        return Err(FrameError::StreamIdRequired());
    }

    let mut fragment_len = frame_len as u32;
    let mut padding_len = 0 as u8;

    if flags::is_set(flags, flags::PADDED) {
        fragment_len = fragment_len - 1;
        padding_len = try_read!(reader.read_u8(), "read_header_frame.padding length");
    }

    let priority_info = if flags::is_set(flags, flags::PRIORITY) {
        let raw_steam_dependency = match reader.read_u32::<NetworkEndian>() {
            Ok(value) => value,
            Err(e) => return Err(FrameError::ReadFailed("Reading stream dependency field", e))
        };
        let weight = match reader.read_u8() {
            Ok(value) => value,
            Err(e) => return Err(FrameError::ReadFailed("Reading weight field", e))
        };
        Some(PriorityInfo::new(raw_steam_dependency, weight))
    } else { None };


    let fragment = try_read!(read_padded_body(reader, fragment_len, padding_len), "read_header_frame: fragment");

    Ok(Frame::Headers {
        priority_info,
        is_end_of_stream: flags::is_set(flags, flags::END_STREAM),
        is_end_of_headers: flags::is_set(flags, flags::END_HEADERS),
        stream_id,
        header_block_fragment: fragment,
    })
}

pub fn write_headers_frame(writer: &mut Write, stream_id: u32, priority_info: Option<PriorityInfo>, is_end_of_stream: bool,
                           is_end_of_headers: bool, header_block_fragment: Vec<u8>) -> Result<(), FrameError> {
    let mut flags: u8 = 0x00;
    if is_end_of_headers { flags = flags | flags::END_HEADERS }
    if is_end_of_stream { flags = flags | flags::END_STREAM }
    if priority_info.is_some() { flags = flags | flags::PRIORITY }
    write_frame_header(writer, (header_block_fragment.len() + if priority_info.is_some() { 5 } else { 0 }) as u32,
                       FRAME_TYPE_HEADERS, flags, stream_id)?;

    if let Some(priority) = priority_info {
        try_write!(writer.write_u32::<NetworkEndian>(priority.stream_dependency_field()), "write_headers_frame: stream dependency");
        try_write!(writer.write_u8(priority.weight), "write_headers_frame: weight");
    }

    try_write!(writer.write_all(header_block_fragment.as_ref()), "write_headers_frame: header block fragment");

    Ok(())
}
