use std::fs;
use std::error::Error;
use serde::{Deserialize};
use serde_json::{Value, value};
extern crate windows_service;

use std::ffi::OsString;
use std::time::Duration;
use windows_service::{service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, 
    ServiceType, ServiceAccess}, service_manager::{ServiceManager, ServiceManagerAccess}
};
use tokio::{join, runtime::Builder};

use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
#[derive(Deserialize, Debug)]
pub struct Author {
    pub name: String,
    pub age: Option<u32>,
    pub product: Option<Product>,
    pub address: Address,
}

#[derive(Deserialize, Debug)]
pub struct Product {
    pub title: String,
    pub price: i64,
}

#[derive(Deserialize, Debug)]
pub struct Address {
    pub city: String,
    pub street: String,
}
use tokio::runtime::Runtime;
use std::thread;
fn main() -> Result<(), Box<dyn Error>> {
//     let result = get_status_services();
//     println!("result: {:#?}", result);
   //let rt = Runtime::new().unwrap();
   let rt = Builder::new_multi_thread().enable_all().build().unwrap();
   let handle = thread::spawn(move || {
        rt.block_on(async {
            // Simulate a long blocking operation
            println!("Blocking task started");
            let _result = long_computation_7().await;
            let _re = long_computation().await;
            println!("Blocking task completed");
            let (f,s) = tokio::join!(long_computation(), long_computation_7());
            // chỉ dùng trong async context, nó để chờ các future hoàn thành
        });
    }); 
    println!("Main thread continues to run while blocking task is in progress");
    handle.join().unwrap();
    // let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    // rt.block_on(main_task());
    Ok(())
}
async fn main_task() {
    println!("Main task started");
    let result = tokio::join!(quick_task(), quick_task());
    println!("Main task result: {:?}", result);
}

async fn quick_task() -> &'static str {
    println!("Quick task running");
    // A short non-blocking delay
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    "Done"
}
async fn long_computation_7() -> &'static str {
    // Emulates a long computation
    tokio::time::sleep(std::time::Duration::from_secs(7)).await;
    println!("Long computation finished 7s");
    "7"
}
async fn long_computation() -> &'static str {
    // Emulates a long computation
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    println!("Long computation finished 5s");
    "5"
}
fn get_status_services() -> Vec<String> {
    let mut status_services: Vec<String> = Vec::new();
    let manager = match ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT) {
            Ok(manager) => manager,
            Err(_err) => {
                return status_services
            }
        };
    let services = vec!["CMCManager".to_string(), 
    "CMCMngService".to_string(), "CMCEDRLogClr".to_string(), "CMCDrvMngService".to_string(), "CMCUpdaterService".to_string()];
    for service in services.iter(){
        let service_cur  = match manager.open_service(service,
                                         windows_service::service::ServiceAccess::QUERY_STATUS | windows_service::service::ServiceAccess::STOP | 
                                         windows_service::service::ServiceAccess::START) {
            Ok(service) => service,
            Err(_err) => {
                println!("open_service for {} error: {:?}", service , _err);
                continue;
            }
        };
        let status = match service_cur.query_status(){
            Ok(status) => status.current_state,
            Err(_err) => {
                println!("get_status_services for {} error: {:?}", service , _err);
                continue;
            }
        };
        match status {
            ServiceState::Running => {
                println!("Service {} is running", service);
                match service_cur.stop(){
                    Ok(_) => println!("Service {} stop command sent successfully", service),
                    Err(err) => println!("err to stop service {}: {:?}", service, err),
                };
                
            },
            _ => println!("Service {} is in state: {:?}", service, status),
        }
        status_services.push(format!("{:?}",status));
    }
    status_services
}