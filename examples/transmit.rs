#![cfg(feature = "transmit")]

use bluer::adv::Advertisement;
use remote_id::{
    codec::{copy_to_id, encode},
    data::{basic_id::BasicId, location::Location},
};
use std::time::Duration;
use uuid::Uuid;

/// Transmit Example Using bluer
///
/// Will advertise a remote-id signal with a fake uas-id and location in Frankfurt, Germany
#[tokio::main]
async fn main() -> bluer::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.adapter("hci0")?;

    adapter.set_powered(true).await?;
    let props = adapter.all_properties().await?;

    println!("ADAPTER NAME:    {}", adapter.name());
    println!("ADAPTER ADDRESS: {}", adapter.address().await?);

    let messages = [
        remote_id::data::RemoteIDMessage::BasicID(BasicId {
            id_type: remote_id::data::basic_id::IdType::SerialNumber,
            ua_type: remote_id::data::basic_id::UAType::None,
            uas_id: copy_to_id("1234567890123456789\0".as_bytes()),
        }),
        remote_id::data::RemoteIDMessage::Location(Location {
            operational_status: remote_id::data::location::OperationalStatus::Ground,
            height_type: remote_id::data::location::HeightType::AboveGroundLevel,
            speed: 0.0,
            vertical_speed: 0.0,
            pressure_altitude: 0.0,
            geodetic_altitude: 0.0,
            track_direction: 0,
            horizontal_accuracy: remote_id::data::location::HorizontalAccuracy::Unknown,
            vertical_accuracy: remote_id::data::location::VerticalAccuracy::Unknown,
            latidute: 50.0828829,
            longitude: 8.6959298,
            height: 0.0,
            baro_altitude_accuracy: remote_id::data::location::VerticalAccuracy::Unknown,
            speed_accuracy: remote_id::data::location::SpeedAccuracy::Unknown,
            timestamp: 0.0,
            timestamp_accuracy: None,
        }),
    ]
    .into_iter()
    .cycle()
    .take(100);

    for (i, m) in messages.enumerate() {
        let message_bytes = encode::to_service_data(&m, (i % 255) as u8);

        let le_advertisement = Advertisement {
            advertisement_type: bluer::adv::Type::Peripheral,
            system_includes: vec![].into_iter().collect(),
            service_data: vec![(
                Uuid::from_u128(0x0000fffa_0000_1000_8000_00805f9b34fb),
                message_bytes.to_vec(),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        let handle = adapter.advertise(le_advertisement).await?;

        tokio::time::sleep(Duration::from_millis(250)).await;

        drop(handle);
    }

    Ok(())
}
