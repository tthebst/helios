use async_trait::async_trait;
use eyre::Result;

use super::ConsensusRpc;
use crate::types::*;

pub struct P2PRpc {
    _rpc: String,
}

impl P2PRpc {
    pub fn new() -> Self {
        P2PRpc {
            _rpc: "hi".to_string(),
        }
    }
}

#[async_trait]
impl ConsensusRpc for P2PRpc {
    async fn get_bootstrap(&self, block_root: &Vec<u8>) -> Result<Bootstrap> {
        !unimplemented!()
    }

    async fn get_updates(&self, period: u64, count: u8) -> Result<Vec<Update>> {
        !unimplemented!()
    }

    async fn get_finality_update(&self) -> Result<FinalityUpdate> {
        !unimplemented!()
    }

    async fn get_optimistic_update(&self) -> Result<OptimisticUpdate> {
        !unimplemented!()
    }

    async fn get_block(&self, slot: u64) -> Result<BeaconBlock> {
        !unimplemented!()
    }
}

// #[derive(serde::Deserialize, Debug)]
// struct BeaconBlockResponse {
//     data: BeaconBlockData,
// }

// #[derive(serde::Deserialize, Debug)]
// struct BeaconBlockData {
//     message: BeaconBlock,
// }

// type UpdateResponse = Vec<UpdateData>;

// #[derive(serde::Deserialize, Debug)]
// struct UpdateData {
//     data: Update,
// }

// #[derive(serde::Deserialize, Debug)]
// struct FinalityUpdateResponse {
//     data: FinalityUpdate,
// }

// #[derive(serde::Deserialize, Debug)]
// struct OptimisticUpdateResponse {
//     data: OptimisticUpdate,
// }

// #[derive(serde::Deserialize, Debug)]
// struct BootstrapResponse {
//     data: Bootstrap,
// }
