use crate::SharedNetworkState;

#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    
}

pub(crate) struct RuntimeState {

}

pub(crate) fn start_accept_loop(shared: SharedNetworkState) {
    match &shared.0.runtime_state {
        crate::RuntimeState::Monoio(s) => todo!(),
        _ => panic!()
    }
}

pub(crate) fn start_broadcast_to_lan_loop(shared: SharedNetworkState) {
    match &shared.0.runtime_state {
        crate::RuntimeState::Tokio(s) => todo!(),
        _ => panic!()
    }
}