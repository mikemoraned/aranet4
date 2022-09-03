use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, CharPropFlags};
use btleplug::platform::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        println!("Starting scan on {}...", adapter.adapter_info().await?);
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(10)).await;
        let peripherals = adapter.peripherals().await?;
        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
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
                    println!("Connecting to peripheral {:?}...", &local_name);
                    if let Err(err) = peripheral.connect().await {
                        eprintln!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                    let is_connected = peripheral.is_connected().await?;
                    println!(
                        "Now connected ({:?}) to peripheral {:?}...",
                        is_connected, &local_name
                    );
                    peripheral.discover_services().await?;
                    println!("Discover peripheral {:?} services...", &local_name);
                    for service in peripheral.services() {
                        println!(
                            "Service UUID {}, primary: {}",
                            service.uuid, service.primary
                        );
                        for characteristic in service.characteristics {
                            if format!("{:?}", characteristic.uuid).contains("f0cd1503-95da-4f4b-9ac8-aa55d312af0c") 
                                && characteristic.properties.contains(CharPropFlags::READ) {
                                let response = peripheral.read(&characteristic).await.expect("failed read");
                                println!("response: {:?}", response);
                                let co2 = u16::from_le_bytes([ response[0], response[1] ]);
                                let temp = u16::from_le_bytes([ response[2], response[3] ]) as f32 / 20.0;
                                let pressure = u16::from_le_bytes([ response[4], response[5] ]) as f32 / 10.0;
                                let humidity = response[6];
                                let battery = response[7];
                                println!("ARANET_CO2_MEASUREMENT_CHARACTERISTIC_UUID = {}, {}, {}, {}, {}", co2, temp, pressure, humidity, battery);
                            }
                        }
                    }
                    println!("Disconnecting from peripheral {:?}...", &local_name);
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