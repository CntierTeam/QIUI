//! Minimal BLE session (btleplug) + mock for offline tests.

use crate::protocol::Profile;
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BleError {
    #[error("ble: {0}")]
    Ble(#[from] btleplug::Error),
    #[error("{0}")]
    Msg(String),
}

pub struct Session {
    pub adapter: Adapter,
    pub peripheral: Option<Peripheral>,
    pub profile: Profile,
    write_char: Option<Characteristic>,
    notify_char: Option<Characteristic>,
}

impl Session {
    pub async fn open(profile: Profile) -> Result<Self, BleError> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters
            .into_iter()
            .next()
            .ok_or_else(|| BleError::Msg("no bluetooth adapter".into()))?;
        Ok(Self {
            adapter,
            peripheral: None,
            profile,
            write_char: None,
            notify_char: None,
        })
    }

    pub async fn scan(&self, secs: u64) -> Result<Vec<(String, Option<String>)>, BleError> {
        self.adapter.start_scan(ScanFilter::default()).await?;
        tokio::time::sleep(Duration::from_secs(secs)).await;
        let peris = self.adapter.peripherals().await?;
        let mut out = Vec::new();
        for p in peris {
            let addr = p.address().to_string();
            let name = p
                .properties()
                .await?
                .and_then(|x| x.local_name);
            out.push((addr, name));
        }
        self.adapter.stop_scan().await?;
        Ok(out)
    }

    pub async fn connect(&mut self, address: &str) -> Result<(), BleError> {
        self.adapter.start_scan(ScanFilter::default()).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
        let peris = self.adapter.peripherals().await?;
        let target = address.to_ascii_lowercase();
        let peri = peris
            .into_iter()
            .find(|p| p.address().to_string().to_ascii_lowercase() == target)
            .ok_or_else(|| BleError::Msg(format!("device not found: {address}")))?;
        self.adapter.stop_scan().await?;
        peri.connect().await?;
        peri.discover_services().await?;
        let chars = peri.characteristics();
        self.write_char = chars.iter().find(|c| c.uuid == self.profile.write()).cloned();
        self.notify_char = chars.iter().find(|c| c.uuid == self.profile.notify()).cloned();
        if self.write_char.is_none() {
            return Err(BleError::Msg(format!(
                "write char {} not found (profile={})",
                self.profile.write(),
                self.profile.name()
            )));
        }
        if let Some(nc) = &self.notify_char {
            peri.subscribe(nc).await?;
        }
        self.peripheral = Some(peri);
        Ok(())
    }

    pub async fn write_hex(&self, hex_str: &str) -> Result<(), BleError> {
        let peri = self
            .peripheral
            .as_ref()
            .ok_or_else(|| BleError::Msg("not connected".into()))?;
        let ch = self
            .write_char
            .as_ref()
            .ok_or_else(|| BleError::Msg("no write char".into()))?;
        let bytes = hex::decode(hex_str.trim()).map_err(|e| BleError::Msg(e.to_string()))?;
        peri.write(ch, &bytes, WriteType::WithResponse).await?;
        Ok(())
    }

    pub async fn wait_notify(&self, timeout_ms: u64) -> Result<Vec<u8>, BleError> {
        let peri = self
            .peripheral
            .as_ref()
            .ok_or_else(|| BleError::Msg("not connected".into()))?;
        let mut stream = peri.notifications().await?;
        let fut = stream.next();
        match tokio::time::timeout(Duration::from_millis(timeout_ms), fut).await {
            Ok(Some(n)) => Ok(n.value),
            Ok(None) => Err(BleError::Msg("notify stream ended".into())),
            Err(_) => Err(BleError::Msg("notify timeout".into())),
        }
    }

    pub fn service_uuid(&self) -> Uuid {
        self.profile.service()
    }
}

/// In-memory fake for CI / no hardware.
pub struct MockSession {
    pub profile: Profile,
    pub last_write: Option<Vec<u8>>,
}

impl MockSession {
    pub fn new(profile: Profile) -> Self {
        Self {
            profile,
            last_write: None,
        }
    }

    pub fn write_hex(&mut self, hex_str: &str) -> Result<Vec<u8>, BleError> {
        let bytes = hex::decode(hex_str.trim()).map_err(|e| BleError::Msg(e.to_string()))?;
        self.last_write = Some(bytes.clone());
        // Echo a fake notify payload (zeros) for smoke.
        Ok(vec![0u8; bytes.len().max(16)])
    }
}
