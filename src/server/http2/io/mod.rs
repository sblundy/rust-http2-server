use std::io::{Read, Write, Error};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use super::frames::Frame;


const FRAME_TYPE_DATA: u8 = 0x0;
const FRAME_TYPE_HEADERS: u8 = 0x1;
const FRAME_TYPE_PRIORITY: u8 = 0x2;
const FRAME_TYPE_RST_STREAM: u8 = 0x3;
const FRAME_TYPE_SETTINGS: u8 = 0x4;
const FRAME_TYPE_PUSH_PROMISE: u8 = 0x5;
const FRAME_TYPE_PING: u8 = 0x6;
const FRAME_TYPE_GO_AWAY: u8 = 0x7;
const FRAME_TYPE_WINDOW_UPDATE: u8 = 0x8;
const FRAME_TYPE_CONTINUATION: u8 = 0x9;

#[derive(Debug)]
pub enum FrameError {
    ReadFailed(&'static str, Error),
    WriteFailed(&'static str, Error),
    InvalidFrameSize(),
    StreamIdForbidden(),
    StreamIdRequired(),
    Unrecognized(u8)
}

pub type FrameResult = Result<Frame, FrameError>;

macro_rules! try_read {
    ($expr:expr, $msg:expr) => (match $expr {
        Ok(value) => value,
        Err(e) => return Err(FrameError::WriteFailed($msg, e))
    })
}

macro_rules! try_write {
    ($expr:expr, $msg:expr) => (if let Err(e) = $expr {
        return Err(FrameError::WriteFailed($msg, e));
    })
}

pub fn read_frame(reader: &mut Read) -> FrameResult {
    let length: u32 = try_read!(reader.read_u24::<NetworkEndian>(), "read_frame:length");
    let frame_type: u8 = try_read!(reader.read_u8(), "read_frame:type");
    let flags: u8 = try_read!(reader.read_u8(), "read_frame:flags");
    let stream_id: u32 = try_read!(reader.read_u32::<NetworkEndian>(), "read_frame:stream_id");

    match frame_type {
        FRAME_TYPE_DATA => data_frame::read_data_frame(length, stream_id, flags, reader),
        FRAME_TYPE_HEADERS => header_frame::read_header_frame(length, stream_id, flags, reader),
        FRAME_TYPE_PRIORITY => priority_frame::read_priority_frame(length, stream_id, flags, reader),
        FRAME_TYPE_RST_STREAM => rst_stream_frame::read_rst_stream_frame(length, stream_id, flags, reader),
        FRAME_TYPE_SETTINGS => settings_frame::read_settings_frame(length, stream_id, flags, reader), //TODO impl lock semantics from spec?
        FRAME_TYPE_PUSH_PROMISE => push_promise_frame::read_push_promise_frame(length, stream_id, flags, reader),
        FRAME_TYPE_PING => ping_frame::read_ping_frame(length, stream_id, flags, reader),
        FRAME_TYPE_GO_AWAY => go_away_frame::read_go_away_frame(length, stream_id, flags, reader),
        FRAME_TYPE_WINDOW_UPDATE => window_update_frame::read_window_update_frame(length, stream_id, flags, reader),
        FRAME_TYPE_CONTINUATION => continuation_frame::read_continuation_frame(length, stream_id, flags, reader),
        other => return Err(FrameError::Unrecognized(other))
    }
}

pub fn write_frame(writer: &mut Write, frame: Frame) -> Result<(), FrameError> {
    match frame {
        Frame::Data { stream_id, is_end_stream, data } => data_frame::write_data_frame(writer, stream_id, &data, is_end_stream),
        Frame::Headers { stream_id, priority_info, is_end_of_stream, is_end_of_headers, header_block_fragment} =>
            header_frame::write_headers_frame(writer, stream_id, priority_info, is_end_of_stream, is_end_of_headers, header_block_fragment),
        Frame::Priority { stream_id, priority_info } => priority_frame::write_priority_frame(writer, stream_id, priority_info),
        Frame::RstStream { stream_id, error_code } => rst_stream_frame::write_rst_stream_frame(writer, stream_id, error_code),
        Frame::Settings { is_ack, values} => settings_frame::write_settings_frame(writer, is_ack, values),
        Frame::PushPromise { stream_id, is_end_of_headers, promised_stream_id, header_block_fragment } =>
            push_promise_frame::write_push_promise_frame(writer, stream_id, is_end_of_headers, promised_stream_id, header_block_fragment),
        Frame::Ping { is_ack, opaque_data } => ping_frame::write_ping_frame(writer, is_ack, opaque_data),
        Frame::GoAway { last_stream_id, error_code, additional_debug_data } => go_away_frame::write_go_away_frame(writer, last_stream_id, error_code, additional_debug_data),
        Frame::WindowUpdate { stream_id, size_increment} => window_update_frame::write_window_update_frame(writer, stream_id, size_increment),
        Frame::Continuation { stream_id, is_end_headers, header_block_fragment } => continuation_frame::write_continuation_frame(writer, stream_id, is_end_headers, header_block_fragment)
    }
}

mod data_frame;
mod header_frame;
mod priority_frame;
mod rst_stream_frame;
mod settings_frame;
mod push_promise_frame;
mod ping_frame;
mod go_away_frame;
mod window_update_frame;
mod continuation_frame;

mod flags;

#[allow(dead_code)]
pub mod settings {
    pub const HEADER_TABLE_SIZE:u16 = 0x01;
    pub const ENABLE_PUSH:u16 = 0x02;
    pub const MAX_CONCURRENT_STREAMS:u16 = 0x03;
    pub const INITIAL_WINDOW_SIZE:u16 = 0x04;
    pub const MAX_FRAME_SIZE:u16 = 0x05;
    pub const MAX_HEADER_LIST_SIZE:u16 = 0x06;
}

fn write_frame_header(writer: &mut Write, len: u32, frame_type: u8, flags: u8, stream_id: u32) -> Result<(), FrameError> {
    try_write!(writer.write_u24::<NetworkEndian>(len),"write_frame_header.length");
    try_write!(writer.write_u8(frame_type), "write_frame_header.type");
    try_write!(writer.write_u8(flags), "write_frame_header.flags");
    try_write!(writer.write_u32::<NetworkEndian>(stream_id), "write_frame_header.stream id");
    Ok(())
}

fn read_padded_body(reader: &mut Read, fragment_len: u32, padding_len: u8) -> Result<Vec<u8>, Error> {
    let mut fragment = Vec::with_capacity(fragment_len as usize);
    reader.take(fragment_len as u64).read_to_end(&mut fragment)?;

    if padding_len > 0 {
        fragment.truncate((fragment_len - padding_len as u32) as usize);
    }
    
    Ok(fragment)
}

#[cfg(test)]
mod data_frame_tests {
    use std::io::Write;
    use byteorder::{NetworkEndian, WriteBytesExt};
    use std::ops::{Deref, DerefMut};

    use super::{Frame, write_frame, read_frame, FRAME_TYPE_DATA};
    use super::flags;

    #[test]
    fn test_no_padding() {
        let mut frame_bytes = create_dummy_frame(0x0);
        let mut raw_bytes: &[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Data { stream_id: _, data: actual, is_end_stream: _ }) => assert_eq!("testing".as_bytes(), actual.deref()),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_with_padding() {
        let mut frame_bytes = Vec::new();
        let frame_payload = "testing";
        let frame_length: u32 = (frame_payload.len() + 2) as u32;
        frame_bytes.write_u24::<NetworkEndian>(frame_length).unwrap();
        frame_bytes.write_u8(FRAME_TYPE_DATA).unwrap();
        frame_bytes.write_u8(flags::PADDED).unwrap();
        frame_bytes.write_u32::<NetworkEndian>(0x0001).unwrap();
        frame_bytes.write_u8(0x1).unwrap();
        frame_bytes.write(frame_payload.as_bytes()).unwrap();
        frame_bytes.write_u8(0x0).unwrap();
        let mut raw_bytes: &[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Data { stream_id: _, data: actual, is_end_stream: _ }) => assert_eq!(frame_payload.as_bytes(), actual.deref()),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_not_end() {
        let mut frame_bytes = create_dummy_frame(0x0);
        let mut raw_bytes: &[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Data { stream_id: _, data: _, is_end_stream }) => assert_eq!(false, is_end_stream),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_is_end() {
        let mut frame_bytes = create_dummy_frame(flags::END_STREAM);
        let mut raw_bytes: &[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Data { stream_id: _, data: _, is_end_stream }) => assert_eq!(true, is_end_stream),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_write_frame() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::Data { stream_id: 1, data: Vec::from("test"), is_end_stream: false }).unwrap();
        assert_eq!(output, vec![0x0, 0x0, 0x4, FRAME_TYPE_DATA, 0x0, 0x0, 0x0, 0x0, 0x1, 't' as u8, 'e' as u8, 's' as u8, 't' as u8])
    }

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::Data { stream_id: 1, data: Vec::from("test"), is_end_stream: false }).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::Data { stream_id: 1, data: payload, is_end_stream: false }) => assert_eq!("test".as_bytes(), payload.as_slice()),
            _ => assert!(false)
        }
    }

