use super::mpq_crypt::{encrypt, hash_string};

const FLAG_EXISTS: u32 = 0x8000_0000;
const FLAG_SINGLE_UNIT: u32 = 0x0100_0000;
const FLAG_COMPRESS: u32 = 0x0000_0200;

pub fn build_mpq(files: &[(&str, &[u8])]) -> Vec<u8> {
    build_mpq_inner(files, false)
}

/// Like [`build_mpq`], but the `(listfile)` entry is written zlib-compressed:
/// compression type byte `0x02` followed by a deflate stream, `FLAG_COMPRESS`
/// set on the block entry, and `packed_size < unpacked_size`. Lets tests
/// exercise the zlib decompression path in `read_file`/`decompress_sector`.
pub fn build_mpq_with_compressed_listfile(files: &[(&str, &[u8])]) -> Vec<u8> {
    build_mpq_inner(files, true)
}

struct Entry {
    name: String,
    packed: Vec<u8>,
    unpacked_size: u32,
    flags: u32,
}

fn build_mpq_inner(files: &[(&str, &[u8])], compress_listfile: bool) -> Vec<u8> {
    let listfile_body: String = files
        .iter()
        .map(|(n, _)| n.replace('/', "\\"))
        .collect::<Vec<_>>()
        .join("\r\n");
    let listfile_bytes = listfile_body.into_bytes();

    let mut entries: Vec<Entry> = files
        .iter()
        .map(|(n, d)| Entry {
            name: n.to_string(),
            packed: d.to_vec(),
            unpacked_size: d.len() as u32,
            flags: FLAG_EXISTS | FLAG_SINGLE_UNIT,
        })
        .collect();

    if compress_listfile {
        use std::io::Write;
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&listfile_bytes).unwrap();
        let compressed = encoder.finish().unwrap();
        let mut packed = Vec::with_capacity(compressed.len() + 1);
        packed.push(0x02);
        packed.extend_from_slice(&compressed);
        assert!(
            packed.len() < listfile_bytes.len(),
            "fixture listfile body not compressible enough for this test"
        );
        entries.push(Entry {
            name: "(listfile)".to_string(),
            packed,
            unpacked_size: listfile_bytes.len() as u32,
            flags: FLAG_EXISTS | FLAG_SINGLE_UNIT | FLAG_COMPRESS,
        });
    } else {
        entries.push(Entry {
            name: "(listfile)".to_string(),
            unpacked_size: listfile_bytes.len() as u32,
            packed: listfile_bytes,
            flags: FLAG_EXISTS | FLAG_SINGLE_UNIT,
        });
    }

    let header_size: u32 = 32;
    let mut file_data = Vec::new();
    let mut block_entries: Vec<[u32; 4]> = Vec::new();
    for e in &entries {
        let offset = header_size + file_data.len() as u32;
        file_data.extend_from_slice(&e.packed);
        block_entries.push([offset, e.packed.len() as u32, e.unpacked_size, e.flags]);
    }

    let hash_entries_count = (entries.len().next_power_of_two().max(4)) as u32;
    let mut hash_table: Vec<[u32; 4]> =
        vec![[0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF]; hash_entries_count as usize];
    for (block_index, e) in entries.iter().enumerate() {
        let start = (hash_string(&e.name, 0) & (hash_entries_count - 1)) as usize;
        let mut i = start;
        loop {
            if hash_table[i][3] == 0xFFFF_FFFF {
                hash_table[i] = [hash_string(&e.name, 1), hash_string(&e.name, 2), 0, block_index as u32];
                break;
            }
            i = (i + 1) % hash_entries_count as usize;
            assert_ne!(i, start, "hash table full");
        }
    }

    let hash_table_offset = header_size + file_data.len() as u32;
    let block_table_offset = hash_table_offset + hash_entries_count * 16;
    let archive_size = block_table_offset + block_entries.len() as u32 * 16;

    let mut out = Vec::new();
    out.extend_from_slice(b"MPQ\x1a");
    out.extend_from_slice(&header_size.to_le_bytes());
    out.extend_from_slice(&archive_size.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // format version 1
    out.extend_from_slice(&3u16.to_le_bytes()); // sector size shift (unused: single unit)
    out.extend_from_slice(&hash_table_offset.to_le_bytes());
    out.extend_from_slice(&block_table_offset.to_le_bytes());
    out.extend_from_slice(&hash_entries_count.to_le_bytes());
    out.extend_from_slice(&(block_entries.len() as u32).to_le_bytes());
    out.extend_from_slice(&file_data);

    let mut hash_words: Vec<u32> = hash_table.iter().flatten().copied().collect();
    encrypt(&mut hash_words, hash_string("(hash table)", 3));
    for w in &hash_words {
        out.extend_from_slice(&w.to_le_bytes());
    }
    let mut block_words: Vec<u32> = block_entries.iter().flatten().copied().collect();
    encrypt(&mut block_words, hash_string("(block table)", 3));
    for w in &block_words {
        out.extend_from_slice(&w.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_starts_with_mpq_magic_and_contains_data() {
        let bytes = build_mpq(&[("alt/objects/gA", b"aaaa" as &[u8]), ("alt/objects/gB", b"bb" as &[u8])]);
        assert_eq!(&bytes[0..4], b"MPQ\x1a");
        assert!(bytes.len() > 32);
    }
}
