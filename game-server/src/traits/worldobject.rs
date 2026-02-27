use crate::error::ZResult;
use serde_json;

type ClientData = serde_json::Value;

pub trait ToClientData {
    fn to_client_data(&self) -> ZResult<ClientData>;
    fn update_from_client_data(&mut self, data: ClientData) -> ZResult<()>;
}
