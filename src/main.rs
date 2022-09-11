use pretty_hex::PrettyHex;
use std::ffi::c_void;

extern "stdcall" {
    fn LoadLibraryA(name: *const u8) -> HModule;
    fn GetProcAddress(module: HModule, name: *const u8) -> FarProc;
}

type Handle = *const c_void;
type HModule = *const c_void;
type FarProc = *const c_void;
type IcmpCreateFile = extern "stdcall" fn() -> Handle;
type IcmpSendEcho = extern "stdcall" fn(
    handle: Handle,
    dest: IPAddr,
    request_data: *const u8,
    request_size: u16,
    request_options: Option<&IpOptionInformation>,
    reply_buffer: *mut u8,
    reply_size: u32,
    timeout: u32,
) -> u32;

#[repr(C)]
#[derive(Debug)]
struct IpOptionInformation {
    ttl: u8,
    tos: u8,
    flags: u8,
    options_size: u8,
    options_data: u32, // Actually a 32-bit pointer.
}

#[repr(C)]
#[derive(Debug)]
struct IcmpEchoReply {
    address: IPAddr,
    status: u32,
    rtt: u32,
    data_size: u16,
    reserved: u16,
    data: *const u8,
    options: IpOptionInformation,
}

struct IPAddr([u8; 4]);

impl std::fmt::Debug for IPAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b, c, d] = self.0;
        write!(f, "{a}.{b}.{c}.{d}")
    }
}

fn main() {
    #[allow(non_snake_case)]
    unsafe {
        let h = LoadLibraryA("IPHLPAPI.dll\0".as_ptr());
        let IcmpCreateFile: IcmpCreateFile =
            std::mem::transmute(GetProcAddress(h, "IcmpCreateFile\0".as_ptr()));
        let IcmpSendEcho: IcmpSendEcho =
            std::mem::transmute(GetProcAddress(h, "IcmpSendEcho\0".as_ptr()));

        let handle = IcmpCreateFile();

        let data = "O Romeo, Romeo. Reachable art thou Romeo?";
        let reply_size = std::mem::size_of::<IcmpEchoReply>();
        let reply_buf_size = reply_size + 8 + data.len();
        let mut reply_buf = vec![0u8; reply_buf_size];

        let ip_opts = IpOptionInformation {
            ttl: 128,
            tos: 0,
            flags: 0,
            options_size: 0,
            options_data: 0,
        };

        let ret = IcmpSendEcho(
            handle,
            IPAddr([8, 8, 8, 8]),
            data.as_ptr(),
            data.len() as u16,
            Some(&ip_opts),
            reply_buf.as_mut_ptr(),
            reply_buf_size as u32,
            4000,
        );

        if ret == 0 {
            panic!("IcmpSendEcho failed! ret = {ret}");
        }

        let reply: &IcmpEchoReply = std::mem::transmute(&reply_buf[0]);
        println!("{:#?}", *reply);

        let reply_data: *const u8 = std::mem::transmute(&reply_buf[reply_size + 8]);
        let reply_data = std::slice::from_raw_parts(reply_data, reply.data_size as usize);
        println!("{:?}", reply_data.hex_dump());
    }
}
