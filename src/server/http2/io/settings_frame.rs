use std::io::{Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use super::{write_frame_header, FRAME_TYPE_SETTINGS, FrameError};
use server::http2::frames::Frame;
use super::flags;

pub fn read_settings_frame(frame_len: u32, stream_id: u32, flags: u8, reader: &mut Read) -> Result<Frame, FrameError> {
    if frame_len % 6 != 0 {
        return Err(FrameError::InvalidFrameSize());
    }
    if stream_id != 0x0 {
        return Err(FrameError::StreamIdForbidden());
    }
    let is_ack = flags::is_set(flags, flags::ACK);
    let mut expected_length = frame_len / 6;
    let mut values = Vec::new();

    loop {
        if expected_length == 0 {
            break;
        }
        let id = try_read!(reader.read_u16::<NetworkEndian>(), "read_settings_frame.id");
        let value = try_read!(reader.read_u32::<NetworkEndian>(), "read_settings_frame.value");
        values.push((id, value));
        expected_length = expected_length - 1;
    }

    Ok(Frame::Settings {
        is_ack,
        values,
    })
}

pub fn write_settings_frame(writer: &mut Write, is_ack: bool, values: Vec<(u16, u32)>) -> Result<(), FrameError> {
    write_frame_header(writer, (values.len() * 6) as u32, FRAME_TYPE_SETTINGS, if is_ack { flags::ACK } else { 0x0 }, 0x0)?;
    for value in values {
        try_write!(writer.write_u16::<NetworkEndian>(value.0), "write_settings_frame:setting.0");
        try_write!(writer.write_u32::<NetworkEndian>(value.1), "write_settings_frame:setting.1");
    }
    Ok(())
}
