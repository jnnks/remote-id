#![cfg(feature = "linux")]

use bluer::{DeviceEvent, DeviceProperty, DiscoveryFilter};
use futures::stream::StreamExt;
use remote_id::codec::decode;
use uuid::Uuid;

const REMOTE_ID_SERVICE_UUID: Uuid = Uuid::from_u128(remote_id::REMOTE_ID_SERVICE_UUID);

/// Receive Example Using bluer
///
/// Will scan for bluetooth devices and listen to their broadcast messages,
/// printing parsable remote id messages
#[tokio::main]
async fn main() {
    let session = bluer::Session::new()
        .await
        .expect("failed to initiate bluetooth session");

    let adapter = session
        .adapter("hci0")
        .expect("failed to find bluetooth adapter");

    adapter
        .set_powered(true)
        .await
        .expect("failed to power bluetooth device");

    println!("ADAPTER NAME:    {}", adapter.name());
    println!(
        "ADAPTER ADDRESS: {}",
        adapter
            .address()
            .await
            .expect("failed to get device address")
    );

    adapter
        .set_discovery_filter(DiscoveryFilter {
            // required to receive service data
            duplicate_data: true,
            ..Default::default()
        })
        .await
        .expect("failed to set discovery filter");

    let mut discovery = adapter
        .discover_devices()
        .await
        .expect("failed to start device discovery session");
    println!("Scanning for BLE advertisements...\n");

    while let Some(event_result) = discovery.next().await {
        if let bluer::AdapterEvent::DeviceAdded(address) = event_result {
            if let Ok(device) = adapter.device(address) {
                println!("Device Discovered: {} {:?}", address, device.alias().await);
                tokio::spawn(handle_device(device));
            }
        }
    }
}

async fn handle_device(device: bluer::Device) {
    let mut events = device.events().await.expect("failed to get events");
    let mac_addr = device.address();
    while let Some(ev) = events.next().await {
        if let DeviceEvent::PropertyChanged(DeviceProperty::ServiceData(service_data)) = ev {
            if let Some(service_data) = service_data.get(&REMOTE_ID_SERVICE_UUID) {
                if let Some(rid_msg) = decode::from_service_data(&service_data) {
                    println!("{mac_addr}: {rid_msg:?}");
                }
            }
        }
    }
}
