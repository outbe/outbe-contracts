use cosmwasm_std::CosmosMsg;
use prost::Message;

#[derive(Clone, PartialEq, prost::Message)]
pub struct MineTokensMsg {
    #[prost(string, tag = "1")]
    pub sender: String,
    #[prost(string, tag = "2")]
    pub recipient: String,
    #[prost(message, optional, tag = "3")]
    pub amount: Option<ProtoCoin>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoCoin {
    #[prost(string, tag = "1")]
    pub denom: String,
    #[prost(string, tag = "2")]
    pub amount: String,
}

impl From<MineTokensMsg> for CosmosMsg {
    fn from(val: MineTokensMsg) -> Self {
        let serialized = val.encode_to_vec();
        #[allow(deprecated)]
        CosmosMsg::Stargate {
            type_url: "/outbe.tokenminer.MsgMineTokens".to_string(),
            value: serialized.into(),
        }
    }
}
