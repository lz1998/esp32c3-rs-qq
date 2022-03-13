use core::sync::atomic::Ordering;
use alloc::sync::Arc;

use crate::Client;
use crate::provider::*;

pub async fn after_login<CP, OSCP, RP, MP, TP, TCP>(client:&Arc<Client<CP, OSCP, RP, MP, TP, TCP>>) 
where
    CP: ChannelProvider + 'static,
    OSCP: OneShotChannelProvider+ 'static,
    RP: RwLockProvider+ 'static,
    TP: TaskProvider+ 'static,
    TCP: TcpStreamProvider+ 'static,
    MP: MutexProvider+ 'static,
    
{
    log::info!("register_client()");
    if let Err(err) = client.register_client().await {
        tracing::error!("failed to register client: {}", err)
    }
    // 为了能成功发出信息直接舍弃心跳和刷新状态
    // log::info!("start_heartbeat()");
    // start_heartbeat(client.clone()).await;
    // log::info!("refresh_status()");
    // if let Err(err) = client.refresh_status().await {
    //     tracing::error!("failed to refresh status: {}", err)
    // }
    log::info!("after_login() done");
}

pub async fn start_heartbeat<CP, OSCP, RP, MP, TP, TCP>(client:Arc<Client<CP, OSCP, RP, MP, TP, TCP>>) 
where
    CP: ChannelProvider+ 'static,
    OSCP: OneShotChannelProvider+ 'static,
    RP: RwLockProvider+ 'static,
    TP: TaskProvider+ 'static,
    TCP: TcpStreamProvider+ 'static,
    MP: MutexProvider+ 'static,
    
{
    if !client.heartbeat_enabled.load(Ordering::Relaxed) {
        // tokio::spawn(async move {
        //     client.do_heartbeat().await;
        // });
        TP::spawn(async move {
            client.do_heartbeat().await;
        });
    }
}