#[macro_use] extern crate log;

use std::error::Error;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, CharPropFlags};
use btleplug::platform::Manager;

mod cli;

#[derive(Debug)]
struct Sample {
    co2: u16,
    temp: f32,
    pressure: f32,
    humidity: u8,
    battery: u8
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::formatted_builder()
        .filter_module("probe", log::LevelFilter::Info)
        .init();

    let cli = cli::parse();

    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        error!("No Bluetooth adapters found");
    }

    let cli::ScanLength(scan_length_duration) = cli.scan_length;

    for adapter in adapter_list.iter() {
        info!("Starting {:?} scan on {}...", scan_length_duration, adapter.adapter_info().await?);
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(scan_length_duration).await;
        let peripherals = adapter.peripherals().await?;
        if peripherals.is_empty() {
            error!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            // All peripheral devices in range
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?;
                let is_connected = peripheral.is_connected().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
                if !is_connected && local_name.contains("Aranet4"){
                    info!("Connecting to peripheral {:?}...", &local_name);
                    if let Err(err) = peripheral.connect().await {
                        error!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                    let is_connected = peripheral.is_connected().await?;
                    info!(
                        "Now connected ({:?}) to peripheral {:?}...",
                        is_connected, &local_name
                    );
                    peripheral.discover_services().await?;
                    info!("Discover peripheral {:?} services...", &local_name);
                    for service in peripheral.services() {
                        info!(
                            "Service UUID {}, primary: {}",
                            service.uuid, service.primary
                        );
                        for characteristic in service.characteristics {
                            if format!("{:?}", characteristic.uuid).contains("f0cd1503-95da-4f4b-9ac8-aa55d312af0c") 
                                && characteristic.properties.contains(CharPropFlags::READ) {
                                let response = peripheral.read(&characteristic).await.expect("failed read");
                                debug!("response: {:?}", response);
                                let sample = Sample {
                                    co2 : u16::from_le_bytes([ response[0], response[1] ]),
                                    temp : u16::from_le_bytes([ response[2], response[3] ]) as f32 / 20.0,
                                    pressure : u16::from_le_bytes([ response[4], response[5] ]) as f32 / 10.0,
                                    humidity : response[6],
                                    battery : response[7],
                                };
                                info!("{}: ARANET_CO2_MEASUREMENT_CHARACTERISTIC_UUID: {:?}", &local_name, sample);
                            }
                        }
                    }
                    info!("Disconnecting from peripheral {:?}...", &local_name);
                    peripheral
                        .disconnect()
                        .await
                        .expect("Error disconnecting from BLE peripheral");
                }
            }
        }
        
    }

    Ok(())
}