use std::sync::OnceLock;

pub fn crypt_table() -> &'static [u32; 1280] {
    static TABLE: OnceLock<[u32; 1280]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u32; 1280];
        let mut seed: u32 = 0x0010_0001;
        for index1 in 0..256u32 {
            let mut index2 = index1 as usize;
            for _ in 0..5 {
                seed = (seed.wrapping_mul(125).wrapping_add(3)) % 0x2AAAAB;
                let temp1 = (seed & 0xFFFF) << 16;
                seed = (seed.wrapping_mul(125).wrapping_add(3)) % 0x2AAAAB;
                let temp2 = seed & 0xFFFF;
                table[index2] = temp1 | temp2;
                index2 += 256;
            }
        }
        table
    })
}

pub fn hash_string(s: &str, hash_type: u32) -> u32 {
    let table = crypt_table();
    let mut seed1: u32 = 0x7FED_7FED;
    let mut seed2: u32 = 0xEEEE_EEEE;
    for ch in s.chars() {
        let ch = if ch == '/' { '\\' } else { ch };
        let ch = (ch.to_ascii_uppercase() as u32) & 0xFF;
        seed1 = table[(hash_type * 256 + ch) as usize] ^ seed1.wrapping_add(seed2);
        seed2 = ch
            .wrapping_add(seed1)
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
    }
    seed1
}

pub fn decrypt(data: &mut [u32], key: u32) {
    let table = crypt_table();
    let mut seed1 = key;
    let mut seed2: u32 = 0xEEEE_EEEE;
    for value in data.iter_mut() {
        seed2 = seed2.wrapping_add(table[(0x400 + (seed1 & 0xFF)) as usize]);
        let decrypted = *value ^ seed1.wrapping_add(seed2);
        seed1 = ((!seed1 << 0x15).wrapping_add(0x1111_1111)) | (seed1 >> 0x0B);
        seed2 = decrypted
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
        *value = decrypted;
    }
}

pub fn encrypt(data: &mut [u32], key: u32) {
    let table = crypt_table();
    let mut seed1 = key;
    let mut seed2: u32 = 0xEEEE_EEEE;
    for value in data.iter_mut() {
        seed2 = seed2.wrapping_add(table[(0x400 + (seed1 & 0xFF)) as usize]);
        let plain = *value;
        *value = plain ^ seed1.wrapping_add(seed2);
        seed1 = ((!seed1 << 0x15).wrapping_add(0x1111_1111)) | (seed1 >> 0x0B);
        seed2 = plain
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_string_matches_known_storm_values() {
        // Reference values computed with mpyq's _hash() (verified 2026-07-30):
        // hash_string is case-insensitive (uppercases) and treats '/' as '\\'.
        assert_eq!(hash_string("(hash table)", 3), 0xC3AF3770);
        assert_eq!(hash_string("(block table)", 3), 0xEC83B3A3);
    }

    #[test]
    fn hash_string_non_ascii_does_not_panic() {
        // Regression for the merge-gate finding: `char as u32` for chars
        // >= U+0200 previously indexed `table[hash_type * 256 + ch]` out of
        // bounds once ch exceeded 255, since to_ascii_uppercase() is a no-op
        // on non-ASCII input. Masking to a byte (matching StormLib's
        // byte-oriented treatment) must keep every hash_type in range.
        for hash_type in 0..4u32 {
            let _ = hash_string("Ā☃𐍈", hash_type);
        }
        // Known ASCII value must be unaffected by the byte mask.
        assert_eq!(hash_string("(hash table)", 3), 0xC3AF3770);
    }

    #[test]
    fn encrypt_then_decrypt_round_trips() {
        let key = hash_string("(listfile)", 3);
        let original: Vec<u32> = (0..64u32).map(|i| i.wrapping_mul(2654435761u32)).collect();
        let mut data = original.clone();
        encrypt(&mut data, key);
        assert_ne!(data, original);
        decrypt(&mut data, key);
        assert_eq!(data, original);
    }
}
