//packet header

#[derive(Clone, Copy)]
pub enum PacketType {
    Dir { path: PathBuf},
    FileStart { path: PathBuf, size: u64 },
    FileChunk {data: Vec<u8>},
    EndFile,
}

impl TryFrom<u8> for PacketType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PacketType::Sync),
            _ => Err("Invalid Packet Type"),
        }
    }
}

pub struct PacketHeader {
    pub packet_type: PacketType,
    pub body_size: u16,
    pub seq_num: u8,
}

impl PacketHeader {
    pub fn init() -> PacketHeader {
        return PacketHeader {
            packet_type: PacketType::Sync,
            body_size: 0,
            seq_num: 0,
        };
    }

    pub fn serialize_header(&self) -> Vec<u8> {
        let mut serial: Vec<u8> = Vec::new();
        serial.push(self.packet_type as u8);
        serial.extend_from_slice(&self.body_size.to_be_bytes());
        serial.push(self.seq_num);
        return serial;
    }

    pub fn deserialize_header(serial: Vec<u8>) -> Result<PacketHeader, &'static str> {
        let header = PacketHeader {
            packet_type: PacketType::try_from(serial[0])?,
            body_size: u16::from_be_bytes([serial[1], serial[2]]),
            seq_num: serial[3],
        };

        Ok(header)
    }
}

// Packet body

pub struct Packet {
    pub header: PacketHeader,
    pub body: Vec<u8>,
}

impl Packet {
    pub fn init() -> Packet {
        return Packet {
            header: PacketHeader::init(),
            body: Vec::new(),
        };
    }
}

pub fn serialize_packet(pkt: Packet) -> Vec<u8> {
    let mut serial: Vec<u8> = Vec::new();
    serial.extend(pkt.header.serialize_header());
    serial.extend_from_slice(&pkt.body);

    return serial;
}