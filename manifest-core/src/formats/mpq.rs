use super::mpq_crypt::{decrypt, hash_string};
use super::FormatError;
use std::io::Read;
use std::path::Path;

const FLAG_COMPRESS: u32 = 0x0000_0200;
const FLAG_IMPLODE: u32 = 0x0000_0100;
const FLAG_ENCRYPTED: u32 = 0x0001_0000;
const FLAG_SINGLE_UNIT: u32 = 0x0100_0000;

struct Header {
    header_offset: u64,
    sector_shift: u16,
    hash_table_offset: u32,
    block_table_offset: u32,
    hash_table_entries: u32,
    block_table_entries: u32,
}

fn read_u32(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}
fn read_u16(b: &[u8], at: usize) -> Option<u16> {
    b.get(at..at + 2).map(|s| u16::from_le_bytes(s.try_into().unwrap()))
}

fn find_header(data: &[u8]) -> Result<Header, FormatError> {
    // MPQ headers may start at any 512-byte boundary (SoH writes at 0).
    let mut offset = 0usize;
    while offset + 32 <= data.len() {
        if &data[offset..offset + 4] == b"MPQ\x1a" {
            return Ok(Header {
                header_offset: offset as u64,
                sector_shift: read_u16(data, offset + 14).ok_or_else(corrupt)?,
                hash_table_offset: read_u32(data, offset + 16).ok_or_else(corrupt)?,
                block_table_offset: read_u32(data, offset + 20).ok_or_else(corrupt)?,
                hash_table_entries: read_u32(data, offset + 24).ok_or_else(corrupt)?,
                block_table_entries: read_u32(data, offset + 28).ok_or_else(corrupt)?,
            });
        }
        offset += 512;
        if offset == 512 && data.len() < 512 { break; }
    }
    Err(FormatError::NotAnArchive("no MPQ header".into()))
}

fn corrupt() -> FormatError {
    FormatError::Corrupt("truncated header".into())
}

fn read_table(data: &[u8], base: u64, offset: u32, entries: u32, key_name: &str) -> Result<Vec<[u32; 4]>, FormatError> {
    let start = (base + offset as u64) as usize;
    let len = entries as usize * 16;
    let bytes = data
        .get(start..start + len)
        .ok_or_else(|| FormatError::Corrupt("table out of range".into()))?;
    let mut words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    decrypt(&mut words, hash_string(key_name, 3));
    Ok(words.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect())
}

fn find_block(hash_table: &[[u32; 4]], name: &str) -> Option<u32> {
    let count = hash_table.len() as u32;
    if count == 0 {
        // An empty hash table has no entries to search; treat as "not found"
        // rather than underflowing `count - 1` below.
        return None;
    }
    let start = (hash_string(name, 0) & (count - 1)) as usize;
    let (a, b) = (hash_string(name, 1), hash_string(name, 2));
    let mut i = start;
    loop {
        let entry = hash_table[i];
        if entry[3] == 0xFFFF_FFFF {
            return None; // empty never-used slot terminates the chain
        }
        if entry[0] == a && entry[1] == b && entry[3] != 0xFFFF_FFFE {
            return Some(entry[3]);
        }
        i = (i + 1) % count as usize;
        if i == start {
            return None;
        }
    }
}

fn decompress_sector(sector: &[u8]) -> Result<Vec<u8>, FormatError> {
    match sector.first() {
        Some(0x02) => {
            let mut out = Vec::new();
            flate2::read::ZlibDecoder::new(&sector[1..])
                .read_to_end(&mut out)
                .map_err(|e| FormatError::Corrupt(format!("zlib: {e}")))?;
            Ok(out)
        }
        Some(other) => Err(FormatError::Unlistable(format!(
            "unsupported compression 0x{other:02x}"
        ))),
        None => Ok(Vec::new()),
    }
}