    fn create_dummy_frame(flags: u8) -> Vec<u8> {
        let mut frame_bytes = Vec::new();
        let frame_payload = "testing";
        let frame_length: u32 = (frame_payload.len()) as u32;
        frame_bytes.write_u24::<NetworkEndian>(frame_length).unwrap();
        frame_bytes.write_u8(FRAME_TYPE_DATA).unwrap();
        frame_bytes.write_u8(flags).unwrap();
        frame_bytes.write_u32::<NetworkEndian>(0x0001).unwrap();
        frame_bytes.write(frame_payload.as_bytes()).unwrap();
        return frame_bytes;
    }

}

#[cfg(test)]
mod settings_tests {
    use byteorder::{NetworkEndian, WriteBytesExt};
    use std::ops::{DerefMut};

    use super::{Frame, read_frame, write_frame, FRAME_TYPE_SETTINGS};
    use super::flags;
    use super::settings;

    #[test]
    fn test_empty_frame() {
        let mut frame_bytes = create_dummy_settings_frame(0x0, Vec::new());
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Settings { is_ack: _, values}) => assert_eq!(values.len(), 0),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_empty_ack_frame() {
        let mut frame_bytes = create_dummy_settings_frame(flags::ACK, Vec::new());
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Settings { is_ack, values: _}) => assert_eq!(true, is_ack),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_frame_with_single_setting() {
        let mut frame_bytes = create_dummy_settings_frame(0x0, vec![(settings::HEADER_TABLE_SIZE, 0x0001)]);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Settings { is_ack: _, values}) => assert_eq!(values.len(), 1),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_frame_with_multiple_settings() {
        let mut frame_bytes = create_dummy_settings_frame(0x0, vec![(settings::HEADER_TABLE_SIZE, 0x0001), (settings::ENABLE_PUSH, 0x0000)]);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Settings { is_ack: _, values}) => assert_eq!(values, vec![(settings::HEADER_TABLE_SIZE, 0x0001), (settings::ENABLE_PUSH, 0x0000)]),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::Settings { is_ack: true, values: vec![(settings::HEADER_TABLE_SIZE, 0x0009)]}).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::Settings { is_ack, values}) => {
                assert_eq!(true, is_ack);
                assert_eq!(values, vec![(settings::HEADER_TABLE_SIZE, 0x0009)]);
            },
            _ => assert!(false)
        }
    }

    fn create_dummy_settings_frame(flags: u8, settings: Vec<(u16, u32)>) -> Vec<u8> {
        let mut frame_bytes = Vec::new();
        let frame_length: u32 = (settings.len() * 6) as u32;
        frame_bytes.write_u24::<NetworkEndian>(frame_length).unwrap();
        frame_bytes.write_u8(FRAME_TYPE_SETTINGS).unwrap();
        frame_bytes.write_u8(flags).unwrap();
        frame_bytes.write_u32::<NetworkEndian>(0x0000).unwrap();
        for setting in settings {
            frame_bytes.write_u16::<NetworkEndian>(setting.0).unwrap();
            frame_bytes.write_u32::<NetworkEndian>(setting.1).unwrap();
        }
        return frame_bytes;
    }
}

