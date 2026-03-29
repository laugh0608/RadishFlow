use std::fmt::Write;

use rf_types::{RfError, RfResult};

use crate::StoredPropertyPackagePayload;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPropertyPackagePayloadIntegrity {
    pub hash: String,
    pub size_bytes: u64,
}

pub fn property_package_payload_integrity(
    payload: &StoredPropertyPackagePayload,
) -> RfResult<StoredPropertyPackagePayloadIntegrity> {
    payload.validate()?;

    let canonical_json = crate::json::property_package_payload_to_pretty_json(payload)?;
    let bytes = canonical_json.as_bytes();

    Ok(StoredPropertyPackagePayloadIntegrity {
        hash: format!("sha256:{}", sha256_hex(bytes)),
        size_bytes: bytes.len() as u64,
    })
}

pub fn validate_property_package_payload_integrity(
    payload: &StoredPropertyPackagePayload,
    expected_hash: &str,
    expected_size_bytes: u64,
) -> RfResult<()> {
    let actual = property_package_payload_integrity(payload)?;

    if !expected_hash.trim().is_empty() && actual.hash != expected_hash {
        return Err(RfError::invalid_input(format!(
            "stored property package payload hash `{}` does not match expected hash `{expected_hash}`",
            actual.hash
        )));
    }

    if expected_size_bytes != 0 && actual.size_bytes != expected_size_bytes {
        return Err(RfError::invalid_input(format!(
            "stored property package payload size `{}` does not match expected size `{expected_size_bytes}`",
            actual.size_bytes
        )));
    }

    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256_digest(bytes);
    let mut output = String::with_capacity(64);

    for byte in digest {
        let _ = write!(&mut output, "{byte:02x}");
    }

    output
}

fn sha256_digest(bytes: &[u8]) -> [u8; 32] {
    let mut state = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut padded = bytes.to_vec();
    let bit_length = (bytes.len() as u64).wrapping_mul(8);

    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        compress_sha256_block(&mut state, chunk);
    }

    let mut digest = [0u8; 32];
    for (index, value) in state.into_iter().enumerate() {
        digest[index * 4..(index + 1) * 4].copy_from_slice(&value.to_be_bytes());
    }

    digest
}

fn compress_sha256_block(state: &mut [u32; 8], chunk: &[u8]) {
    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    let mut message_schedule = [0u32; 64];
    for (index, word) in message_schedule.iter_mut().take(16).enumerate() {
        let offset = index * 4;
        *word = u32::from_be_bytes([
            chunk[offset],
            chunk[offset + 1],
            chunk[offset + 2],
            chunk[offset + 3],
        ]);
    }

    for index in 16..64 {
        let s0 = message_schedule[index - 15].rotate_right(7)
            ^ message_schedule[index - 15].rotate_right(18)
            ^ (message_schedule[index - 15] >> 3);
        let s1 = message_schedule[index - 2].rotate_right(17)
            ^ message_schedule[index - 2].rotate_right(19)
            ^ (message_schedule[index - 2] >> 10);
        message_schedule[index] = message_schedule[index - 16]
            .wrapping_add(s0)
            .wrapping_add(message_schedule[index - 7])
            .wrapping_add(s1);
    }

    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;

    for index in 0..64 {
        let big_sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choice = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(big_sigma1)
            .wrapping_add(choice)
            .wrapping_add(ROUND_CONSTANTS[index])
            .wrapping_add(message_schedule[index]);
        let big_sigma0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = big_sigma0.wrapping_add(majority);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

#[cfg(test)]
mod tests {
    use rf_types::ComponentId;

    use crate::{StoredPropertyPackagePayload, StoredThermoComponent};

    use super::{property_package_payload_integrity, sha256_hex};

    #[test]
    fn sha256_matches_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn payload_integrity_uses_canonical_pretty_json_bytes() {
        let payload = StoredPropertyPackagePayload::new(
            "pkg-1",
            "2026.03.1",
            vec![StoredThermoComponent::new(
                ComponentId::new("methane"),
                "Methane",
            )],
        );

        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");

        assert_eq!(
            integrity.hash,
            "sha256:4240b655ba93cd4a9cc8553ab7ae737a27660268e52bc07227eabb2da3639999"
        );
        assert_eq!(integrity.size_bytes, 407);
    }
}
