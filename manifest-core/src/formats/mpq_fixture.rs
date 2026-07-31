use super::mpq_crypt::{encrypt, hash_string};

const FLAG_EXISTS: u32 = 0x8000_0000;
const FLAG_SINGLE_UNIT: u32 = 0x0100_0000;

pub fn build_mpq(files: &[(&str, &[u8])]) -> Vec<u8> {
    let listfile_body: String = files
        .iter()
        .map(|(n, _)| n.replace('/', "\\"))
        .collect::<Vec<_>>()
        .join("\r\n");
    let mut all: Vec<(String, Vec<u8>)> = files
        .iter()
        .map(|(n, d)| (n.to_string(), d.to_vec()))
        .collect();
    all.push(("(listfile)".to_string(), listfile_body.into_bytes()));

    let header_size: u32 = 32;
    let mut file_data = Vec::new();
    let mut block_entries: Vec<[u32; 4]> = Vec::new();
    for (_, data) in &all {
        let offset = header_size + file_data.len() as u32;
        file_data.extend_from_slice(data);
        block_entries.push([offset, data.len() as u32, data.len() as u32, FLAG_EXISTS | FLAG_SINGLE_UNIT]);
    }

    let hash_entries_count = (all.len().next_power_of_two().max(4)) as u32;
    let mut hash_table: Vec<[u32; 4]> =
        vec![[0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF]; hash_entries_count as usize];
    for (block_index, (name, _)) in all.iter().enumerate() {
        let start = (hash_string(name, 0) & (hash_entries_count - 1)) as usize;
        let mut i = start;
        loop {
            if hash_table[i][3] == 0xFFFF_FFFF {
                hash_table[i] = [hash_string(name, 1), hash_string(name, 2), 0, block_index as u32];
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
