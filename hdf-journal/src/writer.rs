use crate::event::{ENCODED_RECORD_LEN, EncodedRecord, EventRecord, JournalEvent};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendError {
    Full,
}

#[derive(Clone, Copy, Debug)]
pub struct Journal<const N: usize> {
    len: usize,
    next_sequence: u32,
    records: [EncodedRecord; N],
}

impl<const N: usize> Journal<N> {
    pub fn new() -> Self {
        Self {
            len: 0,
            next_sequence: 0,
            records: [[0u8; ENCODED_RECORD_LEN]; N],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn append(&mut self, event: JournalEvent) -> Result<u32, AppendError> {
        if self.len >= N {
            return Err(AppendError::Full);
        }

        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.records[self.len] = EventRecord { sequence, event }.encode();
        self.len += 1;
        Ok(sequence)
    }

    pub fn encoded(&self, index: usize) -> Option<EncodedRecord> {
        if index < self.len {
            Some(self.records[index])
        } else {
            None
        }
    }
}

impl<const N: usize> Default for Journal<N> {
    fn default() -> Self {
        Self::new()
    }
}
