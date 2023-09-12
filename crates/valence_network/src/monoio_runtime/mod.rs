use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use monoio::net::udp::UdpSocket;
use monoio::net::{TcpListener, TcpStream};
use tracing::{error, trace, warn};
use valence_server::protocol::{PacketDecoder, PacketEncoder};

use crate::legacy_ping::try_handle_legacy_ping;
use crate::packet_io::PacketIo;
use crate::{BroadcastToLan, SharedNetworkState};

#[derive(Debug, Clone)]
pub struct RuntimeOptions {}

pub(crate) struct RuntimeState {}

pub(crate) fn start_accept_loop(shared: SharedNetworkState) {
    match &shared.0.runtime_state {
        crate::RuntimeState::Monoio(s) => todo!(),
        _ => panic!(),
    }
}

pub(crate) fn start_broadcast_to_lan_loop(shared: SharedNetworkState) {
    match &shared.0.runtime_state {
        crate::RuntimeState::Monoio(s) => todo!(),
        _ => panic!(),
    }
}

/// Accepts new connections to the server as they occur.
pub(super) async fn do_accept_loop(shared: SharedNetworkState) {
    let listener = match TcpListener::bind(shared.0.address) {
        Ok(listener) => listener,
        Err(e) => {
            error!("failed to start TCP listener: {e}");
            return;
        }
    };

    let timeout = Duration::from_secs(5);

    loop {
        match shared.0.connection_sema.clone().acquire_owned().await {
            Ok(permit) => match listener.accept().await {
                Ok((stream, remote_addr)) => {
                    let shared = shared.clone();

                    monoio::spawn(async move {
                        if let Err(e) = monoio::time::timeout(
                            timeout,
                            handle_connection(shared, stream, remote_addr),
                        )
                        .await
                        {
                            warn!("initial connection timed out: {e}");
                        }

                        drop(permit);
                    });
                }
                Err(e) => {
                    error!("failed to accept incoming connection: {e}");
                }
            },
            // Closed semaphore indicates server shutdown.
            Err(_) => return,
        }
    }
}

async fn handle_connection(
    shared: SharedNetworkState,
    mut stream: TcpStream,
    remote_addr: SocketAddr,
) {
    trace!("handling connection");

    if let Err(e) = stream.set_nodelay(true) {
        error!("failed to set TCP_NODELAY: {e}");
    }

    match try_handle_legacy_ping(&shared, &mut stream, remote_addr).await {
        Ok(true) => return, // Legacy ping succeeded.
        Ok(false) => {}     // No legacy ping.
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {}
        Err(e) => {
            warn!("legacy ping  ended with error: {e:#}");
        }
    }

    let io = PacketIo::new(stream, PacketEncoder::new(), PacketDecoder::new());

    if let Err(e) = crate::connect::handle_handshake(shared, io, remote_addr).await {
        // EOF can happen if the client disconnects while joining, which isn't
        // very erroneous.
        if let Some(e) = e.downcast_ref::<io::Error>() {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return;
            }
        }
        warn!("connection ended with error: {e:#}");
    }
}

async fn do_broadcast_to_lan_loop(shared: SharedNetworkState) {
    let port = shared.0.address.port();

    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        tracing::error!("Failed to bind to UDP socket for broadcast to LAN");
        return;
    };

    loop {
        let motd = match shared.0.callbacks.inner.broadcast_to_lan(&shared).await {
            BroadcastToLan::Disabled => {
                monoio::time::sleep(Duration::from_millis(1500)).await;
                continue;
            }
            BroadcastToLan::Enabled(motd) => motd,
        };

        let message = format!("[MOTD]{motd}[/MOTD][AD]{port}[/AD]");

        if let Err(e) = socket
            .send_to(message.as_bytes(), "224.0.2.60:4445".parse().unwrap())
            .await
            .0
        {
            tracing::warn!("Failed to send broadcast to LAN packet: {}", e);
        }

        // wait 1.5 seconds
        monoio::time::sleep(std::time::Duration::from_millis(1500)).await;
    }
}
