//! Адрес этого компьютера в локальной сети — для ссылки зрителям.

use std::net::{IpAddr, Ipv4Addr};

use windows::Win32::Foundation::ERROR_BUFFER_OVERFLOW;
use windows::Win32::NetworkManagement::IpHelper::{
    GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
    GAA_FLAG_SKIP_MULTICAST, GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows::Win32::Networking::WinSock::{AF_INET, SOCKADDR_IN};

/// Выбирает адрес в домашней сети.
///
/// Главный признак — у адаптера есть шлюз по умолчанию: у виртуальных
/// адаптеров VMware/Hyper-V его нет. VPN обычно выдаёт адрес не из частных
/// диапазонов (100.x, 26.x…), такие отбрасываются сразу.
pub fn lan_ip() -> Option<IpAddr> {
    ipv4_adapters()
        .into_iter()
        .filter_map(|adapter| {
            let rank = private_rank(adapter.ip)?;
            Some(((!adapter.has_gateway, rank), adapter.ip))
        })
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, ip)| IpAddr::V4(ip))
}

/// Чем меньше, тем «домашнее»: 192.168.x.x, потом 172.16–31.x.x, потом 10.x.x.x.
fn private_rank(ip: Ipv4Addr) -> Option<u8> {
    match ip.octets() {
        [192, 168, ..] => Some(0),
        [172, 16..=31, ..] => Some(1),
        [10, ..] => Some(2),
        _ => None,
    }
}

struct Adapter {
    ip: Ipv4Addr,
    has_gateway: bool,
}

/// IPv4-адреса всех включённых сетевых адаптеров.
fn ipv4_adapters() -> Vec<Adapter> {
    let flags = GAA_FLAG_INCLUDE_GATEWAYS
        | GAA_FLAG_SKIP_ANYCAST
        | GAA_FLAG_SKIP_MULTICAST
        | GAA_FLAG_SKIP_DNS_SERVER;
    // Буфер из u64, чтобы структуры в нём были правильно выровнены.
    let mut size: u32 = 16 * 1024;
    let mut buf: Vec<u64> = Vec::new();
    loop {
        buf.resize(size as usize / 8 + 1, 0);
        let result = unsafe {
            GetAdaptersAddresses(
                u32::from(AF_INET.0),
                flags,
                None,
                Some(buf.as_mut_ptr().cast()),
                &mut size,
            )
        };
        match result {
            0 => break,
            code if code == ERROR_BUFFER_OVERFLOW.0 => continue,
            _ => return Vec::new(),
        }
    }

    let mut adapters = Vec::new();
    let mut current = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
    while let Some(adapter) = unsafe { current.as_ref() } {
        if adapter.OperStatus == IfOperStatusUp {
            let has_gateway = !adapter.FirstGatewayAddress.is_null();
            let mut unicast = adapter.FirstUnicastAddress;
            while let Some(address) = unsafe { unicast.as_ref() } {
                let sockaddr = address.Address.lpSockaddr;
                if let Some(sockaddr) = unsafe { sockaddr.as_ref() }
                    && sockaddr.sa_family == AF_INET
                {
                    let sin = unsafe { &*(sockaddr as *const _ as *const SOCKADDR_IN) };
                    let raw = unsafe { sin.sin_addr.S_un.S_addr };
                    adapters.push(Adapter {
                        ip: Ipv4Addr::from(u32::from_be(raw)),
                        has_gateway,
                    });
                }
                unicast = address.Next;
            }
        }
        current = adapter.Next;
    }
    adapters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_network_wins() {
        assert_eq!(private_rank(Ipv4Addr::new(192, 168, 1, 42)), Some(0));
        assert_eq!(private_rank(Ipv4Addr::new(172, 20, 0, 1)), Some(1));
        assert_eq!(private_rank(Ipv4Addr::new(10, 8, 0, 2)), Some(2));
        assert_eq!(private_rank(Ipv4Addr::new(100, 67, 255, 61)), None);
        assert_eq!(private_rank(Ipv4Addr::new(172, 32, 0, 1)), None);
    }
}
