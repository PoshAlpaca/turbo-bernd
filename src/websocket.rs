use crate::http::{Method, Request, Response, Status, Version};
use crate::middleware::{self, Middleware};

use std::io::Write;
use std::net::TcpStream;

// The handshake from the client looks as follows:

//         GET /chat HTTP/1.1
//         Host: server.example.com
//         Upgrade: websocket
//         Connection: Upgrade
//         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
//         Origin: http://example.com
//         Sec-WebSocket-Protocol: chat, superchat
//         Sec-WebSocket-Version: 13

//    The handshake from the server looks as follows:

//         HTTP/1.1 101 Switching Protocols
//         Upgrade: websocket
//         Connection: Upgrade
//         Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
//         Sec-WebSocket-Protocol: chat

pub enum WebSocketError {

}

pub enum State {
    Closed,
    Open,
}

pub struct WebSocket {
    pub state: State,
    pub protocol: Vec<String>,
    pub extensions: Vec<String>,
}

impl WebSocket {
    pub fn new() -> Self {
        WebSocket {
            state: State::Closed,
            protocol: Vec::new(),
            extensions: Vec::new(),
        }
    }

    pub fn try_init(req: &Request) -> Response {
        if req.method != Method::Get { return Response::new(Status::MethodNotAllowed) }
        if req.version != Version::OneDotOne { return Response::new(Status::VersionNotSupported) }

        match req.headers.get("Host") {
            Some(s) => println!("{}", s),
            None => return Response::new(Status::BadRequest),
        }

        match req.headers.get("Upgrade") {
            Some(s) => assert!(s.contains("websocket")),
            None => return Response::new(Status::BadRequest),
        }

        match req.headers.get("Connection") {
            Some(s) => assert!(s.contains("Upgrade")),
            None => return Response::new(Status::BadRequest),
        }

        match req.headers.get("Sec-WebSocket-Key") {
            Some(s) => assert_eq!("s", "some Base64-encoded nonce"),
            None => return Response::new(Status::BadRequest),
        }

        match req.headers.get("Sec-WebSocket-Version") {
            Some(s) => {
                if s != "13" {
                    return Response::new(Status::UpgradeRequired).header(("Sec-WebSocket-Version", "13"))
                }
            },
            None => return Response::new(Status::UpgradeRequired).header(("Sec-WebSocket-Version", "13")),
        }

        let mut ws = WebSocket::new();
        ws.protocol = req.headers.get("Sec-WebSocket-Protocol").unwrap().into_array();
        ws.extensions = req.headers.get("Sec-WebSocket-Extensions").unwrap().into_array();

        let res = Response::new(Status::SwitchingProtocols)
            .header(("Upgrade", "websocket"))
            .header(("Connection", "Upgrade"))
            .header(("Sec-WebSocket-Accept", nonce_concated_with_something_else));

        Afterwards:
        - WebSocket Connection Established and in OPEN state
        - Extensions in use: string
    }
}

impl WebSocket {
    pub fn write(&self, stream: &mut TcpStream, bytes: &[u8]) {
        let frame = Frame::new(&[], bytes);

        let _ = stream.write(&frame.as_bytes());
    }
}

pub struct BitVec(Vec<bool>);

impl BitVec {
    pub fn new() -> BitVec {
        BitVec(Vec::new())
    }

    pub fn from_bytes(mut bytes: &[u8]) -> BitVec {
        let mut bit_vec = BitVec::new();

        for byte in bytes {
            let mut current_bit_vec = Self::from_u8(*byte);
            bit_vec.0.append(&mut current_bit_vec.0);
        }

        bit_vec
    }

    pub fn from_u8(mut x: u8) -> BitVec {
        let mut bit_vec: Vec<bool> = vec![false; 8]
            .iter()
            .map(|_| {
                let bit = (x % 2) != 0;
                x >>= 1;
                bit
            })
            .collect();

        bit_vec.reverse();

        BitVec(bit_vec)
    }

    pub fn to_u8(&self) -> u8 {
        assert!(self.0.len() == 8);

        let mut byte = 0;
        for bit in &self.0 {
            byte <<= 1;
            byte += *bit as u8;
        }

        byte
    }

    pub fn to_8bit_arr(&self) -> [bool; 8] {
        assert_eq!(self.0.len(), 8);

        let mut arr = [false; 8];

        for (i, bit) in self.0.iter().enumerate() {
            arr[i] = *bit;
        }

        arr
    }
}

#[derive(Debug)]
pub struct Frame {
    fin: bool,
    rsv1: bool,
    rsv2: bool,
    rsv3: bool,
    opcode: [bool; 4],
    mask: bool,
    payload_length: [bool; 7],
    extended_payload_length_a: Option<u16>,
    extended_payload_length_b: Option<u64>,
    masking_key: Option<[u8; 4]>,
    payload_data: Vec<u8>,
}

impl Frame {
    pub fn from_buffer(buffer: &[u8]) -> Frame {
        let bit_vec = BitVec::from_bytes(&buffer);
        let b = bit_vec.0;

        Frame {
            fin: b[0],
            rsv1: b[1],
            rsv2: b[2],
            rsv3: b[3],
            opcode: [b[4], b[5], b[6], b[7]],
            mask: b[8],
            payload_length: [b[9], b[10], b[11], b[12], b[13], b[14], b[15]],
            extended_payload_length_a: None,
            extended_payload_length_b: None,
            masking_key: None,
            payload_data: Vec::new(),
        }
    }

