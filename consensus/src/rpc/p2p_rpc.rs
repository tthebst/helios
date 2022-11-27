use async_trait::async_trait;
use common::types::Bytes32;
use config::networks::Network;
use discv5::{
    enr,
    enr::{CombinedKey, NodeId},
    Discv5, Discv5ConfigBuilder, Enr,
};
use eyre::Result;
use libp2p::{
    swarm::{ConnectionHandler, NetworkBehaviour, NetworkBehaviourAction},
    PeerId,
};
use std::{net::SocketAddr, task::Poll};
use tokio::runtime::Handle;

use super::ConsensusRpc;
use crate::types::*;

// https://github.com/sigp/lighthouse/blob/stable/common/eth2_network_config/built_in_network_configs/mainnet/boot_enr.yaml
const MAINNET_BOOT_NODES: [&str;2]= [
    "enr:-Jq4QItoFUuug_n_qbYbU0OY04-np2wT8rUCauOOXNi0H3BWbDj-zbfZb7otA7jZ6flbBpx1LNZK2TDebZ9dEKx84LYBhGV0aDKQtTA_KgEAAAD__________4JpZIJ2NIJpcISsaa0ZiXNlY3AyNTZrMaEDHAD2JKYevx89W0CcFJFiskdcEzkH_Wdv9iW42qLK79ODdWRwgiMo",
    "enr:-Jq4QN_YBsUOqQsty1OGvYv48PMaiEt1AzGD1NkYQHaxZoTyVGqMYXg0K9c0LPNWC9pkXmggApp8nygYLsQwScwAgfgBhGV0aDKQtTA_KgEAAAD__________4JpZIJ2NIJpcISLosQxiXNlY3AyNTZrMaEDBJj7_dLFACaxBfaI8KZTh_SSJUjhyAyfshimvSqo22WDdWRwgiMo"
];
// https://github.com/sigp/lighthouse/blob/stable/common/eth2_network_config/built_in_network_configs/prater/boot_enr.yaml
const GOERLI_BOOT_NODES: [&str;2]= [
    "enr:-LK4QH1xnjotgXwg25IDPjrqRGFnH1ScgNHA3dv1Z8xHCp4uP3N3Jjl_aYv_WIxQRdwZvSukzbwspXZ7JjpldyeVDzMCh2F0dG5ldHOIAAAAAAAAAACEZXRoMpB53wQoAAAQIP__________gmlkgnY0gmlwhIe1te-Jc2VjcDI1NmsxoQOkcGXqbCJYbcClZ3z5f6NWhX_1YPFRYRRWQpJjwSHpVIN0Y3CCIyiDdWRwgiMo",
    "enr:-L64QJmwSDtaHVgGiqIxJWUtxWg6uLCipsms6j-8BdsOJfTWAs7CLF9HJnVqFE728O-JYUDCxzKvRdeMqBSauHVCMdaCAVWHYXR0bmV0c4j__________4RldGgykIL0pysCABAghLYBAAAAAACCaWSCdjSCaXCEQWxOdolzZWNwMjU2azGhA7Qmod9fK86WidPOzLsn5_8QyzL7ZcJ1Reca7RnD54vuiHN5bmNuZXRzD4N0Y3CCIyiDdWRwgiMo"
];

struct Discovery {
    discv5: Discv5,
}

impl Discovery {
    pub fn new(rt: Handle, network: Network) -> Self {
        let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();

        // construct a local ENR
        let enr_key = CombinedKey::generate_secp256k1();
        let enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();

        // default configuration
        let config = Discv5ConfigBuilder::new().build();

        // construct the discv5 server
        let mut discv5 = Discv5::new(enr, enr_key, config).unwrap();

        // Add bootstrap nodes.
        match network {
            Network::MAINNET => MAINNET_BOOT_NODES,
            Network::GOERLI => GOERLI_BOOT_NODES,
        }
        .into_iter()
        .for_each(|e| discv5.add_enr(e.parse().unwrap()).unwrap());

        rt.block_on(discv5.start(listen_addr)).unwrap();

        // start the discv5 server
        Self { discv5 }
    }
}

#[derive(Debug)]
pub struct DiscoveredPeers {
    peers: Vec<PeerId>,
}

impl NetworkBehaviour for Discovery {
    type ConnectionHandler = libp2p::swarm::handler::DummyConnectionHandler;
    type OutEvent = DiscoveredPeers;
    fn new_handler(&mut self) -> Self::ConnectionHandler {
        libp2p::swarm::handler::DummyConnectionHandler::default()
    }
    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection: libp2p::core::connection::ConnectionId,
        event: <Self::ConnectionHandler as ConnectionHandler>::OutEvent,
    ) {
    }
    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
        params: &mut impl libp2p::swarm::PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        Poll::Pending
    }
}

struct EthNetwork {
    discovery: Discovery,
}

impl EthNetwork {
    pub fn new(rt: Handle, network: Network) -> Self {
        Self {
            discovery: Discovery::new(rt, network),
        }
    }
}

pub struct P2PRpc {
    network: EthNetwork,
}

impl P2PRpc {
    pub fn new(network: Network, rt: Handle) -> Self {
        P2PRpc {
            // TODO: Pass runtime.
            network: EthNetwork::new(rt, network),
        }
    }
}

#[async_trait]
impl ConsensusRpc for P2PRpc {
    async fn get_bootstrap(&self, _block_root: &Vec<u8>) -> Result<Bootstrap> {
        Ok(Bootstrap {
            header: Header::default(),
            current_sync_committee: SyncCommittee::default(),
            current_sync_committee_branch: Vec::new(),
        })
    }

    async fn get_updates(&self, _period: u64, _count: u8) -> Result<Vec<Update>> {
        Ok(Vec::new())
    }

    async fn get_finality_update(&self) -> Result<FinalityUpdate> {
        Ok(FinalityUpdate {
            attested_header: Header::default(),
            finalized_header: Header::default(),
            finality_branch: Vec::new(),
            sync_aggregate: SyncAggregate::default(),
            signature_slot: 0,
        })
    }

    async fn get_optimistic_update(&self) -> Result<OptimisticUpdate> {
        Ok(OptimisticUpdate {
            attested_header: Header::default(),
            sync_aggregate: SyncAggregate::default(),
            signature_slot: 0,
        })
    }

    async fn get_block(&self, _slot: u64) -> Result<BeaconBlock> {
        Ok(BeaconBlock {
            slot: 0,
            proposer_index: 0,
            parent_root: Bytes32::default(),
            state_root: Bytes32::default(),
            body: BeaconBlockBody::default(),
        })
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