#[cfg(test)]
mod headers_tests {
    use byteorder::{NetworkEndian, WriteBytesExt};
    use std::ops::{DerefMut};

    use super::{Frame, read_frame, write_frame, FrameError, FRAME_TYPE_HEADERS};
    use server::http2::frames::PriorityInfo;
    use super::flags;

    #[test]
    fn test_empty_frame() {
        let mut frame_bytes = create_dummy_headers_frame(0x0, Vec::new(), 0x0, None);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Headers { priority_info: _, is_end_of_stream: _, is_end_of_headers: _, stream_id: _, header_block_fragment}) => assert_eq!(header_block_fragment.len(), 0),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_frame_random_data() {
        let mut frame_bytes = create_dummy_headers_frame(0x0, vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9], 0x0, None);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Headers { priority_info: _, is_end_of_stream: _, is_end_of_headers: _, stream_id: _, header_block_fragment}) => assert_eq!(header_block_fragment.len(), 10),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_frame_random_data_with_padding() {
        let fragment = vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9];
        let mut frame_bytes = create_dummy_headers_frame(flags::PADDED, fragment.clone(), 0x2, None);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Headers { priority_info: _, is_end_of_stream: _, is_end_of_headers: _, stream_id: _, header_block_fragment}) => assert_eq!(header_block_fragment, fragment),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_priority_frame() {
        let fragment = vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9];
        let mut frame_bytes = create_dummy_headers_frame(flags::PRIORITY, fragment.clone(), 0x0, Some([0x80, 0x0, 0x0, 0x2, 0x1]));
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::Headers { priority_info: Some(PriorityInfo { exclusive: true, weight: 0x01, stream_dependency_id: 2}), is_end_of_stream: _, is_end_of_headers: _, stream_id: _, header_block_fragment}) => {
                assert_eq!(header_block_fragment, fragment)
            },
            Ok(Frame::Headers { priority_info, is_end_of_stream: _, is_end_of_headers: _, stream_id: _, header_block_fragment:_}) => {
                panic!("Incorrect priority_info: {:?}", priority_info)
            },
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::Headers { priority_info: Some(PriorityInfo { exclusive: true, weight: 0x01, stream_dependency_id: 16}), is_end_of_stream: true, is_end_of_headers: true, stream_id: 0x0001, header_block_fragment: vec![0xFF]}).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::Headers {priority_info: Some(PriorityInfo {exclusive, weight, stream_dependency_id }), is_end_of_stream, is_end_of_headers, stream_id, header_block_fragment }) => {
                assert_eq!(true, exclusive);
                assert_eq!(0x01, weight);
                assert_eq!(16, stream_dependency_id);
                assert_eq!(true, is_end_of_stream);
                assert_eq!(true, is_end_of_headers);
                assert_eq!(0x0001, stream_id);
                assert_eq!(header_block_fragment, vec![0xFF]);
            },
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            _ => assert!(false)
        }
    }

    #[test]
    fn test_round_trip_no_priority() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::Headers { priority_info: None, is_end_of_stream: true, is_end_of_headers: true, stream_id: 0x0001, header_block_fragment: vec![0xFF]}).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::Headers {priority_info: None, is_end_of_stream, is_end_of_headers, stream_id, header_block_fragment }) => {
                assert_eq!(true, is_end_of_stream);
                assert_eq!(true, is_end_of_headers);
                assert_eq!(0x0001, stream_id);
                assert_eq!(header_block_fragment, vec![0xFF]);
            },
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            _ => assert!(false)
        }
    }

    fn create_dummy_headers_frame(flags: u8, mut fragment: Vec<u8>, mut padding_len: u8, priority_section: Option<[u8; 5]>) -> Vec<u8> {
        let mut frame_bytes = Vec::new();
        let frame_length: u32 = fragment.len() as u32 + padding_len as u32 +
            if padding_len > 0 { 1 } else { 0 } +
            if priority_section.is_some() { 5 } else { 0 };
        frame_bytes.write_u24::<NetworkEndian>(frame_length).unwrap();
        frame_bytes.write_u8(FRAME_TYPE_HEADERS).unwrap();
        frame_bytes.write_u8(flags).unwrap();
        frame_bytes.write_u32::<NetworkEndian>(0x0001).unwrap();
        if padding_len > 0 {
            frame_bytes.write_u8(padding_len).unwrap();
        }
        if let Some(priority_bytes) = priority_section {
            frame_bytes.push(priority_bytes[0]);
            frame_bytes.push(priority_bytes[1]);
            frame_bytes.push(priority_bytes[2]);
            frame_bytes.push(priority_bytes[3]);
            frame_bytes.push(priority_bytes[4]);
        }
        frame_bytes.append(&mut fragment);

        while padding_len > 0 {
            frame_bytes.push(0x0);
            padding_len = padding_len - 1;
        }
        return frame_bytes;
    }
}

