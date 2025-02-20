pub(crate) mod attention;
pub(crate) mod conv;
pub(crate) mod layer_norm;

use candle::{Device, Result};

pub(crate) trait BenchDevice {
    fn sync(&self) -> Result<()>;

    fn bench_name<S: Into<String>>(&self, name: S) -> String;
}

impl BenchDevice for Device {
    fn sync(&self) -> Result<()> {
        match self {
            Device::Cpu => Ok(()),
            Device::Cuda(device) => {
                #[cfg(feature = "cuda")]
                {
                    use candle::cuda::WrapErr;
                    return Ok(device.synchronize().w()?);
                }
                #[cfg(not(feature = "cuda"))]
                panic!("Cuda device without cuda feature enabled: {:?}", device)
            }
            Device::Metal(device) => {
                #[cfg(feature = "metal")]
                return device.wait_until_completed();
                #[cfg(not(feature = "metal"))]
                panic!("Metal device without metal feature enabled: {:?}", device)
            }
        }
    }

    fn bench_name<S: Into<String>>(&self, name: S) -> String {
        match self {
            Device::Cpu => {
                let cpu_type = if cfg!(feature = "accelerate") {
                    "accelerate"
                } else if cfg!(feature = "mkl") {
                    "mkl"
                } else {
                    "cpu"
                };
                format!("{}_{}", cpu_type, name.into())
            }
            Device::Cuda(_) => format!("cuda_{}", name.into()),
            Device::Metal(_) => format!("metal_{}", name.into()),
        }
    }
}

struct BenchDeviceHandler {
    devices: Vec<Device>,
}

impl BenchDeviceHandler {
    pub fn new() -> Result<Self> {
        let mut devices = Vec::new();
        if cfg!(feature = "metal") {
            devices.push(Device::new_metal(0)?);
        } else if cfg!(feature = "cuda") {
            devices.push(Device::new_cuda(0)?);
        }
        devices.push(Device::Cpu);
        Ok(Self { devices })
    }
}
