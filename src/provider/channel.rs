use std::sync::Arc;

use futures::{Future, FutureExt};
use nrs_qq::provider::{ChannelProvider,TSender,TReceiver};
use broadcaster::BroadcastChannel;
use futures::StreamExt;
use futures::channel::mpsc::{Sender,Receiver,channel};
pub struct MyChannelProvider;

#[derive(Clone)]
pub struct MySender<T:'static + Send + Clone>(BroadcastChannel<T,Sender<T>,Receiver<T>>);
#[derive(Clone)]
pub struct MyReceiver<T:'static +Send+Clone>(BroadcastChannel<T,Sender<T>,Receiver<T>>);


impl<T:Send + Clone> TReceiver<T> for MyReceiver<T> {
    type Error = ();

    type RecvFuture<'a>  = impl Future<Output = Result<T,Self::Error>> + futures::future::FusedFuture + Send + 'a where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move {
            self.0.next().await.ok_or(())
        }.fuse()
    }
}

impl<T:'static + Send + Clone + Sync> TSender<T> for MySender<T> {
    type Error = ();
    type SenderFuture<'a>  = impl Future<Output = Result<usize,Self::Error>> + Send + 'a where Self: 'a;
    type Receiver = MyReceiver<T>;

    fn send(&self,value:T) -> Self::SenderFuture<'_> {

        Box::pin(async move {
            let s = self.0.send(&value);
            drop(self);
            s.await.map(|_|0usize).map_err(|_|())
        })
    }

    fn is_closed(&self) -> bool {
        todo!()
    }

    fn close_channel(&self) -> () {
        todo!()
    }

    fn subscribe(&self) -> Self::Receiver {
        MyReceiver(self.0.clone())
    }
}

impl ChannelProvider for MyChannelProvider {
    type Sender<T:'static + Clone + Send + Sync> = MySender<T>;

    type Receiver<T:'static +Clone +Send + Sync> = MyReceiver<T>;

    fn channel<T:'static +Clone + Send + Sync>(buff:usize) -> (Self::Sender<T>,Self::Receiver<T>) {
        let s = BroadcastChannel::with_ctor(Arc::new(move||{
            channel::<T>(buff)
        }));
        (MySender(s.clone()),MyReceiver(s.clone()))
    }
}
