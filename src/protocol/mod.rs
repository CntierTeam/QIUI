//! BLE GATT profiles for QIUI/Cellmate-compatible devices.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn u(s: &str) -> Uuid {
    Uuid::parse_str(s).expect("valid uuid")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    /// Cellmate Gen2/Gen3 — `t7/g3`, `wb/z2`
    Cellmate,
    /// Metal / KeyPod metal (default) — `x7/j0`, `f8/u`
    Fee5,
    /// KeyPod metal when type==20 — `x7/j0`
    KeyPodMetal20,
    /// PearFlower Three — `k7/p0`
    PearFlower3,
    /// GenMetal / shake family — `o7/j1`
    Ae3,
    /// Another lock family — `c8/m0`
    Ac8,
    /// PulseBird-ish — `u7/d0`
    Ffa0,
}

impl Profile {
    pub fn all() -> &'static [Profile] {
        &[
            Profile::Cellmate,
            Profile::Fee5,
            Profile::KeyPodMetal20,
            Profile::PearFlower3,
            Profile::Ae3,
            Profile::Ac8,
            Profile::Ffa0,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Profile::Cellmate => "cellmate",
            Profile::Fee5 => "fee5",
            Profile::KeyPodMetal20 => "keypod-metal-20",
            Profile::PearFlower3 => "pearflower3",
            Profile::Ae3 => "ae3",
            Profile::Ac8 => "8ac0",
            Profile::Ffa0 => "ffa0",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::all().iter().copied().find(|p| p.name() == s)
    }

    pub fn service(self) -> Uuid {
        match self {
            Profile::Cellmate => u("0000fee7-0000-1000-8000-00805f9b34fb"),
            Profile::Fee5 => u("0000fee5-0000-1000-8000-00805f9b34fb"),
            Profile::KeyPodMetal20 => u("00000b30-0000-1000-8000-00805f9b34fb"),
            Profile::PearFlower3 => u("0000ac3a-0000-1000-8000-00805f9b34fb"),
            Profile::Ae3 => u("0000ae3a-0000-1000-8000-00805f9b34fb"),
            Profile::Ac8 => u("00008ac0-0000-1000-8000-00805f9b34fb"),
            Profile::Ffa0 => u("0000ffa0-0000-1000-8000-00805f9b34fb"),
        }
    }

    pub fn write(self) -> Uuid {
        match self {
            Profile::Cellmate => u("000036f5-0000-1000-8000-00805f9b34fb"),
            Profile::Fee5 => u("00003ff5-0000-1000-8000-00805f9b34fb"),
            Profile::KeyPodMetal20 => u("00000b32-0000-1000-8000-00805f9b34fb"),
            Profile::PearFlower3 => u("0000ac3b-0000-1000-8000-00805f9b34fb"),
            Profile::Ae3 => u("0000ae3b-0000-1000-8000-00805f9b34fb"),
            Profile::Ac8 => u("00008ac1-0000-1000-8000-00805f9b34fb"),
            Profile::Ffa0 => u("0000ffa1-0000-1000-8000-00805f9b34fb"),
        }
    }

    pub fn notify(self) -> Uuid {
        match self {
            Profile::Cellmate => u("000036f6-0000-1000-8000-00805f9b34fb"),
            Profile::Fee5 => u("00003ff6-0000-1000-8000-00805f9b34fb"),
            Profile::KeyPodMetal20 => u("00000b31-0000-1000-8000-00805f9b34fb"),
            Profile::PearFlower3 => u("0000ac3c-0000-1000-8000-00805f9b34fb"),
            Profile::Ae3 => u("0000ae3c-0000-1000-8000-00805f9b34fb"),
            Profile::Ac8 => u("00008ac2-0000-1000-8000-00805f9b34fb"),
            Profile::Ffa0 => u("0000ffa2-0000-1000-8000-00805f9b34fb"),
        }
    }

    /// CCCD for enabling notifications.
    pub fn cccd() -> Uuid {
        u("00002902-0000-1000-8000-00805f9b34fb")
    }
}

/// Classic Nokelock-style local plaintext builders (16-byte blocks before device AES-ECB).
/// Modern QIUI usually gets ciphertext from cloud; these remain useful when you have `lockKey`.
pub mod nokelock {
    use rand::RngCore;

    pub fn get_token_plain() -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = 0x06;
        b[1] = 0x01;
        b[2] = 0x01;
        b[3] = 0x01;
        rand::thread_rng().fill_bytes(&mut b[4..]);
        b
    }

    pub fn get_battery_plain(token: &[u8; 4]) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = 0x02;
        b[1] = 0x01;
        b[2] = 0x01;
        b[3] = 0x01;
        b[4..8].copy_from_slice(token);
        rand::thread_rng().fill_bytes(&mut b[8..]);
        b
    }

    /// Unlock with password `"000000"` (ASCII) + 4-byte session token.
    pub fn unlock_plain(token: &[u8; 4], password: &[u8; 6]) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = 0x05;
        b[1] = 0x01;
        b[2] = 0x06;
        b[3..9].copy_from_slice(password);
        b[9..13].copy_from_slice(token);
        rand::thread_rng().fill_bytes(&mut b[13..]);
        b
    }

    pub fn parse_token_response(plain: &[u8]) -> Option<[u8; 4]> {
        // 06 02 07 TT TT TT TT ...
        if plain.len() >= 7 && plain[0] == 0x06 && plain[1] == 0x02 {
            let mut t = [0u8; 4];
            t.copy_from_slice(&plain[3..7]);
            Some(t)
        } else {
            None
        }
    }
}

/// Device AES-128-ECB (Nokelock / early Cellmate). Key is 16 raw bytes (`lockKey`).
pub mod device_aes {
    use aes::Aes128;
    use cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("key must be 16 bytes")]
        KeyLen,
        #[error("payload must be multiple of 16")]
        BlockLen,
    }

    pub fn encrypt(key: &[u8], plain: &[u8]) -> Result<Vec<u8>, Error> {
        if key.len() != 16 {
            return Err(Error::KeyLen);
        }
        if !plain.len().is_multiple_of(16) {
            return Err(Error::BlockLen);
        }
        let cipher = Aes128::new_from_slice(key).map_err(|_| Error::KeyLen)?;
        let mut out = plain.to_vec();
        for chunk in out.as_chunks_mut::<16>().0 {
            cipher.encrypt_block(chunk.into());
        }
        Ok(out)
    }

    pub fn decrypt(key: &[u8], cipher_bytes: &[u8]) -> Result<Vec<u8>, Error> {
        if key.len() != 16 {
            return Err(Error::KeyLen);
        }
        if !cipher_bytes.len().is_multiple_of(16) {
            return Err(Error::BlockLen);
        }
        let cipher = Aes128::new_from_slice(key).map_err(|_| Error::KeyLen)?;
        let mut out = cipher_bytes.to_vec();
        for chunk in out.as_chunks_mut::<16>().0 {
            cipher.decrypt_block(chunk.into());
        }
        Ok(out)
    }
}