#[cfg(test)]
mod priority_tests {
    use super::{Frame, FrameError, read_frame, write_frame};
    use server::http2::frames::PriorityInfo;

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::Priority { stream_id: 0x1, priority_info: PriorityInfo { exclusive: true, weight: 0x01, stream_dependency_id: 16} }).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::Priority { stream_id, priority_info: PriorityInfo {exclusive, weight, stream_dependency_id} }) => {
                assert_eq!(true, exclusive);
                assert_eq!(0x01, weight);
                assert_eq!(16, stream_dependency_id);
                assert_eq!(0x1, stream_id);
            },
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            _ => assert!(false)
        }
    }
}

#[cfg(test)]
mod rst_stream_tests {
    use server::http2::error_codes::PROTOCOL_ERROR;
    use super::{Frame, FrameError, read_frame, write_frame};

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::RstStream { stream_id: 0x1, error_code: PROTOCOL_ERROR }).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::RstStream { stream_id, error_code }) => {
                assert_eq!(0x1, stream_id);
                assert_eq!(PROTOCOL_ERROR, error_code);
            },
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            _ => assert!(false)
        }
    }
}

#[cfg(test)]
mod push_promise_tests {
    use byteorder::{NetworkEndian, WriteBytesExt};
    use std::ops::{DerefMut};
    use super::{Frame, FrameError, read_frame, write_frame, FRAME_TYPE_PUSH_PROMISE};
    use super::flags;

