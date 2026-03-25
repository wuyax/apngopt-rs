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
