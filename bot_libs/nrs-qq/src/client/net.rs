use crate::provider::*;
use crate::Client;
use alloc::sync::Arc;
use bytes::BytesMut;
use futures::StreamExt;
use nrq_engine::RQError;
use core::sync::atomic::Ordering;
use no_std_net::{Ipv4Addr, SocketAddr};
use futures::SinkExt;

use super::ext::fuse::FusedSplitStream;
use super::ext::length_delimited::LengthDelimitedCodec;
impl<CP, OSCP, RP, MP, TP, TCP> Client<CP, OSCP, RP, MP, TP, TCP>
where
    CP: ChannelProvider,
    OSCP: OneShotChannelProvider,
    RP: RwLockProvider,
    TP: TaskProvider,
    TCP: TcpStreamProvider,
    MP: MutexProvider,
     
{
    pub fn get_address(&self) -> SocketAddr {
        // TODO 选择最快地址
        SocketAddr::new(Ipv4Addr::new(42, 81, 176, 211).into(), 443)
    }


}
impl<CP, OSCP, RP, MP, TP, TCP> Client<CP, OSCP, RP, MP, TP, TCP>
where
    CP: ChannelProvider + 'static,
    OSCP: OneShotChannelProvider + 'static,
    RP: RwLockProvider + 'static,
    TP: TaskProvider + 'static,
    TCP: TcpStreamProvider + 'static,
    MP: MutexProvider + 'static,
    
