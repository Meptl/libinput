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

#[derive(Copy, Clone, Debug)]
pub enum Source {
    Wheel,
    Finger,
    Continuous,
    WheelTilt,
}

#[derive(Clone, Copy, Debug)]
pub enum EventType {
    None,
    DeviceAdd,
    DeviceRemove,
    KeyboardInput(State, u32),
    MouseMove(f64, f64),
    MouseMoveAbsolute(f64, f64),
    MouseButton(State, u32),
    MouseAxis(Source, Option<f64>, Option<f64>),
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

pub struct Event {
    lib_handle: *mut libinput_event,
    device: Device,
    event_type: EventType,
}

impl Event {
    pub fn event_type(&self) -> EventType {
        self.event_type
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn time(&self) -> u64 {
        self.time_usec() / 1000
    }

    pub fn time_usec(&self) -> u64 {
        match self.event_type {
            EventType::KeyboardInput(_, _) => unsafe {
                let key_event = libinput_event_get_keyboard_event(self.lib_handle);
                libinput_event_keyboard_get_time_usec(key_event)
            },
            EventType::MouseMove(_, _)
            | EventType::MouseMoveAbsolute(_, _)
            | EventType::MouseButton(_, _)
            | EventType::MouseAxis(_, _, _) => unsafe {
                let mouse_event = libinput_event_get_pointer_event(self.lib_handle);
                libinput_event_pointer_get_time_usec(mouse_event)
            },
            _ => 0,
        }
    }
}

impl From<*mut libinput_event> for Event {
    fn from(event_handle: *mut libinput_event) -> Self {
        let libevent_type = unsafe { libinput_event_get_type(event_handle) };
        let device = Device::from(event_handle);
        let event_type = match libevent_type {
            libinput_event_type::LIBINPUT_EVENT_NONE => {
                unsafe { abort() };
                EventType::None
            },
            libinput_event_type::LIBINPUT_EVENT_DEVICE_ADDED => {
                EventType::DeviceAdd
            },
            libinput_event_type::LIBINPUT_EVENT_DEVICE_REMOVED => {
                EventType::DeviceRemove
            },
            libinput_event_type::LIBINPUT_EVENT_KEYBOARD_KEY => {
                let key_event = unsafe { libinput_event_get_keyboard_event(event_handle) };
                let key = unsafe { libinput_event_keyboard_get_key(key_event) };
                let key_state = {
                    match unsafe { libinput_event_keyboard_get_key_state(key_event) } {
                        libinput_key_state::LIBINPUT_KEY_STATE_PRESSED => State::Pressed,
                        libinput_key_state::LIBINPUT_KEY_STATE_RELEASED => State::Released,
                    }
                };

                EventType::KeyboardInput(key_state, key)
            },
            libinput_event_type::LIBINPUT_EVENT_POINTER_MOTION => {
                let mouse_event = unsafe { libinput_event_get_pointer_event(event_handle) };
                let x = unsafe { libinput_event_pointer_get_dx(mouse_event) };
                let y = unsafe { libinput_event_pointer_get_dy(mouse_event) };

                EventType::MouseMove(x, y)
            },
            libinput_event_type::LIBINPUT_EVENT_POINTER_MOTION_ABSOLUTE => {
                let mouse_event = unsafe { libinput_event_get_pointer_event(event_handle) };
                let x = unsafe { libinput_event_pointer_get_absolute_x(mouse_event) };
                let y = unsafe { libinput_event_pointer_get_absolute_y(mouse_event) };

                EventType::MouseMoveAbsolute(x, y)
            },
            libinput_event_type::LIBINPUT_EVENT_POINTER_BUTTON => {
                let mouse_event = unsafe { libinput_event_get_pointer_event(event_handle) };
                let button = unsafe { libinput_event_pointer_get_button(mouse_event) };
                let button_state = {
                    match unsafe { libinput_event_pointer_get_button_state(mouse_event) } {
                        libinput_button_state::LIBINPUT_BUTTON_STATE_PRESSED => State::Pressed,
                        libinput_button_state::LIBINPUT_BUTTON_STATE_RELEASED => State::Released,
                    }
                };

                EventType::MouseButton(button_state, button)
            },
            libinput_event_type::LIBINPUT_EVENT_POINTER_AXIS => {
                use libinput_pointer_axis::*;

                let mouse_event = unsafe { libinput_event_get_pointer_event(event_handle) };
                let source_type = {
                    match unsafe { libinput_event_pointer_get_axis_source(mouse_event) } {
                        LIBINPUT_POINTER_AXIS_SOURCE_WHEEL => Source::Wheel,
                        LIBINPUT_POINTER_AXIS_SOURCE_FINGER => Source::Finger,
                        LIBINPUT_POINTER_AXIS_SOURCE_CONTINUOUS => Source::Continuous,
                        LIBINPUT_POINTER_AXIS_SOURCE_WHEEL_TILT => Source::WheelTilt,
                    }
                };

                let vert = unsafe {
                    if libinput_event_pointer_has_axis(mouse_event, LIBINPUT_POINTER_AXIS_SCROLL_VERTICAL) != 0 {
                        Some(libinput_event_pointer_get_axis_value(mouse_event, LIBINPUT_POINTER_AXIS_SCROLL_VERTICAL))
                    }
                    else {
                        None
                    }
                };

                let hori = unsafe {
                    if libinput_event_pointer_has_axis(mouse_event, LIBINPUT_POINTER_AXIS_SCROLL_HORIZONTAL) != 0 {
                        Some(libinput_event_pointer_get_axis_value(mouse_event, LIBINPUT_POINTER_AXIS_SCROLL_HORIZONTAL))
                    }
                    else {
                        None
                    }
                };

                EventType::MouseAxis(source_type, vert, hori)
            },
            _ => {
                println!("Event type unimplemented.");
                EventType::None
            },
        };

        Event {
            lib_handle: event_handle,
            device: device,
            event_type: event_type
        }
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe { libinput_event_destroy(self.lib_handle) };
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    name: String,
    physical_seat: String,
    logical_seat: String,
}

impl Device {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn physical_seat(&self) -> &str {
        &self.physical_seat
    }

    pub fn logical_seat(&self) -> &str {
        &self.logical_seat
    }
}

impl From<*mut libinput_event> for Device {
    fn from(event: *mut libinput_event) -> Device {
        unsafe {
            let device = libinput_event_get_device(event);
            let seat = libinput_device_get_seat(device);
            let dev_name = libinput_device_get_name(device);
            let phys_seat = libinput_seat_get_physical_name(seat);
            let log_seat = libinput_seat_get_logical_name(seat);

            return Device {
                name: cbuf_to_string(dev_name),
                physical_seat: cbuf_to_string(phys_seat),
                logical_seat: cbuf_to_string(log_seat),
            }
        }
    }
}
