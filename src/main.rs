use esp_idf_svc::sys::*;
mod provisioning;

use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

fn main() -> Result<(), EspError> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let prov = provisioning::WifiProvisioning::new()?;
    if !prov.is_provisioned()? {
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ..Default::default()
        });
        wifi.set_configuration(&wifi_configuration)?;
        wifi.start()?;
        prov.start_provisioning(
            wifi_prov_security_WIFI_PROV_SECURITY_1,
            "abcd1234",    // Proof of Possession (POP)
            "PROV_ESP32",  // Service Name
            None,          // No Service Key
        )?;

        println!("Waiting for Wi-Fi provisioning...");
        prov.wait();

        println!("Provisioning completed. Stopping...");
        prov.stop();

    } else {
        wifi.start()?;
        wifi.connect()?;
    }
    wifi.wait_netif_up()?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    println!("Wifi DHCP info: {:?}", ip_info);

    Ok(())
}
