use bincode::{Decode, Encode};
use std::path::PathBuf;

#[derive(Encode, Decode, Debug)]
pub enum SyncMessage {
    MakeDir {
        relative_path: PathBuf,
    },
    BeginFile {
        relative_path: PathBuf,
        file_size: u64,
    },
    FileChunk {
        data: Vec<u8>,
    },
    EndFile,
}

//universal bincode config init
fn conf() -> impl bincode::config::Config {
    bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding()
}

impl SyncMessage {
    pub fn encode(&self) -> Vec<u8> {
        let data = bincode::encode_to_vec(self, conf()).expect("Failed to encode");

        return data;
    }

    pub fn decode(buf: &[u8]) -> SyncMessage {
        let (msg, _): (SyncMessage, usize) =
            bincode::borrow_decode_from_slice(buf, conf()).expect("Failed to decode");

        return msg;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_encode_decode_makedir() {
        let original = SyncMessage::MakeDir {
            relative_path: PathBuf::from("some/path"),
        };

        let encoded = original.encode();
        let decoded = SyncMessage::decode(&encoded);

        if let SyncMessage::MakeDir { relative_path } = decoded {
            assert_eq!(relative_path, PathBuf::from("some/path"));
        } else {
            panic!("Decoded variant does not match MakeDir");
        }
    }

    #[test]
    fn test_encode_decode_beginfile() {
        let original = SyncMessage::BeginFile {
            relative_path: PathBuf::from("file.txt"),
            file_size: 123456,
        };

        let encoded = original.encode();
        let decoded = SyncMessage::decode(&encoded);

        if let SyncMessage::BeginFile {
            relative_path,
            file_size,
        } = decoded
        {
            assert_eq!(relative_path, PathBuf::from("file.txt"));
            assert_eq!(file_size, 123456);
        } else {
            panic!("Decoded variant does not match BeginFile");
        }
    }

    #[test]
    fn test_encode_decode_filechunk() {
        let data = vec![1, 2, 3, 4, 5];
        let original = SyncMessage::FileChunk { data: data.clone() };

        let encoded = original.encode();
        let decoded = SyncMessage::decode(&encoded);

        if let SyncMessage::FileChunk { data: decoded_data } = decoded {
            assert_eq!(decoded_data, data);
        } else {
            panic!("Decoded variant does not match FileChunk");
        }
    }

    #[test]
    fn test_encode_decode_endfile() {
        let original = SyncMessage::EndFile;

        let encoded = original.encode();
        let decoded = SyncMessage::decode(&encoded);

        matches!(decoded, SyncMessage::EndFile); // this will panic if not EndFile
    }
}
