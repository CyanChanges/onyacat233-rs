use async_trait::async_trait;
use std::io;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[async_trait]
pub trait Socket {
    async fn sendto(&self, buf: &[u8], target: SocketAddr) -> io::Result<usize>;
    async fn recvfrom(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;
}

#[async_trait]
impl Socket for UdpSocket {
    async fn sendto(&self, buf: &[u8], target: SocketAddr) -> io::Result<usize> {
        UdpSocket::send_to(self, buf, target).await
    }

    async fn recvfrom(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        UdpSocket::recv_from(self, buf).await
    }
}
