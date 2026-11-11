#![forbid(unsafe_code)]

//! Regulation Service: WiZ UDP hardware abstraction layer.
//!
//! This module is deliberately "dumb". It translates a [`LightingCommand`]
//! that has already been arbitrated by the Controller into the WiZ JSON
//! protocol and sends it over UDP to port [`WIZ_PORT`]. It contains no
//! priority handling, scheduling, recommendations, or other business logic.
//!
//! Failure policy: send errors are returned to the caller (and logged as
//! non-fatal warnings) instead of panicking. The Controller is responsible
//! for retrying on its next reconciliation/control cycle.

use std::fmt::{self, Write as _};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};

use crate::contracts::LightingCommand;

/// WiZ bulbs listen for JSON datagrams on this UDP port.
pub const WIZ_PORT: u16 = 38899;

/// Errors produced by the Regulation Service.
#[derive(Debug)]
pub enum RegulationError {
    /// The local UDP socket could not be created.
    Bind {
        address: SocketAddr,
        source: io::Error,
    },
    /// The datagram could not be handed to the operating system.
    Send {
        target: SocketAddr,
        source: io::Error,
    },
}

impl fmt::Display for RegulationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bind { address, source } => {
                write!(
                    formatter,
                    "failed to bind UDP socket to {address}: {source}"
                )
            }
            Self::Send { target, source } => {
                write!(
                    formatter,
                    "failed to send UDP payload to {target}: {source}"
                )
            }
        }
    }
}

impl std::error::Error for RegulationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bind { source, .. } | Self::Send { source, .. } => Some(source),
        }
    }
}

/// Hardware abstraction layer for one WiZ bulb endpoint.
///
/// The struct owns a bound UDP socket. It performs no background work and
/// therefore consumes no CPU while idle; commands are sent synchronously when
/// the Controller calls [`RegulationService::execute`].
#[derive(Debug)]
pub struct RegulationService {
    socket: UdpSocket,
    target: SocketAddr,
}

impl RegulationService {
    /// Creates a service targeting `target`.
    ///
    /// The local socket is bound to an ephemeral port on the unspecified
    /// address matching the target's IP family.
    pub fn new(target: SocketAddr) -> Result<Self, RegulationError> {
        let local = unspecified_address_for(target);
        Self::bind(local, target)
    }

    /// Creates a service targeting `ip` on the standard WiZ port.
    pub fn with_ip(ip: IpAddr) -> Result<Self, RegulationError> {
        Self::new(SocketAddr::new(ip, WIZ_PORT))
    }

    /// Creates a service with an explicit local bind address.
    ///
    /// This is useful when the daemon must pin its source address/interface.
    pub fn bind(local: SocketAddr, target: SocketAddr) -> Result<Self, RegulationError> {
        let socket = UdpSocket::bind(local).map_err(|source| RegulationError::Bind {
            address: local,
            source,
        })?;

        Ok(Self { socket, target })
    }

    /// The bulb endpoint this service sends to.
    pub fn target(&self) -> SocketAddr {
        self.target
    }

    /// Sends one already-arbitrated command to the bulb.
    ///
    /// Priority and `source` are intentionally ignored here: priority was
    /// resolved by the Controller, and `source` is internal metadata that is
    /// not part of the WiZ protocol.
    pub fn execute(&self, command: &LightingCommand) -> Result<(), RegulationError> {
        let payload = encode_set_pilot(command);

        if let Err(error) = self.send_payload(payload.as_bytes()) {
            // UDP is lossy; a failed send must never take the daemon down.
            // The Controller will retry on its next control cycle.
            eprintln!(
                "regulation: warning: could not send command from '{}' to {}: {}",
                command.source, self.target, error
            );
            return Err(error);
        }

        Ok(())
    }

    /// Sends a raw pre-encoded payload on the service socket.
    fn send_payload(&self, payload: &[u8]) -> Result<(), RegulationError> {
        self.socket
            .send_to(payload, self.target)
            .map(|_| ())
            .map_err(|source| RegulationError::Send {
                target: self.target,
                source,
            })
    }
}

/// Selects the unspecified bind address matching the target's address family.
fn unspecified_address_for(target: SocketAddr) -> SocketAddr {
    match target {
        SocketAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
        SocketAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0),
    }
}

/// Encodes the WiZ `setPilot` payload described in `docs/contracts.md`.
///
/// All values are typed (`bool`, `u8`, `u16`), so no string escaping is
/// required and the payload can be built without a JSON dependency.
///
/// This is public so protocol framing can be verified without opening a
/// socket. It remains a pure translation step with no policy decisions.
pub fn encode_set_pilot(command: &LightingCommand) -> String {
    let mut payload = String::with_capacity(128);
    payload.push_str("{\"method\":\"setPilot\",\"params\":{\"state\":");
    payload.push_str(if command.state { "true" } else { "false" });
    let _ = write!(payload, ",\"dimming\":{}", command.dimming);

    if let Some(kelvin) = command.color_temp {
        let _ = write!(payload, ",\"temp\":{kelvin}");
    }

    if let Some((red, green, blue)) = command.rgb {
        let _ = write!(payload, ",\"r\":{red},\"g\":{green},\"b\":{blue}");
    }

    payload.push_str("}}");
    payload
}
