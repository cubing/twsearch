use std::{fmt::Display, str::FromStr};

use ascii::AsciiString;
use cubing::alg::Alg;
use hex::encode;
use rand::{rng, RngCore};
use rand_core::impls::fill_bytes_via_next;
use sha2::{Digest, Sha256};

use crate::{
    _internal::errors::ArgumentError,
    scramble::{random_scramble_for_event::derive_scramble_for_event, Event},
};

// A fixed (non-zero) value to distinguish a valid byte string from a random
// one. The is an arbitrary, so let's troll the youth. 🤪
const PROTOCOL_INDICATOR: u8 = 0x67;

pub const DERIVATION_SEED_BYTE_LENGTH: usize = 32;
#[derive(Copy, Clone, Debug)]
pub struct DerivationSeed([u8; DERIVATION_SEED_BYTE_LENGTH]);

impl FromStr for DerivationSeed {
    // TODO: This is `String` to play nice with `clap`
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Ok(bytes) = hex::decode(value) else {
            return Err("Invalid hex byte input.".to_owned());
        };

        let len = bytes.len();
        if len != DERIVATION_SEED_BYTE_LENGTH {
            return Err(format!(
                "Invalid hex byte length: {} (expected: {} bytes)",
                len, DERIVATION_SEED_BYTE_LENGTH
            ));
        }

        let mut derivation_seed_bytes: [u8; 32] = Default::default();
        derivation_seed_bytes.copy_from_slice(bytes.as_slice());

        DerivationSeed::try_new(derivation_seed_bytes).map_err(|err| err.description)
    }
}

pub fn protocol_indicator(array: &[u8]) -> u8 {
    array[0]
}

impl DerivationSeed {
    pub fn try_new(bytes: [u8; DERIVATION_SEED_BYTE_LENGTH]) -> Result<Self, ArgumentError> {
        if protocol_indicator(&bytes) != PROTOCOL_INDICATOR {
            return Err("Invalid protocol indicator. The first byte must be 0x67.".into());
        }
        Ok(Self(bytes))
    }

    // Sets the level to `0xFF`.
    pub fn from_thread_rng() -> Self {
        let mut bytes: [u8; DERIVATION_SEED_BYTE_LENGTH] = [0; DERIVATION_SEED_BYTE_LENGTH];
        rng().fill_bytes(&mut bytes);
        bytes[0] = 0x67;
        bytes[1] = 0xFF;
        DerivationSeed::try_new(bytes).unwrap()
    }

    pub fn level(&self) -> u8 {
        self.0[1]
    }

    pub fn derive_hierarchy<'a>(
        &self,
        derivation_salts: impl IntoIterator<Item = &'a DerivationSalt>,
    ) -> Self {
        let mut derivation_seed = *self;
        for derivation_salt in derivation_salts {
            derivation_seed = derivation_seed.derive(derivation_salt);
        }

        derivation_seed
    }

    pub fn derive(&self, derivation_salt: &DerivationSalt) -> Self {
        let mut out: [u8; DERIVATION_SEED_BYTE_LENGTH] = {
            let mut data: [u8; DERIVATION_SEED_BYTE_LENGTH * 2] =
                [0; DERIVATION_SEED_BYTE_LENGTH * 2];
            data[..DERIVATION_SEED_BYTE_LENGTH].copy_from_slice(&self.0);
            data[DERIVATION_SEED_BYTE_LENGTH..].copy_from_slice(&derivation_salt.hashed_salt);
            Sha256::digest(data).into()
        };

        // // TODO: tune params
        // // TOOD: Reuse the salt hash?
        // static ARGON2_ID: LazyLock<Argon2> = std::sync::LazyLock::new(|| {
        //     Argon2::new(
        //         argon2::Algorithm::Argon2id,
        //         argon2::Version::V0x13,
        //         Params::default(),
        //     )
        // });
        // let mut out: [u8; DERIVATION_SEED_BYTE_LENGTH] = {
        //     let mut out: [u8; DERIVATION_SEED_BYTE_LENGTH] = Default::default();
        //     ARGON2_ID
        //         .hash_password_into(&self.0, &derivation_salt.hashed_salt, &mut out)
        //         .unwrap();
        //     out
        // };

        // TODO: Hash to 30 bytes instead of overwriting? (Overwriting would have a benefit to protect against length extension attacks, but we're not using SHA256.)
        // TODO: use the end bytes?
        out[0] = PROTOCOL_INDICATOR;
        out[1] = match &self.0[1] {
            // We don't overflow, but we just cap at `0xff`.
            0xFF => 0xFF,
            v => v + 1,
        };

        // dbg!(out);
        Self(out)
    }
}

