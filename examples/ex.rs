#![cfg(windows)]
//! Test code for prototyping

use std::{error::Error, mem};

use windows::Win32::Devices::Bluetooth::{
    BluetoothFindFirstDevice, BluetoothFindFirstRadio, BluetoothFindNextDevice,
    BluetoothFindNextRadio, BluetoothFindRadioClose, BluetoothGetRadioInfo,
    BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS, BLUETOOTH_FIND_RADIO_PARAMS,
    BLUETOOTH_RADIO_INFO, BTH_ERROR_SUCCESS,
};
use windows::Win32::Foundation::{CloseHandle, HANDLE, SYSTEMTIME};

fn main() -> Result<(), Box<dyn Error>> {
    let mut hradio = HANDLE::default();
    let pbtfrp = BLUETOOTH_FIND_RADIO_PARAMS {
        dwSize: mem::size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
    };

    let hfind = unsafe { BluetoothFindFirstRadio(&pbtfrp, &mut hradio)? };

    loop {
        let mut radioinfo = BLUETOOTH_RADIO_INFO::default();
        radioinfo.dwSize = mem::size_of::<BLUETOOTH_RADIO_INFO>() as u32;

        let res = unsafe { BluetoothGetRadioInfo(hradio, &mut radioinfo) };
        if res != BTH_ERROR_SUCCESS {
            panic!("BluetoothGetRadioInfo error: {res}");
        }

        let i = radioinfo.szName.partition_point(|word| *word == 0);
        unsafe {
            println!(
                "address: {:#018X}, Name: {}, ClassofDevice: {}, lmpSubverion: {}, Manufacturer: {}",
                radioinfo.address.Anonymous.ullLong,
                String::from_utf16(&radioinfo.szName[..i])?,
                radioinfo.ulClassofDevice,
                radioinfo.lmpSubversion,
                radioinfo.manufacturer,
            );
        }
        println!("Devices:");
        find_devices(hradio)?;
        if unsafe { BluetoothFindNextRadio(hfind, &mut hradio) }.is_err() {
            println!("No more Bluetooth Radios");
            break;
        }
    }

    unsafe { BluetoothFindRadioClose(hfind)? };
    unsafe { CloseHandle(hradio)? };
    Ok(())
}

fn find_devices(hradio: HANDLE) -> Result<(), Box<dyn Error>> {
    let mut params = BLUETOOTH_DEVICE_SEARCH_PARAMS::default();
    params.dwSize = mem::size_of_val(&params) as u32;
    params.hRadio = hradio;
    params.fReturnAuthenticated = true.into();
    params.fReturnRemembered = true.into();
    params.fReturnUnknown = true.into();
    params.fReturnConnected = true.into();

    let mut btdi = BLUETOOTH_DEVICE_INFO::default();
    btdi.dwSize = mem::size_of_val(&btdi) as u32;

    let hfind = unsafe { BluetoothFindFirstDevice(&params, &mut btdi) }.unwrap();
    loop {
        let i = btdi.szName.partition_point(|word| *word == 0);
        unsafe {
            println!(
                "Address: {:#018X}, ClassOfDevice: {}, Connected: {}, Remembered: {}, Authenticated: {}, LastSeen: {}, LastUsed: {}, szName: {}",
                btdi.Address.Anonymous.ullLong, btdi.ulClassofDevice,
                btdi.fConnected.as_bool(),
                btdi.fRemembered.as_bool(),
                btdi.fAuthenticated.as_bool(),
                format_date(btdi.stLastSeen),
                format_date(btdi.stLastUsed),
                String::from_utf16(&btdi.szName[..i])?,
            );
        }
        if unsafe { BluetoothFindNextDevice(hfind, &mut btdi) }.is_err() {
            println!("No more Bluetooth Devices");
            break;
        }
    }
    Ok(())
}

fn format_date(time: SYSTEMTIME) -> String {
    format!("{:4}-{:2}-{:2} {:2}:{:2}:{:2}", time.wYear, time.wMonth, time.wDay, time.wHour, time.wMinute, time.wSecond)
}
