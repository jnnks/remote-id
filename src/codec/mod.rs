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

pub fn copy_to_id(slice: &[u8]) -> [u8; 20] {
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
macro_rules! bitmask {
    ($lo:literal, $hi:literal) => {{
        let mut pattern = 1;

        for _ in 0..($hi - $lo) {
            pattern = (pattern << 1) + 1;
        }

        for _ in 0..$lo {
            pattern <<= 1;
        }

        pattern
    }};
}

/// Return a subslice or an array of the given bytes:
/// get_bytes!(buf, OFFSET, LEN)
/// ```ignore
/// assert_eq!([2, 3], get_bytes!(&[1, 2, 3], 1, 2))
/// ```  
#[macro_export]
macro_rules! get_bytes {
    ($buf:expr, $off:expr, 1) => {{
        $buf[$off]
    }};

    ($buf:expr, $off:expr, 2) => {{
        [$buf[$off], $buf[$off + 1]]
    }};

    ($buf:expr, $off:expr, 3) => {{
        [$buf[$off], $buf[$off + 1], $buf[$off + 2]]
    }};

    ($buf:expr, $off:expr, 4) => {{
        [$buf[$off], $buf[$off + 1], $buf[$off + 2], $buf[$off + 3]]
    }};

    ($buf:expr, $off:expr, $len:expr) => {{
        &$buf[$off..($off + $len)]
    }};
}

/// Return the given bits from a value
/// ```ignore
/// assert_eq!(get_bits!(0b1111, 2..0), 7)
/// ```  
#[macro_export]
macro_rules! get_bits {
    ($num:expr, $hi:literal..$lo:literal) => {{
        let p = $crate::bitmask!($lo, $hi);
        ($num & p) >> $lo
    }};
}

#[macro_export]
macro_rules! put_bits {
    ($num:expr, $hi:literal..$lo:literal) => {{
        let p = $crate::bitmask!($lo, $hi);
        ($num & p) >> $lo
    }};
}

#[cfg(test)]
mod test {
    use chrono::{SubsecRound, Utc};

    use crate::{
        codec::{copy_to_id, decode, encode},
        data::{
            basic_id::BasicId, location::Location, operator_id::OperatorId, system::System,
            RemoteIDMessage,
        },
    };

    #[test]
    fn test_get_bits_trivial() {
        assert_eq!(get_bits!(0b0000, 1..0), 0);
        assert_eq!(get_bits!(0b0000, 2..0), 0);
        assert_eq!(get_bits!(0b0000, 3..0), 0);
        assert_eq!(get_bits!(0b0000, 4..0), 0);
    }

    #[test]
    fn test_get_bits_all_ones() {
        assert_eq!(get_bits!(0b1111, 0..0), 1);
        assert_eq!(get_bits!(0b1111, 1..0), 3);
        assert_eq!(get_bits!(0b1111, 2..0), 7);
        assert_eq!(get_bits!(0b1111, 3..0), 15);

        assert_eq!(get_bits!(0b1111, 1..1), 1);
        assert_eq!(get_bits!(0b1111, 2..1), 3);
        assert_eq!(get_bits!(0b1111, 3..1), 7);

        assert_eq!(get_bits!(0b1111, 2..2), 1);
        assert_eq!(get_bits!(0b1111, 3..2), 3);

        assert_eq!(get_bits!(0b1111_1111, 3..3), 1);
        assert_eq!(get_bits!(0b1111_1111, 4..3), 3);
        assert_eq!(get_bits!(0b1111_1111, 5..3), 7);

        assert_eq!(get_bits!(0b1111_1111, 7..7), 1);
        assert_eq!(get_bits!(0b1111_1111, 8..8), 0);
        assert_eq!(get_bits!(0b1111_1111, 9..8), 0);
    }

    #[test]
    fn test_get_bits_x() {
        let n = 0b0001_1100;
        assert_eq!(get_bits!(n, 4..2), (n & 0b0001_1100) >> 2);

        let n = 34;
        assert_eq!(get_bits!(n, 7..3), (n & 0b1111_1000) >> 3);
    }

    #[test]
    fn test_bitmask_from_0() {
        assert_eq!(bitmask!(0, 0), 1);
        assert_eq!(bitmask!(0, 1), 3);
        assert_eq!(bitmask!(0, 2), 7);
        assert_eq!(bitmask!(0, 3), 15);
        assert_eq!(bitmask!(0, 4), 31);
        assert_eq!(bitmask!(0, 5), 63);
        assert_eq!(bitmask!(0, 6), 127);
        assert_eq!(bitmask!(0, 7), 255);
    }

    #[test]
    fn test_bitmask_one_bit() {
        assert_eq!(bitmask!(1, 1), 2);
        assert_eq!(bitmask!(2, 2), 4);
        assert_eq!(bitmask!(3, 3), 8);
        assert_eq!(bitmask!(4, 4), 16);
        assert_eq!(bitmask!(5, 5), 32);
    }

    #[test]
    fn test_bitmask_two_bits() {
        assert_eq!(bitmask!(1, 2), 6);
        assert_eq!(bitmask!(2, 3), 12);
        assert_eq!(bitmask!(3, 4), 24);
        assert_eq!(bitmask!(4, 5), 48);
    }

    #[test]
    fn test_bitmask_three_bits() {
        assert_eq!(bitmask!(1, 3), 14);
        assert_eq!(bitmask!(2, 4), 28);
        assert_eq!(bitmask!(3, 5), 56);
        assert_eq!(bitmask!(4, 6), 112);
    }

    extern crate std;
    #[test]
    fn test_bitmask_xxx() {
        let p = bitmask!(0, 3);
        std::dbg!(p);
    }

    #[test]
    fn test_recode_basic_id() {
        let basic_id = RemoteIDMessage::BasicID(BasicId {
            id_type: crate::data::basic_id::IdType::None,
            ua_type: crate::data::basic_id::UAType::Aeroplane,
            uas_id: copy_to_id("1\0".as_bytes()),
        });

        let mut buf = [0u8; 22];
        encode::to_message_buffer(&basic_id, &mut buf);
        let msg = decode::from_message_buffer(&buf).unwrap();
        std::dbg!(&msg);
        assert_eq!(basic_id, msg);
    }

    #[test]
    fn test_recode_location() {
        let location = RemoteIDMessage::Location(Location {
            operational_status: crate::data::location::OperationalStatus::Undeclared,
            height_type: crate::data::location::HeightType::AboveGroundLevel,
            speed: 4.0,
            vertical_speed: 2.0,
            pressure_altitude: 3.0,
            geodetic_altitude: 4.0,
            track_direction: 5,
            horizontal_accuracy: crate::data::location::HorizontalAccuracy::LessThan_10_NM,
            vertical_accuracy: crate::data::location::VerticalAccuracy::LessThan_10_m,
            latidute: 6.0,
            longitude: 7.0,
            height: 8.0,
            baro_altitude_accuracy: crate::data::location::VerticalAccuracy::LessThan_10_m,
            speed_accuracy: crate::data::location::SpeedAccuracy::LessThan_10_mps,
            timestamp: 9.0,
            timestamp_accuracy: None,
        });

        let mut buf = [0u8; 25];
        encode::to_message_buffer(&location, &mut buf);

        let msg = decode::from_message_buffer(&buf).unwrap();
        std::dbg!(&msg);
        assert_eq!(location, msg);
    }

    #[test]
    fn test_recode_operator_id() {
        let location = RemoteIDMessage::OperatorId(OperatorId {
            id_type: crate::data::operator_id::OperatorIdType::Unknown(99),
            operator_id: copy_to_id("1\0".as_bytes()),
        });

        let mut buf = [0u8; 25];
        encode::to_message_buffer(&location, &mut buf);
        let msg = decode::from_message_buffer(&buf).unwrap();

        assert_eq!(location, msg);
    }

    #[test]
    fn test_recode_system() {
        let location = RemoteIDMessage::System(System {
            classification_type: crate::data::system::ClassificationType::EuropeanUnion,
            operator_location_type: crate::data::system::OperatorLocationType::Fixed,
            operator_latidute: 10.0,
            operator_longitude: 10.0,
            area_count: 11,
            area_radius: 1230.0,
            area_ceiling: 1234.0,
            area_floor: 1234.0,
            ua_classification: crate::data::system::UaClassification {
                category: crate::data::system::UaCategory::Certified,
                class: crate::data::system::UaClass::Class0,
            },
            operator_altitude: 123.0,
            timestamp: Utc::now().trunc_subsecs(0),
        });

        let mut buf = [0u8; 25];
        encode::to_message_buffer(&location, &mut buf);
        let msg = decode::from_message_buffer(&buf).unwrap();

        assert_eq!(location, msg);
    }
}
