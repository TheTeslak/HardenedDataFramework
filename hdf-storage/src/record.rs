pub trait RecordData: Copy + Eq {
    fn checksum(&self) -> u32;
}

macro_rules! impl_record_data_for_int {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl RecordData for $ty {
                fn checksum(&self) -> u32 {
                    let bytes = self.to_le_bytes();
                    let mut acc = 0x811C9DC5u32;
                    let mut index = 0;
                    while index < bytes.len() {
                        acc ^= bytes[index] as u32;
                        acc = acc.wrapping_mul(0x0100_0193);
                        index += 1;
                    }
                    acc
                }
            }
        )+
    };
}

impl_record_data_for_int!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl RecordData for bool {
    fn checksum(&self) -> u32 {
        if *self { 0xA5A5_5A5A } else { 0x5A5A_A5A5 }
    }
}

impl<const N: usize> RecordData for [u8; N] {
    fn checksum(&self) -> u32 {
        let mut acc = 0x811C9DC5u32;
        let mut index = 0;
        while index < N {
            acc ^= self[index] as u32;
            acc = acc.wrapping_mul(0x0100_0193);
            index += 1;
        }
        acc
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistentRecord<T> {
    version: u32,
    value: T,
    checksum: u32,
}

impl<T: RecordData> PersistentRecord<T> {
    pub fn new(version: u32, value: T) -> Self {
        let checksum = checksum_for(version, &value);
        Self {
            version,
            value,
            checksum,
        }
    }

    pub fn with_checksum(version: u32, value: T, checksum: u32) -> Self {
        Self {
            version,
            value,
            checksum,
        }
    }

    pub fn version(self) -> u32 {
        self.version
    }

    pub fn value(self) -> T {
        self.value
    }

    pub fn checksum(self) -> u32 {
        self.checksum
    }

    pub fn expected_checksum(self) -> u32 {
        checksum_for(self.version, &self.value)
    }

    pub fn is_valid(self) -> bool {
        self.checksum == self.expected_checksum()
    }
}

fn checksum_for<T: RecordData>(version: u32, value: &T) -> u32 {
    value.checksum() ^ version.rotate_left(13) ^ 0xC3A5_C85C
}
