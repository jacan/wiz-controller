//! Integration tests for the Regulation Service.
//!
//! These tests exercise the public crate API only, keeping them isolated from
//! the implementation details in `src/udp.rs`.

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use wiz_controller::contracts::{LightingCommand, Priority};
use wiz_controller::udp::{encode_set_pilot, RegulationService, WIZ_PORT};

fn command(
    state: bool,
    dimming: u8,
    color_temp: Option<u16>,
    rgb: Option<(u16, u16, u16)>,
) -> LightingCommand {
    LightingCommand {
        priority: Priority::Standard,
        state,
        dimming,
        color_temp,
        rgb,
        source: "test".to_owned(),
    }
}

#[test]
fn encodes_set_pilot_without_colour_temperature() {
    let payload = encode_set_pilot(&command(true, 30, None, None));
    assert_eq!(
        payload,
        r#"{"method":"setPilot","params":{"state":true,"dimming":30}}"#
    );
}

#[test]
fn encodes_set_pilot_with_colour_temperature() {
    let payload = encode_set_pilot(&command(false, 5, Some(4500), None));
    assert_eq!(
        payload,
        r#"{"method":"setPilot","params":{"state":false,"dimming":5,"temp":4500}}"#
    );
}

#[test]
fn encodes_set_pilot_with_rgb_colour() {
    let payload = encode_set_pilot(&command(false, 20, None, Some((255, 128, 0))));
    assert_eq!(
        payload,
        r#"{"method":"setPilot","params":{"state":false,"dimming":20,"r":255,"g":128,"b":0}}"#
    );
}

#[test]
fn does_not_leak_controller_metadata_onto_the_wire() {
    let payload = encode_set_pilot(&command(true, 10, None, None));
    assert!(!payload.contains("priority"));
    assert!(!payload.contains("source"));
    assert!(!payload.contains("Standard"));
}

#[test]
fn sends_a_well_formed_datagram() -> Result<(), Box<dyn Error>> {
    let receiver = UdpSocket::bind("127.0.0.1:0")?;
    receiver.set_read_timeout(Some(Duration::from_secs(2)))?;

    let service = RegulationService::new(receiver.local_addr()?)?;
    service.execute(&command(true, 30, Some(2700), None))?;

    let mut buffer = [0_u8; 512];
    let (length, _) = receiver.recv_from(&mut buffer)?;

    assert_eq!(
        &buffer[..length],
        br#"{"method":"setPilot","params":{"state":true,"dimming":30,"temp":2700}}"#
    );

    Ok(())
}

#[test]
fn with_ip_uses_the_wiz_port() -> Result<(), Box<dyn Error>> {
    let service = RegulationService::with_ip(IpAddr::V4(Ipv4Addr::LOCALHOST))?;
    assert_eq!(
        service.target(),
        SocketAddr::from(([127, 0, 0, 1], WIZ_PORT))
    );

    Ok(())
}
