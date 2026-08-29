use crate::error::{AppError, AppResult};
use serde::Serialize;
use serialport::{SerialPortInfo as NativeSerialPortInfo, SerialPortType};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialPortInfo {
    pub path: String,
    pub display_name: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub recommended: bool,
}

pub fn available_port_infos() -> AppResult<Vec<SerialPortInfo>> {
    let native_ports = serialport::available_ports()
        .map_err(|error| AppError::serial(format!("Výčet portů selhal: {error}")))?;

    let mut deduplicated: BTreeMap<String, NativeSerialPortInfo> = BTreeMap::new();
    for port in native_ports {
        let key = macos_device_key(&port.port_name);
        let should_replace = deduplicated
            .get(&key)
            .map(|existing| {
                port.port_name.starts_with("/dev/cu.")
                    && !existing.port_name.starts_with("/dev/cu.")
            })
            .unwrap_or(true);
        if should_replace {
            deduplicated.insert(key, port);
        }
    }

    let mut ports: Vec<SerialPortInfo> = deduplicated.into_values().map(to_port_info).collect();
    ports.sort_by(|left, right| {
        right
            .recommended
            .cmp(&left.recommended)
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(ports)
}

fn to_port_info(port: NativeSerialPortInfo) -> SerialPortInfo {
    let (manufacturer, product, serial_number, vid, pid) = match port.port_type {
        SerialPortType::UsbPort(usb) => (
            usb.manufacturer,
            usb.product,
            usb.serial_number,
            Some(usb.vid),
            Some(usb.pid),
        ),
        _ => (None, None, None, None, None),
    };
    let recommended = port.port_name.starts_with("/dev/cu.")
        && (port.port_name.contains("usbserial") || product.is_some());
    let display_name = product
        .clone()
        .map(|product| format!("{product} — {}", port.port_name))
        .unwrap_or_else(|| port.port_name.clone());

    SerialPortInfo {
        path: port.port_name,
        display_name,
        manufacturer,
        product,
        serial_number,
        vid,
        pid,
        recommended,
    }
}

fn macos_device_key(path: &str) -> String {
    path.strip_prefix("/dev/tty.")
        .or_else(|| path.strip_prefix("/dev/cu."))
        .unwrap_or(path)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{macos_device_key, SerialPortInfo};

    #[test]
    fn callout_and_dialin_paths_share_a_key() {
        assert_eq!(
            macos_device_key("/dev/cu.usbserial-123"),
            macos_device_key("/dev/tty.usbserial-123")
        );
    }

    #[test]
    fn non_usb_metadata_is_serialized_as_null() {
        let info = SerialPortInfo {
            path: "/dev/cu.Bluetooth-Incoming-Port".into(),
            display_name: "/dev/cu.Bluetooth-Incoming-Port".into(),
            manufacturer: None,
            product: None,
            serial_number: None,
            vid: None,
            pid: None,
            recommended: false,
        };

        let json = serde_json::to_value(info).unwrap();
        assert!(json["vid"].is_null());
        assert!(json["pid"].is_null());
    }
}
