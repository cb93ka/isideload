//! Anisette from the Mac this runs on, through the system's AOSKit and AuthKit (see macos_local.m).

use std::{ffi::CStr, sync::Arc};

use rootcause::prelude::*;
use web_time::SystemTime;

use crate::anisette::{AnisetteClientInfo, AnisetteData, AnisetteDataResponse, AnisetteProvider};
use crate::auth::grandslam::GrandSlam;

unsafe extern "C" {
    fn isideload_mac_anisette(
        otp: *mut u8,
        machine_id: *mut u8,
        device_id: *mut u8,
        description: *mut u8,
        serial: *mut u8,
        cap: usize,
    ) -> i32;
}

/// The AuthKit part of X-Mme-Client-Info: akd's. With Xcode's here GrandSlam answers this Mac's anisette with 503.
const AKD_AUTHKIT: &str = "<com.apple.AuthKit/1 (com.apple.akd/1.0)>";

struct MacValues {
    otp: String,
    machine_id: String,
    device_id: String,
    description: String,
}

fn read_mac_values() -> Result<MacValues, Report> {
    const CAP: usize = 1024;
    let mut bufs = [[0u8; CAP]; 5];
    let [otp, mid, dev, desc, serial] = &mut bufs;
    let ok = unsafe {
        isideload_mac_anisette(
            otp.as_mut_ptr(),
            mid.as_mut_ptr(),
            dev.as_mut_ptr(),
            desc.as_mut_ptr(),
            serial.as_mut_ptr(),
            CAP,
        )
    };
    if ok != 1 {
        bail!("macOS did not provide anisette data (AOSKit/AuthKit unavailable)");
    }
    let s = |b: &[u8; CAP]| -> Result<String, Report> {
        Ok(CStr::from_bytes_until_nul(b)
            .context("Bad string from macOS")?
            .to_string_lossy()
            .into_owned())
    };
    Ok(MacValues {
        otp: s(&bufs[0])?,
        machine_id: s(&bufs[1])?,
        device_id: s(&bufs[2])?,
        description: s(&bufs[3])?,
    })
}

/// The device part of AuthKit's description ("<Mac16,12> <macOS;26.5.2;25F84>") followed by akd's AuthKit part.
fn client_info_from(description: &str) -> String {

    let device: Vec<&str> = description
        .split('>')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .take(2)
        .collect();
    format!("{}> {}", device.join("> "), AKD_AUTHKIT)
}

#[derive(Default)]
pub struct MacLocalAnisetteProvider;

impl MacLocalAnisetteProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AnisetteProvider for MacLocalAnisetteProvider {
    async fn get_anisette_data(&self) -> Result<AnisetteDataResponse, Report> {
        let v = read_mac_values()?;
        Ok(AnisetteDataResponse::Data(AnisetteData {
            machine_id: v.machine_id,
            one_time_password: v.otp,
            routing_info: "17106176".to_string(),
            _device_description: client_info_from(&v.description),
            device_unique_identifier: v.device_id,
            _local_user_id: String::new(),
            generated_at: SystemTime::now(),
        }))
    }

    async fn resync(&self, _sim: &str) -> Result<AnisetteDataResponse, Report> {
        self.get_anisette_data().await
    }

    async fn get_client_info(&self) -> Result<AnisetteClientInfo, Report> {
        let v = read_mac_values()?;
        Ok(AnisetteClientInfo {
            client_info: client_info_from(&v.description),
            user_agent: "akd/1.0 CFNetwork/808.1.4".to_string(),
        })
    }

    async fn provision(&mut self, _gs: &Arc<GrandSlam>, _force_recheck: bool) -> Result<(), Report> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_info_keeps_device_and_uses_akd() {
        assert_eq!(
            client_info_from("<Mac16,12> <macOS;26.5.2;25F84> <com.apple.AuthKit/1 ((null)/1.0)>"),
            "<Mac16,12> <macOS;26.5.2;25F84> <com.apple.AuthKit/1 (com.apple.akd/1.0)>"
        );
    }

    #[test]
    fn reads_this_mac() {
        let v = read_mac_values().unwrap();
        assert!(!v.otp.is_empty() && !v.machine_id.is_empty() && v.device_id.len() == 36);
    }
}
