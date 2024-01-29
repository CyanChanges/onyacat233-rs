use crate::back_to_enum;
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fmt::Formatter;
use std::future::Future;
use std::net::SocketAddr;
use tokio::net::UdpSocket as tokioUDPSocket;

pub const BASIC_PACKAGE_SIZE: usize = 3;

back_to_enum! {
    #[derive(Clone, Debug)]
    #[repr(u8)]
    pub enum PackageType {
        Handshake = 0x0,
        WaveHand = 0x1,
        _Reserved1 = 0x2,
        Userdata = 0x3,
        PeerConnected = 0x4,
        PeerDisconnected = 0x5,
        Heartbeat = 0x7,
        Timeout = 0x8,
        BadPackage = 0x9,
        ServiceTemporaryUnavailable = 0xA,
        Sign = 0xB,
        JoinNetwork = 0xC,
        LeaveNetwork = 0xD,
        PeerUpdate = 0xE,
    }
}

#[derive(Clone)]
pub struct Package {
    pack_type: PackageType,
    data: Vec<u8>,
}

#[macro_export]
macro_rules! pack {
    ($pack_type:expr) => {Package::new::<1>($pack_type, None)};
    ($pack_type:expr, $($x:expr),+ $(,)?) => {Package::new($pack_type, [$($x), +])}
}

pub trait _AsPack {
    fn to_pack_as_data(self, package_type: PackageType) -> Package;
    fn as_pack(&self) -> Package;
}

impl _AsPack for Vec<u8> {
    #[inline]
    fn to_pack_as_data(mut self, package_type: PackageType) -> Package {
        Package::from_data(package_type, &self)
    }

    #[inline]
    fn as_pack(&self) -> Package {
        Package::from(self).unwrap()
    }
}

impl<const S: usize> _AsPack for [u8; S] {
    #[inline]
    fn to_pack_as_data(self, package_type: PackageType) -> Package {
        Package::from_data(package_type, &Vec::from(self))
    }

    #[inline]
    fn as_pack(&self) -> Package {
        Package::from(self).unwrap()
    }
}

impl _AsPack for &[u8] {
    fn to_pack_as_data(self, package_type: PackageType) -> Package {
        Package::from_data(package_type, &Vec::from(self))
    }

    fn as_pack(&self) -> Package {
        Package::from(self).unwrap()
    }
}

#[macro_export]
macro_rules! as_pack {
    ($pack_type:expr, $d:expr) => {
        $crate::package::_AsPack::to_pack_as_data($d, $pack_type)
    };
    ($buf:expr) => {
        $crate::package::_AsPack::as_pack(&$buf)
    };
}

impl Package {
    #[inline]
    pub fn new<const S: usize>(package_type: PackageType, data: Option<[u8; S]>) -> Package {
        Package {
            pack_type: package_type,
            data: Vec::from(data.unwrap_or([0u8; S])),
        }
    }

    #[inline]
    pub fn from_data(package_type: PackageType, data: &[u8]) -> Package {
        Package {
            pack_type: package_type,
            data: (*data).to_owned(),
        }
    }

    pub fn from(data: &[u8]) -> Result<Package, String> {
        assert!(data.len() >= BASIC_PACKAGE_SIZE, "invalid package");

        if let Some((tail_byte, pack)) = data.split_last() {
            assert_eq!(tail_byte, &0xff, "invalid package trailing");

            let (type_byte, pack_data) = pack.split_first().unwrap();
            let pack_type =
                PackageType::try_from(type_byte.to_owned() as i32).expect("invalid package type");

            Ok(Package::from_data(pack_type, pack_data))
        } else {
            Err("invalid package".to_string())
        }
    }

    pub fn encode(&self) -> VecDeque<u8> {
        let mut vec = VecDeque::with_capacity(self.data.len() + 2);
        vec.push_back(self.pack_type.to_owned() as u8);
        vec.extend(self.data.to_owned());
        vec.push_back(0xffu8);

        vec
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<Package {:?}:{:?}>", self.pack_type, self.data)
    }
}
