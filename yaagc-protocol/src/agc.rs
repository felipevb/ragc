pub struct AGCPacket {
    _hw_packet: bool,
    io_addr: usize,
    io_value: u16,
}

impl AGCPacket {
    pub fn new(data: &[u8; 4]) -> Self {
        let res = parse_yaagc_packet(*data);
        match res {
            Some((channel, value)) => AGCPacket {
                io_addr: channel as usize,
                io_value: value,
                _hw_packet: false,
            },
            _ => AGCPacket {
                io_addr: 0x0,
                io_value: 0x0,
                _hw_packet: false,
            },
        }
    }

    pub fn serialize(&self) -> [u8; 4] {
        generate_yaagc_packet(self.io_addr, self.io_value)
    }
}

pub fn generate_yaagc_packet(channel: usize, value: u16) -> [u8; 4] {
    [
        0x0 | ((channel >> 3) & 0x1F) as u8,
        0x40 | ((channel & 0x7) << 3) as u8 | ((value >> 12) & 0x7) as u8,
        0x80 | ((value >> 6) & 0x3F) as u8,
        0xC0 | (value & 0x3F) as u8,
    ]
}

pub fn parse_yaagc_packet(msg: [u8; 4]) -> Option<(u16, u16)> {
    let a;
    let b;
    let c;
    let d;

    match msg.len() {
        4 => {
            a = *msg.get(0).unwrap();
            b = *msg.get(1).unwrap();
            c = *msg.get(2).unwrap();
            d = *msg.get(3).unwrap();
        }
        5 => {
            a = *msg.get(0).unwrap();
            b = *msg.get(1).unwrap();
            c = *msg.get(2).unwrap();
            d = *msg.get(3).unwrap();
        }
        _ => {
            return None;
        }
    }

    if a & 0xC0 != 0x00 || b & 0xC0 != 0x40 || c & 0xC0 != 0x80 || d & 0xC0 != 0xC0 {
        return None;
    }

    let value: u16 = ((b as u16) & 0x7) << 12 | ((c as u16) & 0x3F) << 6 | ((d & 0x3F) as u16);
    let channel: u16 = ((a as u16) & 0x3F) << 3 | ((b as u16) >> 3 & 0x7);
    Some((channel, value))
}
