use crate::event::{EncodedRecord, EventRecord};

pub fn decode_record(encoded: EncodedRecord) -> Option<EventRecord> {
    EventRecord::decode(encoded)
}

pub fn render_record(encoded: EncodedRecord) -> Option<EventRecord> {
    decode_record(encoded)
}
