use ic_cdk::api::call::call;
use candid::Principal;
use candid::{CandidType, Deserialize, Nat};
use ic_cdk_macros::{query, update};

use evm_logs_types::{SubscriptionRegistration, RegisterSubscriptionResult, EventNotification};

#[update]
async fn call_icrc72_register_subscription() {
    let canister_id = Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap();

    let registrations: Vec<SubscriptionRegistration> = vec![SubscriptionRegistration{namespace:"com.example.myapp.events".to_string(), config: vec![], memo:None}]; 

    ic_cdk::println!("Calling icrc72_register_subscription on namespace: {:?}", registrations[0].namespace); 

    let result: Result<(Vec<RegisterSubscriptionResult>,), _> = call(canister_id, "icrc72_register_subscription", (registrations,)).await;
    
    match result {
        Ok((response,)) => {
            ic_cdk::println!("Success: {:?}", response);  
        }
        Err(e) => {
            ic_cdk::println!("Error calling canister: {:?}", e);  
        }
    }
}

#[update]
async fn icrc72_handle_notification(notification: EventNotification) {
    ic_cdk::println!("Received notification for event ID: {:?}", notification.event_id);

    ic_cdk::println!("Notification details: {:?}", notification);
}
