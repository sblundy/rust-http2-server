pub type StreamId = u32;

#[derive(Debug)]
pub struct PriorityInfo {
    pub exclusive: bool,
    pub stream_dependency_id: StreamId,
    pub weight: u8
}

impl PriorityInfo {
    pub fn new(raw_stream_dependency: u32, weight: u8) -> PriorityInfo {
        let exclusive = (raw_stream_dependency & 0x80000000) != 0;
        let stream_dependency_id = raw_stream_dependency & 0x7fffffff;
        PriorityInfo {
            exclusive,
            stream_dependency_id,
            weight,
        }
    }

    pub fn stream_dependency_field(&self) -> u32 {
        self.stream_dependency_id | if self.exclusive { 0x80000000 } else { 0 }
    }
}

#[derive(Debug)]
pub enum Frame {
    Data {
        stream_id: StreamId/* MUST be associated with a stream. 0x0 invalid*/,
        is_end_stream: bool,
        data: Vec<u8>
    },
    Headers {
        stream_id: StreamId, //MUST be associated with a stream. 0x0 invalid
        priority_info: Option<PriorityInfo>,
        is_end_of_stream: bool,
        is_end_of_headers: bool,
        header_block_fragment: Vec<u8>
    },
    Priority {
        stream_id: StreamId /*MUST be associated with a stream. 0x0 invalid*/,
        priority_info: PriorityInfo
    },
    RstStream {
        stream_id: StreamId /*MUST be associated with a stream. 0x0 invalid*/,
        error_code: u32
    },
    Settings {
        // MUST NOT be associate with a stream. 0x0 only valid stream ID
        //must be 0. Protocol error if any other value
        is_ack: bool,
        values: Vec<(u16, u32)>
    },
    PushPromise {
        stream_id: StreamId, /*MUST be associated with a stream. 0x0 invalid*/
        promised_stream_id: u32,
        is_end_of_headers: bool,
        header_block_fragment: Vec<u8>
    },
    Ping {
        // MUST NOT be associate with a stream. 0x0 only valid stream ID
        is_ack: bool,
        opaque_data: [u8; 8]
    },
    GoAway {
        // MUST NOT be associate with a stream. 0x0 only valid stream ID
        last_stream_id: u32,
        error_code: u32,
        additional_debug_data: Vec<u8>
    },
    WindowUpdate {
        stream_id: Option<StreamId>,
        size_increment: u32
    },
    Continuation {
        stream_id: StreamId,  //MUST be associated with a stream. 0x0 invalid
        is_end_headers: bool,
        header_block_fragment: Vec<u8>
    }
}