    #[test]
    fn test_empty() {
        let mut frame_bytes = create_dummy_push_promise_frame(0x0, 0x1, Vec::new(), 0x0);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::PushPromise { promised_stream_id, is_end_of_headers: _, stream_id: _, header_block_fragment}) => {
                assert_eq!(promised_stream_id, 0x1);
                assert_eq!(header_block_fragment.len(), 0);
            },
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_header_frag() {
        let header_frag = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let mut frame_bytes = create_dummy_push_promise_frame(0x0, 0x1, header_frag.clone(), 0x0);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::PushPromise { promised_stream_id: _, is_end_of_headers: _, stream_id: _, header_block_fragment}) => assert_eq!(header_block_fragment, header_frag),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_header_frag_with_padding() {
        let header_frag = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let mut frame_bytes = create_dummy_push_promise_frame(flags::PADDED, 0x1, header_frag.clone(), 10);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::PushPromise { promised_stream_id: _, is_end_of_headers: _, stream_id: _, header_block_fragment}) => assert_eq!(header_block_fragment, header_frag),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_end_of_headers() {
        let mut frame_bytes = create_dummy_push_promise_frame(flags::END_HEADERS, 0x1, Vec::new(), 0x0);
        let mut raw_bytes:&[u8] = frame_bytes.deref_mut();

        match read_frame(&mut raw_bytes) {
            Ok(Frame::PushPromise { promised_stream_id: _, is_end_of_headers, stream_id: _, header_block_fragment: _}) => assert_eq!(is_end_of_headers, true),
            Ok(frame) => panic!("Success? {:?}", frame),
            Err(e) => panic!("Error:{:?}", e)
        }
    }

