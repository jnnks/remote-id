use crate::MAX_ID_BYTE_SIZE;

pub mod decode;
pub mod encode;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum MessageType {
    BasicId = 0,
    Location = 1,
    Auth = 2,
    Selfid = 3,
    System = 4,
    OperatorId = 5,
    MessagePack = 0xF,

    Invalid,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageType::BasicId,
            1 => MessageType::Location,
            2 => MessageType::Auth,
            3 => MessageType::Selfid,
            4 => MessageType::System,
            5 => MessageType::OperatorId,
            0xF => MessageType::MessagePack,

            _ => MessageType::Invalid,
        }
    }
}

fn copy_to_id(slice: &[u8]) -> [u8; 20] {
    let mut buffer = [0u8; MAX_ID_BYTE_SIZE];
    let max = if slice.len() <= MAX_ID_BYTE_SIZE {
        slice.len()
    } else {
        MAX_ID_BYTE_SIZE
    };
    buffer[0..max].copy_from_slice(&slice[0..max]);
    buffer
}

#[macro_export]
macro_rules! get_pattern {
    ($lo:literal, $hi:literal) => {{
        let mut pattern = 1;

        for _ in 0..($hi - $lo) {
            pattern = (pattern << 1) + 1;
        }

        for _ in 0..$lo {
            pattern = (pattern << 1);
        }

        pattern
    }};
}

#[macro_export]
macro_rules! get_bits {
    ($num:expr, $lo:literal..$hi:literal) => {{
        let p = crate::get_pattern!($lo, $hi);
        ($num & p) >> $lo
    }};
}

#[macro_export]
macro_rules! put_bits {
    ($num:expr, $lo:literal..$hi:literal) => {{
        let p = crate::get_pattern!($lo, $hi);
        ($num & p) >> $lo
    }};
}

#[cfg(test)]
mod test {

    #[test]
    fn test_get_bits_trivial() {
        assert_eq!(get_bits!(0b0000, 0..1), 0);
        assert_eq!(get_bits!(0b0000, 0..2), 0);
        assert_eq!(get_bits!(0b0000, 0..3), 0);
        assert_eq!(get_bits!(0b0000, 0..4), 0);
    }

    #[test]
    fn test_get_bits_all_ones() {
        assert_eq!(get_bits!(0b1111, 0..0), 1);
        assert_eq!(get_bits!(0b1111, 0..1), 3);
        assert_eq!(get_bits!(0b1111, 0..2), 7);
        assert_eq!(get_bits!(0b1111, 0..3), 15);

        assert_eq!(get_bits!(0b1111, 1..1), 1);
        assert_eq!(get_bits!(0b1111, 1..2), 3);
        assert_eq!(get_bits!(0b1111, 1..3), 7);

        assert_eq!(get_bits!(0b1111, 2..2), 1);
        assert_eq!(get_bits!(0b1111, 2..3), 3);
    }

    #[test]
    fn test_pattern_from_0() {
        assert_eq!(get_pattern!(0, 0), 1);
        assert_eq!(get_pattern!(0, 1), 3);
        assert_eq!(get_pattern!(0, 2), 7);
        assert_eq!(get_pattern!(0, 3), 15);
        assert_eq!(get_pattern!(0, 4), 31);
        assert_eq!(get_pattern!(0, 5), 63);
        assert_eq!(get_pattern!(0, 6), 127);
        assert_eq!(get_pattern!(0, 7), 255);
    }

    #[test]
    fn test_get_bits_x() {
        let n = 0b0001_1100;
        assert_eq!(get_bits!(n, 2..4), (n & 0b0001_1100) >> 2);

        let n = 34;
        assert_eq!(get_bits!(n, 3..7), (n & 0b1111_1000) >> 3);
    }

    #[test]
    fn test_pattern_one_bit() {
        assert_eq!(get_pattern!(1, 1), 2);
        assert_eq!(get_pattern!(2, 2), 4);
        assert_eq!(get_pattern!(3, 3), 8);
        assert_eq!(get_pattern!(4, 4), 16);
        assert_eq!(get_pattern!(5, 5), 32);
    }

    #[test]
    fn test_pattern_two_bits() {
        assert_eq!(get_pattern!(1, 2), 6);
        assert_eq!(get_pattern!(2, 3), 12);
        assert_eq!(get_pattern!(3, 4), 24);
        assert_eq!(get_pattern!(4, 5), 48);
    }

    #[test]
    fn test_pattern_three_bits() {
        assert_eq!(get_pattern!(1, 3), 14);
        assert_eq!(get_pattern!(2, 4), 28);
        assert_eq!(get_pattern!(3, 5), 56);
        assert_eq!(get_pattern!(4, 6), 112);
    }

    extern crate std;
    #[test]
    fn test_pattern_xxx() {
        let p = get_pattern!(0, 3);
        std::dbg!(p);
    }
}
