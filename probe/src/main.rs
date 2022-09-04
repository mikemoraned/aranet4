#[macro_use] extern crate log;

use std::error::Error;
use env_logger::Env;
use tokio::time;
use uuid::Uuid;

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

use std::convert::TryFrom;

impl TryFrom<&Vec<u8>> for Sample {
    type Error = &'static str;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let sample = Sample {
            co2 : u16::from_le_bytes([ value[0], value[1] ]),
            temp : u16::from_le_bytes([ value[2], value[3] ]) as f32 / 20.0,
            pressure : u16::from_le_bytes([ value[4], value[5] ]) as f32 / 10.0,
            humidity : value[6],
            battery : value[7],
        };
        Ok(sample)
    }
}

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let aranet_local_name_prefix = "Aranet4";
    let aranet_service_uuid : Uuid 
        = Uuid::parse_str("f0cd1400-95da-4f4b-9ac8-aa55d312af0c")?;
    let aranet_co2_measurement_characteristic_uuid : Uuid 
        = Uuid::parse_str("f0cd1503-95da-4f4b-9ac8-aa55d312af0c")?;

    let default_log_config = format!("{}=info", NAME);
    let log_config =  Env::default().default_filter_or(default_log_config);
    env_logger::Builder::from_env(log_config)
        .init();

    let cli = cli::parse();
    let cli::ScanLength(scan_length_duration) = cli.scan_length;

    info!("Running {} v{}, with {:?}", NAME, VERSION, cli);

    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        error!("No Bluetooth adapters found");
    }

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
            info!("Finding all peripheral devices in range");
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?;
                let is_connected = peripheral.is_connected().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
                if !is_connected && local_name.contains(aranet_local_name_prefix){
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
                        if service.uuid == aranet_service_uuid {
                            info!(
                                "Service UUID {}, primary: {}",
                                service.uuid, service.primary
                            );
                            for characteristic in service.characteristics {
                                if characteristic.uuid == aranet_co2_measurement_characteristic_uuid
                                    && characteristic.properties.contains(CharPropFlags::READ) {
                                    let response = peripheral.read(&characteristic).await.expect("failed read");
                                    debug!("response: {:?}", response);
                                    let sample = Sample::try_from(&response)?;
                                    info!("{}: ARANET_CO2_MEASUREMENT_CHARACTERISTIC_UUID: {:?}", &local_name, sample);
                                }
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