//! Read-only host battery status for the desktop panel; independent of campaign state.
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryStatus {
    percent: Option<u8>,
    charging: bool,
    plugged_in: bool,
}

#[cfg(any(windows, test))]
fn battery_status(flags: u8, percent: u8, ac: u8) -> Option<BatteryStatus> {
    if flags == 255 || flags & 128 != 0 {
        return None;
    }
    Some(BatteryStatus {
        percent: (percent <= 100).then_some(percent),
        charging: flags & 8 != 0,
        plugged_in: ac == 1,
    })
}

#[tauri::command]
pub fn host_battery_status() -> Option<BatteryStatus> {
    #[cfg(windows)]
    {
        // SYSTEM_POWER_STATUS layout from winbase.h. No subprocess or filesystem access.
        // https://learn.microsoft.com/windows/win32/api/winbase/ns-winbase-system_power_status
        #[repr(C)]
        #[derive(Default)]
        struct SystemPowerStatus {
            ac_line_status: u8,
            battery_flag: u8,
            battery_life_percent: u8,
            system_status_flag: u8,
            battery_life_time: u32,
            battery_full_life_time: u32,
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn GetSystemPowerStatus(status: *mut SystemPowerStatus) -> i32;
        }
        let mut status = SystemPowerStatus::default();
        // SAFETY: status has the documented C layout and is writable for the entire call.
        if unsafe { GetSystemPowerStatus(&mut status) } == 0 {
            return None;
        }
        battery_status(
            status.battery_flag,
            status.battery_life_percent,
            status.ac_line_status,
        )
    }
    #[cfg(not(windows))]
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_absent_and_unknown_batteries() {
        assert_eq!(battery_status(128, 100, 1), None);
        assert_eq!(battery_status(255, 255, 255), None);
    }

    #[test]
    fn reports_notebook_charge_without_inventing_unknown_percentage() {
        assert_eq!(
            battery_status(9, 85, 1),
            Some(BatteryStatus {
                percent: Some(85),
                charging: true,
                plugged_in: true,
            })
        );
        assert_eq!(
            battery_status(0, 52, 0),
            Some(BatteryStatus {
                percent: Some(52),
                charging: false,
                plugged_in: false,
            })
        );
        assert_eq!(battery_status(1, 255, 1).unwrap().percent, None);
    }
}
