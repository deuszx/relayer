pub struct NodeConfig {
    pub(crate) rpc_addr: &'static str,
    pub(crate) rpc_port: u16,
    pub(crate) secure: bool,
}

impl NodeConfig {
    const LOCALHOST: &'static str = "localhost";

    pub(crate) const LOCAL: NodeConfig = NodeConfig {
        rpc_addr: NodeConfig::LOCALHOST,
        rpc_port: 26657,
        secure: false,
    };
}