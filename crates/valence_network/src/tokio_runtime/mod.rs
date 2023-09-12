use bevy_ecs::system::Res;
use tokio::runtime::{Runtime, Handle};

use crate::SharedNetworkState;

#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    /// The [`Handle`] to the tokio runtime the server will use. If `None` is
    /// provided, the server will create its own tokio runtime at startup.
    ///
    /// # Default Value
    ///
    /// `None`
    pub handle: Handle
}

pub(crate) struct RuntimeState {
    pub handle: Handle,
    // Holding a runtime handle is not enough to keep tokio working. We need
    // to store the runtime here so we don't drop it.
    pub runtime: Option<Runtime>,
}

pub(crate) fn start_accept_loop(shared: SharedNetworkState) {
    match &shared.0.runtime_state {
        crate::RuntimeState::Tokio(s) => {
                let _guard = s.handle.enter();
        
                // Start accepting new connections.
                tokio::spawn(crate::do_accept_loop(shared.clone()));

        }
        _ => panic!()
    }
}

pub(crate) fn start_broadcast_to_lan_loop(shared: SharedNetworkState) {
    match &shared.0.runtime_state {
        crate::RuntimeState::Tokio(s) => {
                let _guard = s.handle.enter();
        
                // Start accepting new connections.
                tokio::spawn(crate::do_broadcast_to_lan_loop(shared.clone()));

        }
        _ => panic!()
    }
}