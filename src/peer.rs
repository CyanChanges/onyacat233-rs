use crate::layer::Layer;
use crate::socket::Socket;
use async_trait::async_trait;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::io;
use std::net::SocketAddr;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub enum PeerType {
    CSharp = 0x0,
    Client = 0x1,
    Server = 0x2,
}

#[async_trait]
trait Peer {
    async fn send_bytes(&self, buf: &[u8]) -> io::Result<usize>;
    async fn recv_bytes(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Copy)]
pub struct ClientPeer<S: Socket + 'static> {
    layer: &'static Layer<S>,
    pub(crate) id: usize,
    pub(crate) addr: Pin<Box<SocketAddr>>,
    pub(crate) play_genshin_impact: bool,
}

impl<S: Socket> Clone for ClientPeer<S> {
    fn clone(&self) -> Self {
        ClientPeer {
            layer: &self.layer,
            id: self.id,
            addr: Box::from(self.addr),
            play_genshin_impact: self.play_genshin_impact,
        }
    }
}

impl<S: Socket> PartialEq for ClientPeer<S> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<S: Socket> Hash for ClientPeer<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.id)
    }
}

#[async_trait]
impl<S: Socket + Sync> Peer for ClientPeer<S> {
    async fn send_bytes(&self, buf: &[u8]) -> io::Result<usize> {
        self.layer.socket.sendto(buf, *self.get_addr()).await
    }

    async fn recv_bytes(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        todo!()
    }
}

impl<S: Socket> ClientPeer<S> {
    pub fn new(layer: &'static Layer<S>, addr: SocketAddr) -> Self {
        ClientPeer {
            layer,
            id: get_id(),
            play_genshin_impact: true,
            addr: Box::into_pin(Box::new(addr)),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn get_addr(&self) -> &SocketAddr {
        self.addr
    }
    pub fn move_addr(&mut self, addr: SocketAddr) {
        self.addr = &Box::pin(addr)
    }

    pub fn plays_genshin_impact(&self) -> bool {
        self.play_genshin_impact
    }
}

trait _GetPeer<S: Socket> {
    fn peer_of(&self, layer: &Layer<S>) -> ClientPeer<S>;
}

#[macro_export]
macro_rules! peer {
    ($layer:expr, $expr:expr) => {
        $crate::peer::_GetPeer::peer_of($expr, $layer)
    };
}