    pub fn new(ext_data: &[u8], app_data: &[u8]) -> Frame {
        // Calculate length
        let length = ext_data.len() + app_data.len();
        let payload_length: u8;
        let mut extended_payload_length_a = None;
        let mut extended_payload_length_b = None;

        println!("{}", length);

        if length <= 125 {
            payload_length = length as u8;
        } else if length <= u16::MAX as usize {
            payload_length = 126;
            extended_payload_length_a = Some(length as u16);
        } else if length <= u64::MAX as usize {
            payload_length = 127;
            extended_payload_length_b = Some(length as u64);
        } else {
            panic!("Payload data too long, seems like you're sending more than 2^64 bytes in one frame");
        }

        let mut bit_vec = BitVec::from_u8(payload_length);
        println!("{:?}", bit_vec.0);
        bit_vec.0.remove(0);

        assert_eq!(bit_vec.0.len(), 7);

        let mut payload_length_arr = [false; 7];
        for (i, x) in bit_vec.0.iter().enumerate() {
            payload_length_arr[i] = *x;
        }

        Frame {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: [false; 4],
            mask: false,
            payload_length: payload_length_arr,
            extended_payload_length_a: extended_payload_length_a,
            extended_payload_length_b: extended_payload_length_b,
            masking_key: None,
            payload_data: [ext_data, app_data].concat(),
        }
    }

    // pub fn ping(&mut self) -> Frame {
    //     fn u8bool_array
    //     let opcode = 0x9;
    //     self.opcode =
    // }

    // pub fn opcode(&mut self) {
    //     let mask = 0b1111_0000;

    //     let opcode: u16 = 0x0;

    // }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let mut byte_0_arr = BitVec::new();
        byte_0_arr
            .0
            .append(&mut vec![self.fin, self.rsv1, self.rsv2, self.rsv3]);
        byte_0_arr.0.append(&mut self.opcode.to_vec());

        let byte_0 = byte_0_arr.to_u8();
        bytes.push(byte_0);

        let mut byte_1_arr = BitVec::new();
        byte_1_arr.0.push(self.mask);
        byte_1_arr.0.append(&mut self.payload_length.to_vec());

        let byte_1 = byte_1_arr.to_u8();
        bytes.push(byte_1);

        if let Some(extended_payload_length_a) = self.extended_payload_length_a {
            bytes.append(&mut extended_payload_length_a.to_be_bytes().to_vec());
        }

        if let Some(extended_payload_length_b) = self.extended_payload_length_b {
            bytes.append(&mut extended_payload_length_b.to_be_bytes().to_vec());
        }

        if let Some(masking_key) = self.masking_key {
            bytes.append(&mut masking_key.to_vec());
        }

        bytes.append(&mut self.payload_data.to_owned());

        bytes
    }
}

// pub struct WebSocketMiddleware;

// impl Middleware for WebSocketMiddleware {
//     fn answer(&self, req: &Request) -> Result<Response, middleware::Error> {
//         assert_eq!(req.method, Method::Get);
//         assert_eq!(req.version, Version::OneDotOne);
//         assert_eq!(req.headers.get("Host"), Some(_));
//         assert_eq!(req.headers.get("Upgrade"), Some(x) where x contains "websocket");
//         assert_eq!(req.headers.get("Connection"), Some(x) where x contains "Upgrade");
//         assert_eq!(req.headers.get("Sec-WebSocket-Key"), Some(x) where x is a B64-encoded nonce);
//         assert_eq!(req.headers.get("Sec-WebSocket-Version"), Some(x) where x == "13");
//             else:
//             Response::new(Status::UpgradeRequired).header(("Sec-WebSocket-Version", "13"))

//         let mut ws = WebSocket::new();
//         ws.protocol = req.headers.get("Sec-WebSocket-Protocol").unwrap().into_array();
//         ws.extensions = req.headers.get("Sec-WebSocket-Extensions").unwrap().into_array();

//         let res = Response::new(Status::SwitchingProtocols)
//             .header(("Upgrade", "websocket"))
//             .header(("Connection", "Upgrade"))
//             .header(("Sec-WebSocket-Accept", nonce_concated_with_something_else));

//         Afterwards:
//         - WebSocket Connection Established and in OPEN state
//         - Extensions in use: string
//     }
// }

// impl WebSocketMiddleware {
//     pub fn register() {

//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_bytes_works() {
        let data = b"Hello, world!";
        let mut frame = Frame::new(&[], data);
        frame.opcode = [false, false, false, true]; // opcode of 0x1 -> text frame

        println!("{:?}", frame);

        let raw_frame: u16 = 0b1000_0001_0000_1101;

        let mut bytes = Vec::new();
        bytes.append(&mut raw_frame.to_be_bytes().to_vec());
        bytes.append(&mut data.to_vec());

        let bytes_a = frame.as_bytes();
        println!("{:b}", bytes_a[0]);
        println!("{:b}", bytes[0]);
        println!("{:b}", bytes_a[1]);
        println!("{:b}", bytes[1]);
        assert_eq!(bytes_a[0], bytes[0]);
        assert_eq!(bytes_a[1], bytes[1]);
    }

    #[test]
    fn bit_vec_from_u8() {
        let bit_vec = BitVec::from_u8(0b0000_1111);
        assert_eq!(
            bit_vec.0,
            vec![false, false, false, false, true, true, true, true]
        );
    }

    #[test]
    fn bit_vec_to_u8() {
        let bit_vec = BitVec(vec![false, false, false, false, true, true, true, true]);
        assert_eq!(bit_vec.to_u8(), 15);
    }
}
