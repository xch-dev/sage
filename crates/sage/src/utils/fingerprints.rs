use chia_wallet_sdk::chia::sha2::Sha256;

/// How many fingerprints are tried for a wallet before giving up.
const FINGERPRINT_ATTEMPTS: u32 = 1000;

/// Picks a fingerprint for a wallet that has no BLS key to derive one from. Fingerprints name
/// the wallet's database and identify it everywhere else, so they must not collide.
///
/// The first candidate is the first 4 bytes of `seed`, and later ones are hashed from it, so a
/// wallet gets the same fingerprint whenever that one is free.
pub fn assign_fingerprint(seed: &[u8], is_taken: impl Fn(u32) -> bool) -> Option<u32> {
    (0..FINGERPRINT_ATTEMPTS)
        .map(|attempt| {
            if attempt == 0 {
                return u32::from_be_bytes([seed[0], seed[1], seed[2], seed[3]]);
            }

            let mut hasher = Sha256::new();
            hasher.update(seed);
            hasher.update(attempt.to_be_bytes());
            let hash = hasher.finalize();
            u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
        })
        .find(|&fingerprint| !is_taken(fingerprint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_the_seed_prefix() {
        assert_eq!(
            assign_fingerprint(&[1, 2, 3, 4, 5], |_| false),
            Some(0x0102_0304)
        );
    }

    #[test]
    fn skips_taken_fingerprints() {
        let seed = [6; 32];
        let first = assign_fingerprint(&seed, |_| false).expect("free fingerprint");
        let second = assign_fingerprint(&seed, |fingerprint| fingerprint == first)
            .expect("free fingerprint");

        assert_ne!(first, second);
        // The same seed always moves to the same alternative.
        assert_eq!(
            assign_fingerprint(&seed, |fingerprint| fingerprint == first),
            Some(second)
        );
        assert_eq!(assign_fingerprint(&seed, |_| true), None);
    }
}
