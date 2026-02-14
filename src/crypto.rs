//! Cryptographic utilities for CLR metadata.
//!
//! Contains a minimal SHA-1 implementation for public key token computation.

/// Compute SHA-1 hash of data (minimal implementation).
///
/// Used for computing public key tokens from public keys.
#[must_use]
pub fn sha1(data: &[u8]) -> [u8; 20] {
    // Initialize hash values (FIPS 180-1)
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    // Pre-processing: add padding bits
    let ml = (data.len() as u64) * 8; // message length in bits
    let mut padded = data.to_vec();
    padded.push(0x80); // append bit '1' to message

    // Pad to 448 bits mod 512 (56 bytes mod 64)
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }

    // Append original length in bits as 64-bit big-endian
    padded.extend_from_slice(&ml.to_be_bytes());

    // Process each 512-bit (64-byte) chunk
    for chunk in padded.chunks(64) {
        let mut w = [0u32; 80];

        // Break chunk into sixteen 32-bit big-endian words
        for (i, word_bytes) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word_bytes[0], word_bytes[1], word_bytes[2], word_bytes[3]]);
        }

        // Extend the sixteen 32-bit words into eighty 32-bit words
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        // Initialize working variables
        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);

        // Main loop
        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        // Add this chunk's hash to result
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    // Produce the final hash value (big-endian)
    let mut result = [0u8; 20];
    result[0..4].copy_from_slice(&h0.to_be_bytes());
    result[4..8].copy_from_slice(&h1.to_be_bytes());
    result[8..12].copy_from_slice(&h2.to_be_bytes());
    result[12..16].copy_from_slice(&h3.to_be_bytes());
    result[16..20].copy_from_slice(&h4.to_be_bytes());
    result
}

/// Compute the public key token from a public key.
///
/// The public key token is the last 8 bytes of the SHA-1 hash, reversed.
#[must_use]
pub fn public_key_token(public_key: &[u8]) -> [u8; 8] {
    let hash = sha1(public_key);
    // Take last 8 bytes and reverse
    let mut token = [0u8; 8];
    for i in 0..8 {
        token[i] = hash[19 - i];
    }
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_empty() {
        // SHA-1 of empty string = da39a3ee5e6b4b0d3255bfef95601890afd80709
        let hash = sha1(b"");
        assert_eq!(
            hash,
            [
                0xda, 0x39, 0xa3, 0xee, 0x5e, 0x6b, 0x4b, 0x0d, 0x32, 0x55, 0xbf, 0xef, 0x95, 0x60,
                0x18, 0x90, 0xaf, 0xd8, 0x07, 0x09
            ]
        );
    }

    #[test]
    fn test_sha1_abc() {
        // SHA-1 of "abc" = a9993e364706816aba3e25717850c26c9cd0d89d
        let hash = sha1(b"abc");
        assert_eq!(
            hash,
            [
                0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e, 0x25, 0x71, 0x78, 0x50,
                0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d
            ]
        );
    }

    #[test]
    fn test_public_key_token() {
        // Test public key token extraction logic:
        // SHA-1 of "abc" = a9993e364706816aba3e25717850c26c9cd0d89d
        // Last 8 bytes: 7850c26c9cd0d89d
        // Reversed: 9dd8d09c6cc25078
        let hash = sha1(b"abc");
        let token = public_key_token(b"abc");

        // Verify the hash is correct
        assert_eq!(
            hash,
            [
                0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e, 0x25, 0x71, 0x78, 0x50,
                0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d
            ]
        );

        // Last 8 bytes of hash: 78 50 c2 6c 9c d0 d8 9d
        // Reversed: 9d d8 d0 9c 6c c2 50 78
        assert_eq!(token, [0x9d, 0xd8, 0xd0, 0x9c, 0x6c, 0xc2, 0x50, 0x78]);
    }
}

