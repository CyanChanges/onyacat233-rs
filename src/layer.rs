use crate::package::Package;
use crate::peer::{ClientPeer, PeerType};
use crate::socket::Socket;
use crate::syscall;
use crate::util::socket_addr;
use async_trait::async_trait;
use fxhash::FxHashMap;
use nix::sys::socket::sockopt::ReuseAddr;
use nohash::IntMap;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::hash::{BuildHasher, BuildHasherDefault};
use std::net::{SocketAddr, ToSocketAddrs};
use std::os::fd::{AsFd, AsRawFd};
use std::{io, net};
use tokio::net::UdpSocket;

type FuncType<S> = fn(package: Package, peer: ClientPeer<S>) -> dyn Future<Output = ()>;

#[derive(Debug)]
enum HandlerTypeEnum {
    Handshake,
    Disconnect,
}

#[derive(Debug)]
pub struct Layer<S: Socket + 'static> {
    pub(crate) receivers: IntMap<usize, Vec<FuncType<S>>>,
    pub(crate) handlers: HashMap<HandlerTypeEnum, Vec<FuncType<S>>>,
    pub(crate) buffers: FxHashMap<SocketAddr, Box<VecDeque<u8>>>,
    pub(crate) _type: PeerType,
    pub(crate) socket: Box<S>,
}

pub struct UdpLayer {
    layer: Layer<UdpSocket>,
}

#[async_trait]
impl Socket for UdpLayer {
    #[inline]
    async fn sendto(&self, buf: &[u8], target: SocketAddr) -> io::Result<usize> {
        self.layer.socket.sendto(buf, target).await
    }

    #[inline]
    async fn recvfrom(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.layer.socket.recvfrom(buf).await
    }
}

#[async_trait]
trait Handler {
    fn _data<const S: usize>(&mut self, data: [u8; S], addr: SocketAddr);
    fn _handle(&self, vec: VecDeque<u8>, addr: SocketAddr);
}

#[async_trait]
trait Type {
    fn get_type(&self) -> &PeerType;
}

#[async_trait]
pub trait Receiver<S: Socket> {
    async fn on_package(&mut self, peer: ClientPeer<S>, func: FuncType<S>) -> ();
}

#[async_trait]
pub trait Packager<S: Socket> {
    async fn send_package(&self, package: &Package, peer: &ClientPeer<S>) -> usize;
}

#[async_trait]
impl<SO: Socket> Handler for Layer<SO> {
    fn _data<const S: usize>(&mut self, data: [u8; S], addr: SocketAddr) {
        let mut binding = self.buffers.get_mut::<SocketAddr>(&addr);
        let mut new_vec = Box::from(VecDeque::new());
        let vec = binding.get_or_insert(&mut new_vec);
        vec.extend(data);
    }

    fn _handle(&self, vec: VecDeque<u8>, addr: SocketAddr) {
        todo!()
    }
}

#[async_trait]
impl Receiver<UdpSocket> for UdpLayer {
    async fn on_package(&mut self, peer: ClientPeer<UdpSocket>, func: FuncType<UdpSocket>) -> () {
        let mut binding = Vec::new();
        let mut vec = self
            .layer
            .receivers
            .get_mut(&peer.get_id())
            .unwrap_or(&mut binding)
            .clone();
        vec.push(func);
        self.layer.receivers.insert(peer.get_id(), vec.to_vec());
    }
}

#[async_trait]
impl Packager<UdpSocket> for UdpLayer {
    async fn send_package(&self, package: &Package, peer: &ClientPeer<UdpSocket>) -> usize {
        let bytes = package.encode();
        self.layer
            .socket
            .send_to(bytes.as_slices().0, peer.get_addr())
            .await
            .expect("failed to send the package")
    }
}

impl<S: Socket> Type for Layer<S> {
    fn get_type(&self) -> &PeerType {
        &self._type
    }
}

pub trait UdpServer: Packager<UdpSocket> + Receiver<UdpSocket> {
    fn new() -> Self;
    fn bind<A: ToSocketAddrs>(addr: A) -> Self;
    fn bind_new<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()>;
}

impl UdpServer for UdpLayer {
    fn new() -> Self {
        UdpLayer::bind("0.0.0.0:0")
    }

    fn bind<A: ToSocketAddrs>(addr: A) -> Self {
        UdpLayer {
            layer: Layer {
                socket: Box::from({
                    let std_socket = net::UdpSocket::bind(addr).unwrap();
                    nix::sys::socket::setsockopt(&std_socket.as_fd(), ReuseAddr, &true)
                        .expect("cannot reuseAddr");
                    UdpSocket::from_std(std_socket).unwrap()
                }),
                handlers: HashMap::new(),
                receivers: IntMap::default(),
                buffers: FxHashMap::default(),
                _type: PeerType::Server,
            },
        }
    }

    fn bind_new<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        let addresses = ToSocketAddrs::to_socket_addrs(&addr).unwrap();
        let fd = self.layer.socket.as_raw_fd();

        for address in addresses {
            let (raw_addr, raw_addr_length) = socket_addr(&address);

            syscall!(bind(fd, raw_addr.as_ptr(), raw_addr_length))
        }
        Ok(())
    }
}