fn read_file(data: &[u8], header: &Header, block: [u32; 4]) -> Result<Vec<u8>, FormatError> {
    let [offset, packed_size, unpacked_size, flags] = block;
    if flags & FLAG_ENCRYPTED != 0 {
        return Err(FormatError::Unlistable("encrypted listfile".into()));
    }
    if flags & FLAG_IMPLODE != 0 {
        return Err(FormatError::Unlistable("imploded listfile".into()));
    }
    let start = (header.header_offset + offset as u64) as usize;
    let raw = data
        .get(start..start + packed_size as usize)
        .ok_or_else(|| FormatError::Corrupt("file data out of range".into()))?;

    let compressed = flags & FLAG_COMPRESS != 0 && packed_size < unpacked_size;
    if flags & FLAG_SINGLE_UNIT != 0 || (!compressed && packed_size == unpacked_size) {
        return if compressed { decompress_sector(raw) } else { Ok(raw.to_vec()) };
    }

    // Multi-sector: leading u32 sector-offset table, one sector per
    // sector_size = 512 << sector_shift bytes of unpacked data. Reject
    // implausible shifts before computing sector_size: on 64-bit targets
    // `512usize << shift` silently wraps to 0 once shift reaches 55 (2^9 *
    // 2^55 == 2^64 == 0 mod 2^64), which would otherwise divide-by-zero
    // below. Cap at a still-generous 4 GiB sector.
    const MAX_SECTOR_SHIFT: u16 = 23;
    if header.sector_shift > MAX_SECTOR_SHIFT {
        return Err(FormatError::Corrupt("implausible sector shift".into()));
    }
    let sector_size = 512usize << header.sector_shift;
    let sector_count = (unpacked_size as usize).div_ceil(sector_size);
    let mut offsets = Vec::with_capacity(sector_count + 1);
    for i in 0..=sector_count {
        offsets.push(read_u32(raw, i * 4).ok_or_else(|| FormatError::Corrupt("sector table".into()))? as usize);
    }
    let mut out = Vec::with_capacity(unpacked_size as usize);
    for w in offsets.windows(2) {
        let sector = raw
            .get(w[0]..w[1])
            .ok_or_else(|| FormatError::Corrupt("sector out of range".into()))?;
        let remaining = (unpacked_size as usize).saturating_sub(out.len());
        if compressed && sector.len() < remaining.min(sector_size) {
            out.extend(decompress_sector(sector)?);
        } else {
            out.extend_from_slice(sector);
        }
    }
    out.truncate(unpacked_size as usize);
    Ok(out)
}

