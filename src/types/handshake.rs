#[derive(Clone, Debug)]
pub struct HanshakeCompleteEvent {
    pub node_address: String,
    pub success: bool,
}
