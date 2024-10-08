use candid::{CandidType, Deserialize, Nat};
use ic_cdk::api::call::{call, call_with_payment128};
use candid::Principal; // Імпорт для Principal
use std::cell::RefCell;
use std::io::{self, Write};
use candid::Encode;
use ic_cdk::api::time;
use ic_cdk_timers::TimerId;
use std::time::Duration;
use std::rc::Rc;
// use evm_rpc_types::{self, Nat256};


use evm_rpc_canister_types::{
    BlockTag, EthMainnetService, GetLogsArgs, EvmRpcCanister, GetBlockByNumberResult, GetLogsResult, HttpOutcallError, MultiGetBlockByNumberResult, MultiGetLogsResult, RejectionCode, RpcError, RpcServices, EVM_RPC
};


pub struct ChainService {
    canister_id: String, 
    evm_rpc: EvmRpcCanister,
    last_checked_time: RefCell<u64>,
    timer_id: RefCell<Option<TimerId>>,
}

impl ChainService {
    pub fn new(canister_id: String) -> Self {
        let principal = Principal::from_text("bd3sg-teaaa-aaaaa-qaaba-cai").unwrap();
        let evm_rpc = EvmRpcCanister(principal);
        let last_checked_time = RefCell::new(time() / 1_000_000); 
        let timer_id = RefCell::new(None);

        ChainService {
            canister_id,
            evm_rpc,
            last_checked_time,
            timer_id,
        }
    }

    pub async fn fetch_logs(&self, from_block: u64, to_block: u64, address: Option<String>) -> Result<Vec<String>, String> {


        let rpc_providers = RpcServices::EthMainnet(Some(vec![EthMainnetService::Alchemy]));


        let get_logs_args = GetLogsArgs {
            fromBlock: Some(BlockTag::Number(Nat::from(from_block))),
            toBlock: Some(BlockTag::Number(Nat::from(to_block))),
            addresses: address
            .into_iter()
            // .map(|addr| addr.parse::<Hex20>().expect("Invalid address format"))     // TODO
            .collect(),

            topics: None,
        };

        let cycles = 10_000_000_000;
        let (result,) = self.evm_rpc
            .eth_get_logs(rpc_providers, None, get_logs_args, cycles)
            .await
            .expect("Call failed");

        match result {
            MultiGetLogsResult::Consistent(r) => match r {
                GetLogsResult::Ok(block) => {
                    let log_strings: Vec<String> = block.into_iter().map(|log_entry| {
                        format!(
                            "Address: {}, TxHash: {:?}, Block: {:?}, Topics: {:?}, Data: {}",
                            log_entry.address,
                            log_entry.transactionHash,
                            log_entry.blockNumber,
                            log_entry.topics,
                            log_entry.data
                            
                        )
                    }).collect();
                    Ok(log_strings)
                },
                GetLogsResult::Err(err) => Err(format!("{:?}", err)),
            },
            MultiGetLogsResult::Inconsistent(_) => {
                Err("RPC providers gave inconsistent results".to_string())
            }
        }
        
    }


    pub fn start_monitoring(&self, interval: Duration) {
        let self_clone = Rc::new(self.clone()); 
        
        let timer_id = ic_cdk_timers::set_timer_interval(interval, move || {
            let self_clone = Rc::clone(&self_clone);
            let current_time = time() / 1_000_000;
            if *self_clone.last_checked_time.borrow() < current_time {
                ic_cdk::spawn(async move {
                    self_clone.fetch_logs_and_update_time().await;
                });
            }
        });

        *self.timer_id.borrow_mut() = Some(timer_id);
    }
    

    async fn fetch_logs_and_update_time(&self) {

        match self.fetch_logs(20798658, 20798660, Some("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852".to_string())).await {
            Ok(logs) => {
                if !logs.is_empty() {
                    *self.last_checked_time.borrow_mut() = time() / 1_000_000;
                    for log in logs {
                        ic_cdk::println!("Log: {}", log); 
                    }
                }
            },
            Err(e) => {
                ic_cdk::println!("Error during logs extraction: {}", e);
            }
        }
    }

    fn clone(&self) -> Self {
        ChainService {
            canister_id: self.canister_id.clone(),
            evm_rpc: self.evm_rpc.clone(),
            last_checked_time: RefCell::new(*self.last_checked_time.borrow()),
            timer_id: RefCell::new(*self.timer_id.borrow()),
        }
    }

}


