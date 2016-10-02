use ::libinput_sys::*;
use ::std::ffi::CStr;
use ::std::os::raw::c_char;

// Returns an allocated String.
unsafe fn cbuf_to_string(buf: *const c_char) -> String {
    let u8_buf = CStr::from_ptr(buf).to_bytes();
    // to_owned() allocates the buffer.
    ::std::str::from_utf8(u8_buf).unwrap().to_owned()
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Pressed,
    Released,
}

#[derive(Clone, Debug)]
pub enum Event {
    None,
    DeviceAdd(Device),
    DeviceRemove(Device),
    KeyboardInput(State, u32),
    MouseMove,
    MouseMoveAbs,
    MouseButton,
    MouseAxis,
    TouchpadDown,
    TouchpadMotion,
    TouchpadUp,
    TouchpadCancel,
    TouchpadFrame,
    GestureSwipeBegin,
    GestureSwipeUpdate,
    GestureSwipeEnd,
    GesturePinchBegin,
    GesturePinchUpdate,
    GesturePinchEnd,
    TabletAxis,
    TabletProximity,
    TabletTip,
    TabletButton,
    TabletpadButton,
    TabletpadRing,
    TabletpadStrip,
}

#[derive(Clone, Debug)]
pub struct Device {
    name: String,
    physical_seat: String,
    logical_seat: String,
}

impl Device {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn physical_seat(&self) -> &String {
        &self.physical_seat
    }

    pub fn logical_seat(&self) -> &String {
        &self.logical_seat
    }
}

impl From<*mut libinput_event> for Device {
    fn from(event: *mut libinput_event) -> Device {
        let device = unsafe { libinput_event_get_device(event) };
	    let seat = unsafe { libinput_device_get_seat(device) };
        let dev_name = unsafe { libinput_device_get_name(device) };
        let phys_seat = unsafe { libinput_seat_get_physical_name(seat) };
        let log_seat = unsafe { libinput_seat_get_logical_name(seat) };

        Device {
            name: unsafe { cbuf_to_string(dev_name) },
            physical_seat: unsafe { cbuf_to_string(phys_seat) },
            logical_seat: unsafe { cbuf_to_string(log_seat) },
        }
    }
}
