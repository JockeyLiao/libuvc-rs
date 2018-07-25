use streaming;
use uvc_sys::*;

use std::ffi::CStr;

use error::{UvcError, UvcResult};

use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct Device<'a> {
    dev: NonNull<uvc_device>,
    _dev: PhantomData<&'a uvc_device>,
}

impl<'a> Drop for Device<'a> {
    fn drop(&mut self) {
        unsafe { uvc_unref_device(self.dev.as_ptr()) };
    }
}

impl<'a> Device<'a> {
    pub(crate) unsafe fn from_raw(dev: *mut uvc_device) -> Self {
        Device {
            dev: NonNull::new(dev).unwrap(),
            _dev: PhantomData,
        }
    }
    pub fn open(&'a self) -> UvcResult<DeviceHandle<'a>> {
        unsafe {
            let mut devh = ::std::mem::uninitialized();
            let err = uvc_open(self.dev.as_ptr(), &mut devh).into();
            match err {
                UvcError::Success => Ok(DeviceHandle {
                    devh: NonNull::new(devh).unwrap(),
                    _devh: PhantomData,
                }),
                err => Err(err),
            }
        }
    }
    pub fn description(&self) -> UvcResult<DeviceDescription> {
        unsafe {
            let mut desc = ::std::mem::uninitialized();
            let err = uvc_get_device_descriptor(self.dev.as_ptr(), &mut desc).into();
            if err != UvcError::Success {
                return Err(err);
            }

            let id_vendor = (*desc).idVendor;
            let id_product = (*desc).idProduct;
            let bcd_uvc = (*desc).bcdUVC;

            let serial_number_c_str = (*desc).serialNumber;
            let serial_number = if serial_number_c_str.is_null() {
                None
            } else {
                Some(
                    CStr::from_ptr(serial_number_c_str)
                        .to_owned()
                        .into_string()
                        .unwrap(),
                )
            };
            let manufacturer_c_str = (*desc).manufacturer;
            let manufacturer = if manufacturer_c_str.is_null() {
                None
            } else {
                Some(
                    CStr::from_ptr(manufacturer_c_str)
                        .to_owned()
                        .into_string()
                        .unwrap(),
                )
            };
            let product_c_str = (*desc).product;
            let product = if product_c_str.is_null() {
                None
            } else {
                Some(
                    CStr::from_ptr(product_c_str)
                        .to_owned()
                        .into_string()
                        .unwrap(),
                )
            };
            let descp = Ok(DeviceDescription {
                id_vendor,
                id_product,
                bcd_uvc,
                serial_number,
                manufacturer,
                product,
            });

            uvc_free_device_descriptor(desc);

            descp
        }
    }
}

pub struct DeviceHandle<'a> {
    pub(crate) devh: NonNull<uvc_device_handle>,
    _devh: PhantomData<&'a uvc_device_handle>,
}

impl<'a> DeviceHandle<'a> {
    pub fn get_stream_ctrl_with_size_and_fps(
        &self,
        width: u32,
        height: u32,
        fps: u32,
    ) -> UvcResult<streaming::StreamCtrl<'a>> {
        unsafe {
            let mut ctrl = ::std::mem::uninitialized();
            let err = uvc_get_stream_ctrl_format_size(
                self.devh.as_ptr(),
                &mut ctrl,
                uvc_frame_format_UVC_FRAME_FORMAT_YUYV,
                width as i32,
                height as i32,
                fps as i32,
            ).into();
            if err != UvcError::Success {
                Err(err)
            } else {
                Ok(::StreamCtrl {
                    ctrl,
                    _ctrl: PhantomData,
                })
            }
        }
    }
}

impl<'a> Drop for DeviceHandle<'a> {
    fn drop(&mut self) {
        unsafe {
            uvc_close(self.devh.as_ptr());
        }
    }
}

#[derive(Debug)]
pub struct DeviceDescription {
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_uvc: u16,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}