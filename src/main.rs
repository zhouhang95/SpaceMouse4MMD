#![allow(dead_code)]
use std::mem::size_of;

use glam::{vec3, Mat3, Vec3};
use windows_sys::{s, Win32::{Foundation::HMODULE, System::{Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory}, ProcessStatus::EnumProcessModules, Threading::{OpenProcess, PROCESS_ALL_ACCESS}}, UI::WindowsAndMessaging::{FindWindowA, GetWindowThreadProcessId, MessageBoxA, PostMessageA, MB_OK, WM_PAINT}}};

struct AxisSpec {
    channel: u8,
    byte1: u8,
    byte2: u8,
}
// (channel=1, byte1=1, byte2=2, scale=1)

struct DeviceSpec {
    vid: u16,
    pid: u16,
    axis_specs: [AxisSpec; 6], // x, y, z, pitch, roll, yaw
}

impl DeviceSpec {
    fn get_num_bytes_to_read(&self) -> u8 {
        let mut byte_indices = Vec::new();
        for axis_spec in &self.axis_specs {
            byte_indices.push(axis_spec.byte1);
            byte_indices.push(axis_spec.byte2);
        }
        *byte_indices.iter().max().unwrap() + 1
    }
}

unsafe fn get_base_addr_by_enum_process(pid: u32) -> u64 {
    let h_process = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
    let mut module: [HMODULE; 256] = [0; 256];
    let mut size: u32 = 0;

    EnumProcessModules(h_process, &mut module[0], module.len() as _, &mut size);

    module[0] as _
}
unsafe fn get_mmd_main_h_wnd() -> Option<isize> {
    let h_wnd = FindWindowA(s!("Polygon Movie Maker"), 0 as _);
    if h_wnd > 0 {
        Some(h_wnd)
    } else {
        None
    }
}
unsafe fn get_mmd_main_handle_and_addr(h_wnd: isize) -> (isize, u64) {
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(h_wnd, &mut pid);
    let base = get_base_addr_by_enum_process(pid);
    let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

    let mut addr: u64 = 0;
    ReadProcessMemory(
        handle,
        (base + 0x1445F8) as _,
        &mut addr as *mut u64 as *mut _,
        size_of::<u64>(),
        0 as _
    );
    (handle, addr)
}

pub fn rot3(rot: Vec3) -> Mat3 {
    let yaw = Mat3::from_rotation_y(rot.y);
    let pitch = Mat3::from_rotation_x(rot.x);
    let roll = Mat3::from_rotation_z(rot.z);
    roll * pitch * yaw
}

fn main() {
    let h_wnd = unsafe { get_mmd_main_h_wnd() };
    if h_wnd.is_none() {
        unsafe { MessageBoxA(0, s!("Not Found MMD, please launch MMD!"), s!("MMD 3D Space Mouse"), MB_OK) };
        return;
    }
    let (handle, addr) = unsafe { get_mmd_main_handle_and_addr(h_wnd.unwrap()) };
    eprint!("addr: {:X}", addr);

    let space_navigator = DeviceSpec {
        vid: 0x046d,
        pid: 0xc626,
        axis_specs: [
            AxisSpec { channel: 1, byte1: 1, byte2: 2 },
            AxisSpec { channel: 1, byte1: 3, byte2: 4 },
            AxisSpec { channel: 1, byte1: 5, byte2: 6 },
            AxisSpec { channel: 2, byte1: 1, byte2: 2 },
            AxisSpec { channel: 2, byte1: 3, byte2: 4 },
            AxisSpec { channel: 2, byte1: 5, byte2: 6 },
        ],
    };
    let api = hidapi::HidApi::new().unwrap();
    // Print out information about all connected devices
    let mut devices = Vec::new();
    for device in api.device_list() {
        if device.vendor_id() == space_navigator.vid {
            if device.product_id() == space_navigator.pid {
                devices.push(device.clone());
            }
        }
    }
    if devices.len() == 0 {
        unsafe { MessageBoxA(0, s!("Not Found 3D Space Mouse, please connect your device"), s!("MMD 3D Space Mouse"), MB_OK) };
        return;
    }
    let device: hidapi::HidDevice = devices[0].open_device(&api).unwrap();
    // Read data from device
    loop {
        unsafe {
            let mut dist: f32 = 0.0;
            WriteProcessMemory(
                handle,
                (addr + 661752) as _,
                &mut dist as *mut f32 as *mut _,
                size_of::<f32>(),
                0 as _
            );
        }
        
        let mut buf = [0u8; 32];
        let _res = device.read(&mut buf[..]).unwrap();
        // println!("Read: {}: {:?}", res, &buf[..res]);
        let mut rxyz = glam::Vec3::ZERO;
        unsafe {
            ReadProcessMemory(
                handle,
                (addr + 840) as _,
                &mut rxyz as *mut glam::Vec3 as *mut _,
                size_of::<glam::Vec3>(),
                0 as _
            );
        }

        if buf[0] == 1 {
            let x: i16 = bytemuck::must_cast([buf[1], buf[2]]);
            let y: i16 = bytemuck::must_cast([buf[3], buf[4]]);
            let z: i16 = bytemuck::must_cast([buf[5], buf[6]]);
            let mmd_xyz = glam::vec3(x as f32, z as f32, y as f32) * glam::vec3(-1.0, 1.0, 1.0) * -1.0;
            let mut xyz = glam::Vec3::ZERO;
            unsafe {
                ReadProcessMemory(
                    handle,
                    (addr + 876) as _,
                    &mut xyz as *mut glam::Vec3 as *mut _,
                    size_of::<glam::Vec3>(),
                    0 as _
                );
            }
            let rot = rot3(rxyz * vec3(-1.0, -1.0, 1.0));
            let mmd_xyz = rot * mmd_xyz;
            xyz = xyz + mmd_xyz * 0.003;
            unsafe {
                WriteProcessMemory(
                    handle,
                    (addr + 876) as _,
                    &mut xyz as *mut glam::Vec3 as *mut _,
                    size_of::<glam::Vec3>(),
                    0 as _
                );
            }
        }
        else if buf[0] == 2 {
            let x: i16 = bytemuck::must_cast([buf[1], buf[2]]);
            let y: i16 = bytemuck::must_cast([buf[3], buf[4]]);
            let z: i16 = bytemuck::must_cast([buf[5], buf[6]]);
            let mmd_rxyz = glam::vec3(x as f32, z as f32, y as f32) * glam::vec3(-1.0, 1.0, 1.0)  * -1.0;
            rxyz = rxyz + mmd_rxyz * 0.0001;
            unsafe {
                WriteProcessMemory(
                    handle,
                    (addr + 840) as _,
                    &mut rxyz as *mut glam::Vec3 as *mut _,
                    size_of::<glam::Vec3>(),
                    0 as _
                );
            }
            
        }
        unsafe { PostMessageA(h_wnd.unwrap(), WM_PAINT, 0, 0) };
    }
}
