use crc32fast::Hasher;
use std::io::{self, Read, Write};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub size: u32,
    pub chunk_type: [u8; 4],
    pub data: Vec<u8>,
    pub crc: u32,
}

impl Chunk {
    /// Create a new chunk and compute its CRC32
    pub fn new(chunk_type: [u8; 4], data: Vec<u8>) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(&chunk_type);
        hasher.update(&data);
        let crc = hasher.finalize();

        Self {
            size: data.len() as u32,
            chunk_type,
            data,
            crc,
        }
    }

    /// Write the complete chunk to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.size.to_be_bytes())?;
        writer.write_all(&self.chunk_type)?;
        writer.write_all(&self.data)?;
        writer.write_all(&self.crc.to_be_bytes())?;
        Ok(())
    }
}

/// Read a chunk from a reader and verify its CRC32
#[allow(dead_code)]
pub fn read_chunk<R: Read>(reader: &mut R) -> io::Result<Chunk> {
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let size = u32::from_be_bytes(len_bytes);

    let mut chunk_type = [0u8; 4];
    reader.read_exact(&mut chunk_type)?;

    let mut data = vec![0u8; size as usize];
    reader.read_exact(&mut data)?;

    let mut crc_bytes = [0u8; 4];
    reader.read_exact(&mut crc_bytes)?;
    let crc = u32::from_be_bytes(crc_bytes);

    // Verify CRC
    let mut hasher = Hasher::new();
    hasher.update(&chunk_type);
    hasher.update(&data);
    let expected_crc = hasher.finalize();

    if crc != expected_crc {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "CRC mismatch for chunk type: {:?}",
                std::str::from_utf8(&chunk_type).unwrap_or("UNKNOWN")
            ),
        ));
    }

    Ok(Chunk {
        size,
        chunk_type,
        data,
        crc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunk_creation_and_write() {
        let chunk_type = *b"IHDR";
        let data = vec![0, 0, 0, 10, 0, 0, 0, 10, 8, 6, 0, 0, 0];
        let chunk = Chunk::new(chunk_type, data.clone());

        assert_eq!(chunk.size, 13);
        assert_eq!(chunk.chunk_type, chunk_type);
        assert_eq!(chunk.data, data);

        let mut buf = Vec::new();
        chunk.write(&mut buf).unwrap();

        assert_eq!(buf.len(), 4 + 4 + 13 + 4); // size + type + data + crc
        assert_eq!(&buf[0..4], &13u32.to_be_bytes());
        assert_eq!(&buf[4..8], &chunk_type);
        assert_eq!(&buf[8..21], data.as_slice());
        assert_eq!(&buf[21..25], &chunk.crc.to_be_bytes());
    }

    #[test]
    fn test_read_chunk_success() {
        let chunk_type = *b"tEXt";
        let data = b"Hello".to_vec();
        let chunk = Chunk::new(chunk_type, data);

        let mut buf = Vec::new();
        chunk.write(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let read_back = read_chunk(&mut cursor).unwrap();

        assert_eq!(read_back.size, chunk.size);
        assert_eq!(read_back.chunk_type, chunk.chunk_type);
        assert_eq!(read_back.data, chunk.data);
        assert_eq!(read_back.crc, chunk.crc);
    }

    #[test]
    fn test_read_chunk_invalid_crc() {
        let chunk_type = *b"tEXt";
        let data = b"Hello".to_vec();
        let chunk = Chunk::new(chunk_type, data);

        let mut buf = Vec::new();
        chunk.write(&mut buf).unwrap();

        // Corrupt the data to invalidate CRC
        buf[10] = b'M';

        let mut cursor = Cursor::new(buf);
        let result = read_chunk(&mut cursor);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    }
}
