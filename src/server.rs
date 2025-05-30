/// Replace the below import with the contract you want the MCP server
/// to support
use cw20_wrap::msg::{ExecuteMsg, QueryMsg};

use cosmwasm_std::{Coin, CosmosMsg, QueryRequest, Uint128, WasmMsg, WasmQuery, to_json_binary};
use rmcp::{
    Error, ServerHandler, model::CallToolResult, model::Content, model::Implementation,
    model::ProtocolVersion, model::ServerCapabilities, model::ServerInfo, tool,
};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::contract::*;
use crate::execute::*;
use crate::instruction::*;
use crate::query::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ServerTransport {
    Stdio,
    Sse,
    StreamableHttp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CwMcp {
    contracts: [CwContract; 2],
}

#[tool(tool_box)]
impl CwMcp {
    pub fn new() -> Self {
        Self {
            contracts: [
                CwContract {
                    network: Network::Mainnet,
                    chain_id: "archway-1".to_string(),
                    contract_address: CONTRACT_MAINNET.to_string(),
                },
                CwContract {
                    network: Network::Testnet,
                    chain_id: "constantine-3".to_string(),
                    contract_address: CONTRACT_TESTNET.to_string(),
                },
            ],
        }
    }

    /// List deployed contracts, networks, chain ids
    #[tool(description = LIST_CONTRACTS_DESCR)]
    async fn list_contract_deployments(&self) -> Result<CallToolResult, Error> {
        let serialized: String = serde_json::to_string(&self.contracts).unwrap_or("".to_string());
        Ok(CallToolResult::success(vec![Content::text(serialized)]))
    }

    /// List query entry points
    #[tool(description = LIST_QUERY_ENTRY_POINTS_DESCR)]
    async fn list_query_entry_points(&self) -> Result<CallToolResult, Error> {
        let schema = schema_for!(QueryMsg);
        let serialized: String = serde_json::to_string(&schema).unwrap_or("".to_string());
        Ok(CallToolResult::success(vec![Content::text(serialized)]))
    }

    // (Optionally) if your contract provides any custom query response types
    // configure this tool so the MCP agent can access them. Allowing the MCP
    // agent to access the custom query responses enables it to provide smarter
    // advice, and summaries, about exacly what data can be fetched when making
    // a query to the contract.
    // @see: src/query.rs
    // Uncomment (and configure) to use:
    // #[tool(description = LIST_QUERY_RESPONSE_DESCR)]
    // async fn list_query_responses(&self) -> Result<CallToolResult, Error> {
    //     let schema = schema_for!(AllQueryResponse);
    //     let serialized: String = serde_json::to_string(&schema).unwrap_or("".to_string());
    //     Ok(CallToolResult::success(vec![Content::text(serialized)]))
    // }

    /// Build a query that can be broadcast by an RPC connected wallet
    #[tool(description = BUILD_QUERY_MSG_DESCR)]
    async fn build_query_msg(
        &self,
        #[tool(param)]
        #[schemars(
            description = "address of the deployed contract (e.g. mainnet or testnet address)"
        )]
        contract_addr: String,
        #[tool(param)]
        #[schemars(
            description = "JSON stringified QueryMsg variant needed for building the query as a Cosmos SDK QueryRequest"
        )]
        query_msg: String,
    ) -> Result<CallToolResult, Error> {
        let deserialized: QueryMsg = serde_json::from_str(query_msg.as_str()).unwrap();
        let query_req: QueryRequest<QueryMsg> = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr,
            msg: to_json_binary(&deserialized).unwrap_or_default(),
        });
        let serialized_query_req = serde_json::to_string(&query_req);
        if serialized_query_req.is_err() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Error wrapping QueryMsg as QueryRequest",
            )]));
        }
        let valid_query = ValidatedQuery {
            query_msg,
            query_request: serialized_query_req.unwrap_or_default(),
        };
        let serialized: String = serde_json::to_string(&valid_query).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(serialized)]))
    }

    /// List execute entry points
    #[tool(description = LIST_TX_ENTRY_POINTS_DESCR)]
    async fn list_tx_entry_points(&self) -> Result<CallToolResult, Error> {
        let schema = schema_for!(ExecuteMsg);
        let serialized: String = serde_json::to_string(&schema).unwrap_or("".to_string());
        Ok(CallToolResult::success(vec![Content::text(serialized)]))
    }

    /// Build a transaction that can be signed and broadcast by an RPC connected wallet
    #[tool(description = BUILD_EXECUTE_MSG_DESCR)]
    async fn build_execute_msg(
        &self,
        #[tool(param)]
        #[schemars(
            description = "address of the deployed contract (e.g. mainnet or testnet address)"
        )]
        contract_addr: String,
        #[tool(param)]
        #[schemars(
            description = "ExecuteMsg variant and its values needed for building the transaction as a Cosmos SDK CosmosMsg"
        )]
        execute_msg: String,
        #[tool(param)]
        #[schemars(
            description = "Optionally include native payment funds to be sent in the transaction (required for any transactions that require native denom payments; e.g. not cw20 payments)"
        )]
        payment: Option<String>,
        #[tool(param)]
        #[schemars(
            description = "Optionally include native payment denom for funds being sent in the transaction (required for any transactions that require native denom payments; e.g. not cw20 payments)"
        )]
        payment_denom: Option<String>,
    ) -> Result<CallToolResult, Error> {
        let funds: Vec<Coin> = if payment.is_some() && payment_denom.is_some() {
            let funds = Coin {
                denom: payment_denom.unwrap_or_default(),
                amount: Uint128::from_str(payment.unwrap_or_default().as_str()).unwrap_or_default(),
            };
            vec![funds]
        } else {
            vec![]
        };
        let deserialized: ExecuteMsg = serde_json::from_str(execute_msg.as_str()).unwrap();
        let cosmos_msg: CosmosMsg = WasmMsg::Execute {
            contract_addr,
            msg: to_json_binary(&deserialized).unwrap_or_default(),
            funds,
        }
        .into();
        let serialized_cosmos_msg = serde_json::to_string(&cosmos_msg);
        if serialized_cosmos_msg.is_err() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Error wrapping ExecuteMsg as CosmosMsg",
            )]));
        }
        let valid_execute = ValidatedExecute {
            execute_msg,
            cosmos_msg: serialized_cosmos_msg.unwrap_or_default(),
        };
        let serialized: String = serde_json::to_string(&valid_execute).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(serialized)]))
    }
}

impl Default for CwMcp {
    fn default() -> Self {
        Self::new()
    }
}

#[tool(tool_box)]
impl ServerHandler for CwMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(SERVER_INFO_DESCR.to_string()),
        }
    }
}
