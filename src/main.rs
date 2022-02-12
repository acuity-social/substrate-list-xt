
use subxt::{
    ClientBuilder,
    DefaultConfig,
    DefaultExtra,
    BlockNumber,
};

use sp_core::storage::StorageKey;
use sp_core::twox_128;
use core::str::FromStr;
use codec::Decode;

#[subxt::subxt(runtime_metadata_path = "src/polkadot_metadata.scale")]
pub mod polkadot {}

struct SystemEvents(StorageKey);

impl SystemEvents {
    pub(crate) fn new() -> Self {
        let mut storage_key = twox_128(b"System").to_vec();
        storage_key.extend(twox_128(b"Events").to_vec());
        Self(StorageKey(storage_key))
    }
}

impl From<SystemEvents> for StorageKey {
    fn from(key: SystemEvents) -> Self {
        key.0
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .build()
        .await?;

    let _api = client.clone().to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>>();

    let mut block_number = 0;

    loop {
        if block_number % 1000 == 0 {
            println!("block_number: {:?}", block_number);
        }
        let block_hash = client.rpc().block_hash(Some(BlockNumber::from(block_number))).await.unwrap();

        let raw_events = client
            .rpc()
            .storage(
                &StorageKey::from(SystemEvents::new()),
                Some(block_hash.unwrap()),
            )
            .await?
            .map(|s| s.0)
            .unwrap_or_else(Vec::new);

        let events = client
            .events_decoder()
            .decode_events(&mut &*raw_events);

        match events {
            Ok(events) => {
                for event in events {
                    if event.1.pallet == "Balances" && event.1.variant == "Transfer" {
                        let decoded = polkadot::balances::events::Transfer::decode(&mut &event.1.data[..]).unwrap();
                        if decoded.0 == subxt::sp_runtime::AccountId32::from_str("5DV3QBwhs7djHop5WEcUQ1BMFSmsqvqnDhrYh343x495AL4v").unwrap() {
                            println!("block_number: {:?}", block_number);
                            println!("from: {}", decoded.0);
                            println!("to: {}", decoded.1);
                            println!("value: {}", decoded.2);
                        }
                    }
                }
            }
            Err(_) => {}
        }

        block_number += 1;
    }
}
