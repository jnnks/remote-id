use basic_id::BasicId;
use location::HeightType;
use location::Location;
use location::OperationalStatus;

extern crate std;

use crate::data::*;
use crate::MAX_ID_BYTE_SIZE;
use crate::OPEN_DRONE_ID_AD_CODE;

pub fn to_service_data(msg: &RemoteIDMessage, message_counter: u8) -> [u8; 27] {
    let mut data = [0u8; 27];

    data[0] = OPEN_DRONE_ID_AD_CODE;
    data[1] = message_counter;

    let version = 2;

    match msg {
        RemoteIDMessage::BasicID(basic_id) => {
            data[2] = (basic_id::MESSAGE_TYPE << 4) | version;
            encode_basic_id(basic_id, &mut data[2..]);
        }
        RemoteIDMessage::Location(location) => {
            data[2] = (location::MESSAGE_TYPE << 4) | version;
            encode_location(location, &mut data[2..]);
        }

        _ => todo!(),
    }

    data
}

fn encode_basic_id(msg: &BasicId, target: &mut [u8]) {
    let first_nibble: u8 = msg.id_type.into();
    let last_nibble: u8 = msg.ua_type.into();
    target[1] = (first_nibble << 4) | last_nibble;

    target[2..(MAX_ID_BYTE_SIZE + 2)].clone_from_slice(&msg.uas_id);
}

fn encode_location(msg: &Location, target: &mut [u8]) {
    let operational_status: u8 = OperationalStatus::into(msg.operational_status);
    let height_type: u8 = HeightType::into(msg.height_type);
    let ew_direction_segment: u8 = if msg.track_direction > 180 { 1 } else { 0 };

    let speed_multiplier: u8 = if msg.speed > 255. * 0.25 { 1 } else { 0 };

    target[1] =
        operational_status << 3 | height_type << 2 | ew_direction_segment << 1 | speed_multiplier;

    // Track Direction
    target[2] = if msg.track_direction > 180 {
        (msg.track_direction - 180) as u8
    } else {
        msg.track_direction as u8
    };

    // Speed
    target[3] = if msg.speed <= 255. * 0.25 {
        (msg.speed * 0.25) as u8
    } else if msg.speed > 255. * 0.25 && msg.speed < 254.25 {
        ((msg.speed - (255. * 0.25)) / 0.75) as u8
    } else {
        254
    };

    // Vertical Speed
    target[4] = (msg.vertical_speed / 0.5) as u8;

    // Latitude
    let lat = (msg.latidute * f32::powf(10., 7.)) as u32;
    target[5..9].clone_from_slice(&lat.to_le_bytes());

    // Longitude
    let lon = (msg.longitude * f32::powf(10., 7.)) as u32;
    target[9..13].clone_from_slice(&lon.to_le_bytes());

    // Pressure Altitude
    let pressure_altitude = ((msg.pressure_altitude + 1000.) / 0.5) as u16;
    target[13..15].clone_from_slice(&pressure_altitude.to_le_bytes());

    // Geodetic Altitude
    let geodetic_altitude = ((msg.geodetic_altitude + 1000.) / 0.5) as u16;
    target[15..17].clone_from_slice(&geodetic_altitude.to_le_bytes());

    // Height
    let height = ((msg.height + 1000.) / 0.5) as u16;
    target[17..19].clone_from_slice(&height.to_le_bytes());

    // Vertical / Horizontal Accuracy
    let vertical_accuracy: u8 = msg.vertical_accuracy.into();
    let horizontal_accuracy: u8 = msg.horizontal_accuracy.into();
    target[19] = vertical_accuracy << 4 | horizontal_accuracy;

    // Baro Altitude Accuracy / Speed Accuracy
    let baro_altitude_accuracy: u8 = msg.baro_altitude_accuracy.into();
    let speed_accuracy: u8 = msg.speed_accuracy.into();
    target[20] = baro_altitude_accuracy << 4 | speed_accuracy;

    // Timestamp
    let timestamp = (msg.timestamp * 10.) as u16;
    target[21..23].clone_from_slice(&timestamp.to_le_bytes());

    // Reserved / Timestamp Accuracy
    let timestamp_accuracy = if let Some(acc) = msg.timestamp_accuracy {
        (acc.as_secs_f32() / 0.1) as u8
    } else {
        0
    };
    target[23] = timestamp_accuracy;

    // Reserved
    target[24] = 0;
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::basic_id::{IdType, UAType};
    use super::location::{HeightType, Location, OperationalStatus};
    use crate::codec::copy_to_id;
    use crate::codec::encode::to_service_data;
    use crate::data::basic_id::BasicId;
    use crate::data::RemoteIDMessage;

    #[test]
    fn encode_basic_id() {
        // DroneTag Mini
        let basic_id = RemoteIDMessage::BasicID(BasicId {
            id_type: IdType::SerialNumber,
            ua_type: UAType::None,
            uas_id: copy_to_id("1596F359746167260749".as_bytes()),
        });

        let service_data = [
            13, 1, 2, 16, 49, 53, 57, 54, 70, 51, 53, 57, 55, 52, 54, 49, 54, 55, 50, 54, 48, 55,
            52, 57, 0, 0, 0,
        ];

        assert_eq!(service_data, to_service_data(&basic_id, 1));
    }

    #[test]
    fn encode_basic_id_2() {
        // DroneTag BS
        let basic_id = RemoteIDMessage::BasicID(BasicId {
            id_type: IdType::SerialNumber,
            ua_type: UAType::None,
            uas_id: copy_to_id("1596F3170CE908F55122".as_bytes()),
        });

        let expected = [
            13, 1, 2, 16, 49, 53, 57, 54, 70, 51, 49, 55, 48, 67, 69, 57, 48, 56, 70, 53, 53, 49,
            50, 50, 0, 0, 0,
        ];

        assert_eq!(expected, to_service_data(&basic_id, 1));
    }

    #[test]
    fn encode_location_1() {
        let location = RemoteIDMessage::Location(Location {
            height_type: HeightType::AboveTakeoff,
            operational_status: OperationalStatus::RemoteIdSystemFailure,
            speed: 10.,
            vertical_speed: 10.,
            pressure_altitude: 190.5,
            geodetic_altitude: 210.0,
            baro_altitude_accuracy: crate::data::location::VerticalAccuracy::Unknown,
            horizontal_accuracy: crate::data::location::HorizontalAccuracy::LessThan_3_m,
            speed_accuracy: crate::data::location::SpeedAccuracy::LessThan_third_mps,
            vertical_accuracy: crate::data::location::VerticalAccuracy::LessThan_3_m,
            track_direction: 77,
            latidute: 49.874855,
            longitude: 8.912173,
            height: 0.,
            timestamp: 361.0,
            timestamp_accuracy: None,
        });
        let expected = [
            13, 1, 18, 32, 77, 2, 20, 128, 76, 186, 29, 200, 227, 79, 5, 77, 9, 116, 9, 208, 7, 91,
            4, 26, 14, 0, 0,
        ];
        assert_eq!(expected, to_service_data(&location, 1));
    }
}