impl Display for DerivationSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", encode(self.0))
    }
}

#[derive(Debug, Clone)]
pub struct DerivationSalt {
    unhashed_salt: AsciiString,
    hashed_salt: [u8; DERIVATION_SEED_BYTE_LENGTH],
}

impl FromStr for DerivationSalt {
    // TODO: This is `String` to play nice with `clap`
    type Err = String;

    fn from_str(salt: &str) -> Result<Self, Self::Err> {
        if salt.is_empty() {
            return Err("Empty salt values are not currently allowed.".to_owned());
        }
        if salt.contains('/') {
            return Err("Salt values must not contain slashes.".to_owned());
        }
        let unhashed_salt = match AsciiString::from_ascii(salt) {
            Ok(unhashed_salt) => unhashed_salt,
            Err(_) => return Err("Salt values must currently be ASCII.".to_owned()),
        };
        let hashed_salt: [u8; DERIVATION_SEED_BYTE_LENGTH] = Sha256::digest(&unhashed_salt).into();

        Ok(Self {
            unhashed_salt,
            hashed_salt,
        })
    }
}

impl From<Event> for DerivationSalt {
    fn from(event: Event) -> Self {
        Self::from_str(event.id()).unwrap()
    }
}

impl DerivationSalt {
    pub fn unhashed_salt(&self) -> &AsciiString {
        &self.unhashed_salt
    }
}

const SCRAMBLE_DERIVATION_LEVEL: u8 = 8;
pub fn derive_scramble_for_event_seeded(
    derivation_seed: &DerivationSeed,
    derivation_salt_hierarchy: &Vec<DerivationSalt>,
    subevent: Event,
) -> Result<Alg, String> {
    if derivation_salt_hierarchy.len() > 1
        && derivation_salt_hierarchy[derivation_salt_hierarchy.len() - 2].unhashed_salt()
            != subevent.id()
    {
        return Err("Mismatched subevent in second-to-last level of hierarchy".to_owned());
    }
    let derivation_seed = derivation_seed.derive_hierarchy(derivation_salt_hierarchy);

    if derivation_seed.level() != SCRAMBLE_DERIVATION_LEVEL {
        return Err(format!(
            "Expected derivation level {}, saw: {}",
            SCRAMBLE_DERIVATION_LEVEL,
            derivation_seed.level()
        ));
    }
    let derivation_seed = derivation_seed.derive(&subevent.into());
    derive_scramble_for_event(subevent, derivation_seed).map_err(|e| e.description)
}

pub struct DerivationSeedRng {
    derivation_seed: DerivationSeed,
    index: u128,
}

impl DerivationSeedRng {
    pub fn new(derivation_seed: DerivationSeed) -> Self {
        Self {
            derivation_seed,
            index: 0,
        }
    }
}

// TODO: make our own `Trait` for seeded permutation RNG that is easy to implement portably.
impl RngCore for DerivationSeedRng {
    fn next_u64(&mut self) -> u64 {
        let mut data: [u8; DERIVATION_SEED_BYTE_LENGTH * 2] = [0; DERIVATION_SEED_BYTE_LENGTH * 2];
        data[..DERIVATION_SEED_BYTE_LENGTH].copy_from_slice(&self.derivation_seed.0);
        // TODO
        data[(DERIVATION_SEED_BYTE_LENGTH * 3 / 2)..].copy_from_slice(&self.index.to_le_bytes());

        let hash: &[u8; DERIVATION_SEED_BYTE_LENGTH] = &Sha256::digest(data).into();
        let value = u64::from_le_bytes(hash[..8].try_into().unwrap());

        self.index += 1;
        value
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }
}
