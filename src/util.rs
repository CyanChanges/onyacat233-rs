use nix::libc;
use std::mem::size_of;
use std::net::SocketAddr;
#[macro_export]
macro_rules! back_to_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<i32> for $name {
            type Error = ();

            fn try_from(v: i32) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as i32 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}

#[repr(C)]
pub union SocketAddrCRepr {
    v4: libc::sockaddr_in,
    v6: libc::sockaddr_in6,
}

impl SocketAddrCRepr {
    pub(crate) fn as_ptr(&self) -> *const libc::sockaddr {
        self as *const _ as *const libc::sockaddr
    }
}

#[macro_export]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        use nix::libc;
        let res = unsafe { libc::$fn($($arg, )*) };
        let _ = if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        };
    }};
}

pub(crate) fn socket_addr(addr: &SocketAddr) -> (SocketAddrCRepr, libc::socklen_t) {
    match addr {
        SocketAddr::V4(ref addr) => {
            // `s_addr` is stored as BE on all machine and the array is in BE order.
            // So the native endian conversion method is used so that it's never swapped.
            let sin_addr = libc::in_addr {
                s_addr: u32::from_ne_bytes(addr.ip().octets()),
            };

            let sockaddr_in = libc::sockaddr_in {
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port: addr.port().to_be(),
                sin_addr,
                #[cfg(not(target_os = "vita"))]
                sin_zero: [0; 8],
                #[cfg(target_os = "vita")]
                sin_zero: [0; 6],
                #[cfg(any(
                    target_os = "aix",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "ios",
                    target_os = "macos",
                    target_os = "netbsd",
                    target_os = "openbsd",
                    target_os = "tvos",
                    target_os = "watchos",
                    target_os = "espidf",
                    target_os = "vita",
                ))]
                sin_len: 0,
                #[cfg(target_os = "vita")]
                sin_vport: addr.port().to_be(),
            };

            let sockaddr = SocketAddrCRepr { v4: sockaddr_in };
            let socklen = size_of::<libc::sockaddr_in>() as libc::socklen_t;
            (sockaddr, socklen)
        }
        SocketAddr::V6(ref addr) => {
            let sockaddr_in6 = libc::sockaddr_in6 {
                sin6_family: libc::AF_INET6 as libc::sa_family_t,
                sin6_port: addr.port().to_be(),
                sin6_addr: libc::in6_addr {
                    s6_addr: addr.ip().octets(),
                },
                sin6_flowinfo: addr.flowinfo(),
                sin6_scope_id: addr.scope_id(),
                #[cfg(any(
                    target_os = "aix",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "ios",
                    target_os = "macos",
                    target_os = "netbsd",
                    target_os = "openbsd",
                    target_os = "tvos",
                    target_os = "watchos",
                    target_os = "espidf",
                    target_os = "vita",
                ))]
                sin6_len: 0,
                #[cfg(target_os = "vita")]
                sin6_vport: addr.port().to_be(),
                #[cfg(any(target_os = "illumos", target_os = "solaris"))]
                __sin6_src_id: 0,
            };

            let sockaddr = SocketAddrCRepr { v6: sockaddr_in6 };
            let socklen = size_of::<libc::sockaddr_in6>() as libc::socklen_t;
            (sockaddr, socklen)
        }
    }
}
