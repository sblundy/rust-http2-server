use std::io::{Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use super::{FRAME_TYPE_PRIORITY, FrameError, write_frame_header};
use server::http2::frames::{Frame, PriorityInfo};

pub fn read_priority_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if frame_len != 5 {
        Err(FrameError::InvalidFrameSize())
    } else if stream_id == 0x0 {
        return Err(FrameError::StreamIdRequired());
    } else {
        let stream_dependency = try_read!(reader.read_u32::<NetworkEndian>(), "read_priority_frame.stream_dependency");
        let weight = try_read!(reader.read_u8(), "read_priority_frame.weight");
        Ok(Frame::Priority { stream_id, priority_info: PriorityInfo::new(stream_dependency, weight) })
    }
}

pub fn write_priority_frame(writer: &mut Write, stream_id: u32, priority_info: PriorityInfo) -> Result<(), FrameError> {
    write_frame_header(writer, 5, FRAME_TYPE_PRIORITY, 0x0, stream_id)?;
    try_write!(writer.write_u32::<NetworkEndian>(priority_info.stream_dependency_field()), "write_priority_frame:stream dependency field");
    try_write!(writer.write_u8(priority_info.weight), "write_priority_frame:weight");
    Ok(())
}
