use crate::layer::{Receiver, UdpLayer, UdpServer};
use crate::package::{Package, PackageType};
use tokio::io;

mod layer;
mod package;
mod peer;
mod socket;
mod util;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut server = UdpLayer::bind("0.0.0.0:5100");
    let package = pack!(PackageType::Handshake);
    println!("{:?}", package.encode());
    println!("{}", package);
    println!("{}", as_pack!([0x0, 0x0, 0xff]));
    println!("{}", as_pack!(PackageType::Handshake, vec!(0x0)));

    Ok(())
}
