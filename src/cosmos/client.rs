use tendermint_rpc::{
    query::EventType, Error as TendermintRpcError, Subscription, SubscriptionClient,
    WebSocketClient, WebSocketClientDriver,
};
use tokio::task::JoinHandle;

use super::NodeConfig;

#[derive(Debug)]
pub enum ClientError {
    TendermintRpcError(TendermintRpcError),
    InvalidState {
        expected: ClientState,
        current: ClientState,
    },
}

impl ClientError {
    fn expected_running(current: ClientState) -> Self {
        Self::invalid_state(ClientState::Running, current)
    }

    fn expected_initialized(current: ClientState) -> Self {
        Self::invalid_state(ClientState::Initialized, current)
    }

    fn invalid_state(expected: ClientState, current: ClientState) -> Self {
        Self::InvalidState { expected, current }
    }
}

impl Into<ClientError> for TendermintRpcError {
    fn into(self) -> ClientError {
        ClientError::TendermintRpcError(self)
    }
}

#[derive(Debug)]
pub enum ClientState {
    Initialized,
    Running,
}

pub enum Client {
    Initialized(InitClient),
    Running(RunningClient),
}

/// Cosmos node client.
pub struct InitClient {
    ws_client: WebSocketClient,
    ws_driver: WebSocketClientDriver,
}

pub struct RunningClient {
    ws_client: WebSocketClient,
    driver_handle: JoinHandle<Result<(), TendermintRpcError>>,
}

impl Client {
    pub async fn new(node_config: NodeConfig) -> Result<Self, ClientError> {
        let address = if node_config.secure {
            format!(
                "wss://{}:{}/websocket",
                node_config.rpc_addr, node_config.rpc_port
            )
        } else {
            format!(
                "ws://{}:{}/websocket",
                node_config.rpc_addr, node_config.rpc_port
            )
        };

        let (client, driver) = WebSocketClient::new(address.as_str())
            .await
            .map_err(Into::into)?;
        Ok(Client::Initialized(InitClient {
            ws_client: client,
            ws_driver: driver,
        }))
    }

    pub fn run(self) -> Result<Self, ClientError> {
        match self {
            Client::Initialized(init_client) => {
                let driver_handle = tokio::spawn(async move { init_client.ws_driver.run().await });

                Ok(Client::Running(RunningClient {
                    ws_client: init_client.ws_client,
                    driver_handle,
                }))
            }
            Client::Running(_) => {
                println!("[ERROR]: client already running. Ignoring...");
                Err(ClientError::expected_initialized(ClientState::Running))
            }
        }
    }

    pub async fn subscribe_to_blocks(&self) -> Result<Subscription, ClientError> {
        match self {
            Client::Initialized(_) => Err(ClientError::expected_running(ClientState::Initialized)),
            Client::Running(client) => {
                // Subscription functionality
                client
                    .ws_client
                    .subscribe(EventType::NewBlock.into())
                    .await
                    .map_err(Into::into)
            }
        }
    }

    pub fn close(self) -> Result<(), ClientError> {
        match self {
            Client::Initialized(client) => client.ws_client.close().map_err(Into::into),
            Client::Running(client) => client.ws_client.close().map_err(Into::into),
        }
    }
}
