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

fn main() {
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
    println!("Hello, world!");
    let api = hidapi::HidApi::new().unwrap();
    // Print out information about all connected devices
    let mut devices = Vec::new();
    for device in api.device_list() {
        println!("{:#?}", device);
        if device.vendor_id() == space_navigator.vid {
            if device.product_id() == space_navigator.pid {
                devices.push(device.clone());
            }
        }
    }
    let device: hidapi::HidDevice = devices[0].open_device(&api).unwrap();
    // Read data from device
    loop {
        let mut buf = [0u8; 32];
        let res = device.read(&mut buf[..]).unwrap();
        println!("Read: {}: {:?}", res, &buf[..res]);
    }
}