pub fn list_mpq_assets(path: &Path) -> Result<Vec<String>, FormatError> {
    let data = std::fs::read(path).map_err(|e| FormatError::Io(e.to_string()))?;
    let header = find_header(&data)?;
    let hash_table = read_table(&data, header.header_offset, header.hash_table_offset, header.hash_table_entries, "(hash table)")?;
    let block_table = read_table(&data, header.header_offset, header.block_table_offset, header.block_table_entries, "(block table)")?;
    let block_index = find_block(&hash_table, "(listfile)")
        .ok_or_else(|| FormatError::Unlistable("no (listfile)".into()))?;
    let block = *block_table
        .get(block_index as usize)
        .ok_or_else(|| FormatError::Corrupt("listfile block index out of range".into()))?;
    let body = read_file(&data, &header, block)?;
    let text = String::from_utf8_lossy(&body);
    let mut out: Vec<String> = text
        .split(['\r', '\n', ';'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|s| !matches!(*s, "(listfile)" | "(attributes)" | "(signature)"))
        .map(|s| s.replace('\\', "/"))
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::mpq_crypt::encrypt;
    use crate::formats::mpq_fixture::{build_mpq, build_mpq_with_compressed_listfile, build_mpq_with_multisector_listfile};

    #[test]
    fn lists_assets_from_fixture_listfile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fixture.otr");
        std::fs::write(&path, build_mpq(&[
            ("alt/objects/gSwordTex", b"aaaa"),
            ("alt/objects/gShieldTex", b"bb"),
        ])).unwrap();
        let assets = list_mpq_assets(&path).unwrap();
        assert_eq!(assets, vec![
            "alt/objects/gShieldTex".to_string(),
            "alt/objects/gSwordTex".to_string(),
        ]);
    }

    #[test]
    fn non_mpq_file_is_not_an_archive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nope.otr");
        std::fs::write(&path, b"definitely not an mpq").unwrap();
        assert!(matches!(list_mpq_assets(&path), Err(FormatError::NotAnArchive(_))));
    }

    #[test]
    fn mpq_without_listfile_is_unlistable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bare.otr");
        let mut bytes = build_mpq(&[("alt/gA", b"a")]);
        // Corrupt the (listfile) hash entry lookup by renaming trick: rebuild
        // with no files, which still writes a (listfile) - so instead flip the
        // magic of the listfile name hash: overwrite hash table is complex, so
        // simply build with zero files and assert the empty listfile yields
        // an empty asset list, then separately test the missing case by
        // truncating the hash table:
        bytes.truncate(32);
        std::fs::write(&path, bytes).unwrap();
        assert!(list_mpq_assets(&path).is_err());
    }

    #[test]
    fn mpq_missing_listfile_hash_entry_is_unlistable() {
        // Genuine no-(listfile) case: build a normal fixture, then blank out
        // just the (listfile) hash-table entry so the lookup terminates with
        // "not found" instead of the truncated-table corruption above.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no_listfile.otr");
        let mut bytes = build_mpq(&[("alt/gA", b"a")]);

        let header = find_header(&bytes).unwrap();
        let hash_start = (header.header_offset + header.hash_table_offset as u64) as usize;
        let hash_len = header.hash_table_entries as usize * 16;
        let mut words: Vec<u32> = bytes[hash_start..hash_start + hash_len]
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        decrypt(&mut words, hash_string("(hash table)", 3));

        let name_a = hash_string("(listfile)", 1);
        let name_b = hash_string("(listfile)", 2);
        for entry in words.chunks_exact_mut(4) {
            if entry[0] == name_a && entry[1] == name_b {
                entry.copy_from_slice(&[0xFFFF_FFFF; 4]);
            }
        }

        encrypt(&mut words, hash_string("(hash table)", 3));
        for (i, w) in words.iter().enumerate() {
            bytes[hash_start + i * 4..hash_start + i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }

        std::fs::write(&path, bytes).unwrap();
        assert!(matches!(list_mpq_assets(&path), Err(FormatError::Unlistable(_))));
    }

    #[test]
    fn empty_hash_table_is_err_not_panic() {
        // Regression for finding 1: hash_table_entries = 0 must not panic
        // `find_block`'s `count - 1` mask computation (debug: subtract
        // overflow; release: wraps to u32::MAX and indexes out of bounds).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty_hash.otr");
        let mut bytes = build_mpq(&[("alt/gA", b"a")]);
        // hash_table_entries is the u32 at header offset 24.
        bytes[24..28].copy_from_slice(&0u32.to_le_bytes());
        std::fs::write(&path, bytes).unwrap();
        assert!(list_mpq_assets(&path).is_err());
    }

    #[test]
    fn implausible_sector_shift_is_corrupt_not_panic() {
        // Regression for finding 2: a huge sector_shift must not divide by
        // zero in `read_file` (512usize << shift wraps to 0 once shift
        // reaches 55 on a 64-bit target, per LLVM's masked-shift semantics).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad_shift.otr");
        let mut bytes = build_mpq(&[("alt/gA", b"aaaaaa")]);

        // sector_shift is the u16 at header offset 14.
        bytes[14..16].copy_from_slice(&0xFFFFu16.to_le_bytes());

        // Force the (listfile) block off the single-unit fast path so
        // read_file reaches the multi-sector code that derives sector_size
        // from sector_shift. The fixture writes (listfile) as the last
        // block entry with packed_size == unpacked_size and
        // FLAG_SINGLE_UNIT set; clear the flag and make packed != unpacked
        // so neither shortcut condition holds.
        let header = find_header(&bytes).unwrap();
        let block_start = (header.header_offset + header.block_table_offset as u64) as usize;
        let block_len = header.block_table_entries as usize * 16;
        let mut words: Vec<u32> = bytes[block_start..block_start + block_len]
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        decrypt(&mut words, hash_string("(block table)", 3));

        let last = words.len() - 4; // (listfile) is always the last entry
        words[last + 1] -= 1; // packed_size != unpacked_size
        words[last + 3] &= !FLAG_SINGLE_UNIT;

        encrypt(&mut words, hash_string("(block table)", 3));
        for (i, w) in words.iter().enumerate() {
            bytes[block_start + i * 4..block_start + i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }

        std::fs::write(&path, bytes).unwrap();
        assert!(matches!(list_mpq_assets(&path), Err(FormatError::Corrupt(_))));
    }

    #[test]
    fn zlib_compressed_listfile_decodes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("compressed.otr");

        // Enough repetitive names that zlib comfortably shrinks the listfile
        // body below its original size, so the fixture's compressibility
        // assertion (and read_file's packed_size < unpacked_size check)
        // both hold.
        let names: Vec<String> = (0..24)
            .map(|i| format!("alt/objects/repeated_padding_prefix_for_zlib_compression_test/asset_{i:03}"))
            .collect();
        let files: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), b"" as &[u8])).collect();

        std::fs::write(&path, build_mpq_with_compressed_listfile(&files)).unwrap();

        let mut expected = names;
        expected.sort();
        expected.dedup();
        assert_eq!(list_mpq_assets(&path).unwrap(), expected);
    }

    #[test]
    fn multisector_zlib_listfile_decodes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multisector.otr");

        // ~70 bytes per name times 200 names is ~14 KB unpacked, so the
        // listfile spans four 4096-byte sectors. Repetitive prefixes keep
        // every sector (including the short final one) zlib-compressible.
        let names: Vec<String> = (0..200)
            .map(|i| format!("alt/objects/multisector_padding_prefix_shared_by_every_name/asset_{i:04}"))
            .collect();
        let files: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), b"" as &[u8])).collect();

        std::fs::write(&path, build_mpq_with_multisector_listfile(&files)).unwrap();

        let mut expected = names;
        expected.sort();
        expected.dedup();
        assert_eq!(list_mpq_assets(&path).unwrap(), expected);
    }
}