    #[test]
    fn test_round_trip() {
        let header_frag = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let mut output = Vec::new();
        write_frame(&mut output, Frame::PushPromise {
            stream_id: 0x1,
            promised_stream_id: 0x2,
            is_end_of_headers: true,
            header_block_fragment: header_frag.clone(),
        }).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::PushPromise { promised_stream_id, is_end_of_headers, stream_id, header_block_fragment }) => {
                assert_eq!(stream_id, 0x1);
                assert_eq!(promised_stream_id, 0x2);
                assert_eq!(is_end_of_headers, true);
                assert_eq!(header_block_fragment, header_frag);
            },
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            _ => assert!(false)
        }
    }

    fn create_dummy_push_promise_frame(flags: u8, promised_stream_id: u32, mut fragment: Vec<u8>, mut padding_len: u8) -> Vec<u8> {
        let mut frame_bytes = Vec::new();
        let frame_length: u32 = if padding_len > 0 { 1 } else { 0 } +
            4 + fragment.len() as u32 + padding_len as u32;
        frame_bytes.write_u24::<NetworkEndian>(frame_length).unwrap();
        frame_bytes.write_u8(FRAME_TYPE_PUSH_PROMISE).unwrap();
        frame_bytes.write_u8(flags).unwrap();
        frame_bytes.write_u32::<NetworkEndian>(0x0001).unwrap();
        if padding_len > 0 {
            frame_bytes.write_u8(padding_len).unwrap();
        }
        frame_bytes.write_u32::<NetworkEndian>(promised_stream_id).unwrap();
        frame_bytes.append(&mut fragment);

        while padding_len > 0 {
            frame_bytes.push(0x0);
            padding_len = padding_len - 1;
        }
        return frame_bytes;
    }
}

#[cfg(test)]
mod ping_frame_tests {
    use super::{Frame, FrameError, read_frame, write_frame};

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::Ping {is_ack: true, opaque_data: [0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]}).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::Ping {
                   is_ack,
                   opaque_data
               }) => {
                assert_eq!(true, is_ack);
                assert_eq!(opaque_data, [0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
            },
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            _ => assert!(false)
        }
    }
}

#[cfg(test)]
mod go_away_frame_tests {
    use super::{Frame, FrameError, read_frame, write_frame};

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::GoAway { last_stream_id: 0x1, error_code: 0x2, additional_debug_data: vec! [0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]}).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::GoAway {
                   last_stream_id,
                   error_code,
                   additional_debug_data
               }) => {
                assert_eq!(0x1, last_stream_id);
                assert_eq!(0x2, error_code);
                assert_eq!(additional_debug_data, vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
            },
            Ok(frame) => panic!("Wrong type:{:?}", frame),
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            Err(e) => panic!("error:{:?}", e),
        }
    }
}

#[cfg(test)]
mod window_update_frame_tests {
    use super::{Frame, FrameError, read_frame, write_frame};

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::WindowUpdate {stream_id: None, size_increment: 0xff}).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::WindowUpdate {
                   stream_id,
                   size_increment
               }) => {
                assert_eq!(None, stream_id);
                assert_eq!(0xff, size_increment);
            },
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            _ => assert!(false)
        }
    }
}

#[cfg(test)]
mod continuation_frame_tests {
    use super::{Frame, FrameError, read_frame, write_frame};

    #[test]
    fn test_round_trip() {
        let mut output = Vec::new();
        write_frame(&mut output, Frame::Continuation {stream_id: 0x1, is_end_headers: true, header_block_fragment: vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8] }).unwrap();
        let frame = read_frame(&mut output.as_slice());
        match frame {
            Ok(Frame::Continuation {
                   stream_id,
                   is_end_headers,
                   header_block_fragment
               }) => {
                assert_eq!(0x1, stream_id);
                assert_eq!(is_end_headers, true);
                assert_eq!(vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8], header_block_fragment);
            },
            Err(FrameError::ReadFailed(context, e)) => panic!("ReadFailed:{}:{}", context, e),
            _ => assert!(false)
        }
    }
}