{
    pub fn stop(self: &Arc<Self>) {
        self.running.store(false, Ordering::Relaxed);
        self.disconnect();
    }
    fn disconnect(self:&Arc<Self>) {
        // TODO dispatch disconnect event
        // don't unwrap (Err means there is no receiver.)
        let my = self.clone();
        TP::spawn(async move {
            my.disconnect_signal.send(()).await.ok()
        });
    }


    // 开始处理流数据
    pub async fn start<S: AsyncRead + AsyncWrite>(self: &Arc<Self>, stream: S) {
        self.running.store(true, Ordering::Relaxed);
        self.net_loop(stream).await; // 阻塞到断开
        self.disconnect();
    }

    async fn net_loop<S: AsyncRead + AsyncWrite>(self: &Arc<Self>, stream: S) {
        log::info!("net loop");
        let (mut write_half, read_half) = LengthDelimitedCodec::builder()
            .length_field_length(4)
            .length_adjustment(-4)
            .new_framed(stream)
            .split();
        let cli = self.clone();
        let mut rx = self.out_pkt_sender.subscribe();
        let mut disconnect_signal = self.disconnect_signal.subscribe();
        let mut read_half = FusedSplitStream(read_half);
        log::info!("channel build success");
        loop {
            log::info!("in event loop");
            futures::select_biased! {
                input = read_half.next() => {
                    if let Some(Ok(mut input)) = input {
                        if let Ok(pkt)=cli.engine.read().await.transport.decode_packet(&mut input){
                            cli.process_income_packet(pkt).await;
                        }else {
                            break;
                        }
                    }else {
                        break;
                    }
                }
                output = rx.recv() => {
                    if let Ok(output) = output {
                        if write_half.send(output).await.is_err(){
                            break;
                        }
                    }
                }
                _ = disconnect_signal.recv() => {
                    break;
                }
            }
            // {
            //     use futures as __futures_crate;
            //     {
            //       enum __PrivResult<_0,_1,_2, >{
            //         _0(_0),_1(_1),_2(_2),
            //       }
            //       let __select_result = {
            //         log::info!("read_half.next()");
            //         let mut _0 = read_half.next();
            //         log::info!("rx.recv()");
            //         let mut _1 = rx.recv();
            //         log::info!("disconnect_signal.recv()");
            //         let mut _2 = disconnect_signal.recv();
            //         let mut __poll_fn =  |__cx: &mut __futures_crate::task::Context< '_> |{
            //           log::info!("poll_fn()");
            //           let mut __any_polled = false;
            //           let mut _0 =  |__cx: &mut __futures_crate::task::Context< '_> |{
            //             let mut _0 = unsafe {
            //               core::pin::Pin::new_unchecked(&mut _0)
            //             };
            //             if __futures_crate::future::FusedFuture::is_terminated(&_0){
            //               None
            //             }else {
            //                 log::info!("read_half poll_unpin()");
            //               let res = Some(__futures_crate::future::FutureExt::poll_unpin(&mut _0,__cx,).map(__PrivResult::_0));
            //               log::info!("read_half poll_unpin() success");
            //               res
            //             }
            //           };
            //           let _0: &mut dyn FnMut(&mut __futures_crate::task::Context< '_>) -> Option<__futures_crate::task::Poll<_>>  =  &mut _0;
            //           let mut _1 =  |__cx: &mut __futures_crate::task::Context< '_> |{
            //             let mut _1 = unsafe {
            //               core::pin::Pin::new_unchecked(&mut _1)
            //             };
            //             if __futures_crate::future::FusedFuture::is_terminated(&_1){
            //               None
            //             }else {
            //               log::info!("rx.recv poll_unpin()");
            //               Some(__futures_crate::future::FutureExt::poll_unpin(&mut _1,__cx,).map(__PrivResult::_1))
            //             }
            //           };
            //           let _1: &mut dyn FnMut(&mut __futures_crate::task::Context< '_>) -> Option<__futures_crate::task::Poll<_>>  =  &mut _1;
            //           let mut _2 =  |__cx: &mut __futures_crate::task::Context< '_> |{
            //             let mut _2 = unsafe {
                            
            //               core::pin::Pin::new_unchecked(&mut _2)
            //             };
            //             if __futures_crate::future::FusedFuture::is_terminated(&_2){
            //               None
            //             }else {
            //                 log::info!("disconnect_signal.recv poll_unpin()");
            //             //   Some(__futures_crate::future::FutureExt::poll_unpin(&mut _2,__cx,).map(__PrivResult::_2))
            //                 Some(__futures_crate::task::Poll::<__PrivResult<_,_,Result<(),()>>>::Pending)
            //             }
            //           };
            //           let _2: &mut dyn FnMut(&mut __futures_crate::task::Context< '_>) -> Option<__futures_crate::task::Poll<_>>  =  &mut _2;
            //           let mut __select_arr = [_0,_1,_2];
            //           for poller in&mut __select_arr {
            //             let poller: &mut&mut dyn FnMut(&mut __futures_crate::task::Context< '_>) -> Option<__futures_crate::task::Poll<_>>  = poller;
            //             match poller(__cx){
            //               Some(x@__futures_crate::task::Poll::Ready(_)) => return x,
            //               Some(__futures_crate::task::Poll::Pending) => {
            //                 log::info!("set __any_polled");
            //                 __any_polled = true;
                            
            //               }
            //               None => {}
                          
                        
            //               }
            //           }if!__any_polled {
            //             // $crate::panicking::panic_fmt(unsafe {
            //             //   std::fmt::Arguments::new_v1(&[], &[])
            //             // })
            //             log::info!("not any polled panic");
            //             __futures_crate::task::Poll::Pending
            //           }else {
            //             log::info!("continue pending");
            //             __futures_crate::task::Poll::Pending
            //           }
            //         };
            //         __futures_crate::future::poll_fn(__poll_fn).await
            //       };
            //       log::info!("to match select");
            //       match __select_result {
            //         __PrivResult::_0(input) => {
            //           {
            //             if let Some(Ok(mut input)) = input {
            //               if let Ok(pkt) = cli.engine.read().await.transport.decode_packet(&mut input){
            //                 cli.process_income_packet(pkt).await;
                            
            //               }else {
            //                 break;
                            
            //               }
            //             }else {
            //               break;
                          
            //             }
            //           }
            //         },
            //         __PrivResult::_1(output) => {
            //           {
            //             if let Ok(output) = output {
            //               if write_half.send(output).await.is_err(){
            //                 break;
                            
            //               }
            //             }
            //           }
            //         },
            //         __PrivResult::_2(_) => {
            //           {
            //             break;
                        
            //           }
            //         },
                  
            //         }
            //     }
            //   }
            log::info!("to next loop");
        }
    }
}
