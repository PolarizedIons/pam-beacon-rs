extern crate pam;

use std::ffi::CStr;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use pam::constants::{PamFlag, PamResultCode, };
use pam::module::{PamHandle, PamHooks};

struct CustomPam;
pam::pam_hooks!(CustomPam);

impl PamHooks for CustomPam {
    fn acct_mgmt(_pamh: &mut PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
        PamResultCode::PAM_SUCCESS
    }

    fn sm_authenticate(pamh: &mut PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
        let user = match pamh.get_user(None) {
            Ok(user) => user,
            _ => return PamResultCode::PAM_AUTH_ERR,
        };

        let auth_file_path = format!("/home/{}/.pambeacon", user);

        let file_content = match fs::read_to_string(auth_file_path) {
            Ok(content) => content,
            Err(_) => return PamResultCode::PAM_AUTH_ERR,
        };
        let authorised_macs = file_content
            .split("\n")
            .map(|line| line.trim())
            .collect();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let res = rt.block_on(async { scan_for_device(authorised_macs).await });

        match res {
            Ok(true) => PamResultCode::PAM_SUCCESS,
            _ => PamResultCode::PAM_AUTH_ERR,
        }
    }

    fn sm_setcred(_pamh: &mut PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
        PamResultCode::PAM_SUCCESS
    }
}

async fn scan_for_device(expected_values: Vec<&str>) -> Result<bool, btleplug::Error> {
    println!("Scanning for beacons....");

    let manager = match Manager::new().await {
        Ok(manager) => manager,
        Err(_) => return Ok(false),
    };

    let adapters = match manager.adapters().await {
        Ok(adapter) => adapter,
        Err(_) => return Ok(false),
    };
    let central = match adapters.into_iter().nth(0) {
        Some(adapter) => adapter,
        None => return Ok(false),
    };

    // start scanning for devices
    match central.start_scan(ScanFilter::default()).await {
        Ok(_) => {}
        Err(_) => return Ok(false),
    };

    // TODO
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    sleep(Duration::from_secs(2));

    let found_peripherals = match central.peripherals().await {
        Ok(found) => found,
        Err(_) => return Ok(false),
    };

    for p in found_peripherals {
        if expected_values.contains(&p.address().to_string().as_str()){
            println!("Found beacon {} ({})", p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .unwrap_or(String::new()),
            p.address());
            return Ok(true);
        }
    }

    return Ok(false);
}