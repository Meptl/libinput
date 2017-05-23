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

pub struct Button {
    state: State,
    key: u32
}

pub struct Motion {
    x: f64,
    y: f64
}

#[derive(Clone, Copy, Debug)]
pub enum EventType {
    None,
    DeviceAdd,
    DeviceRemove,
    Keyboard(Button),
    MouseMove(Motion),
    MouseMoveAbsolute(Motion)
    MouseButton(Button)
    MouseAxis(Source, Option<f64>, Option<f64>),
    TouchDown(f64, f64),
    TouchMotion(f64, f64),
    TouchUp,
    TouchCancel,
    TouchFrame,
    GestureSwipeBegin(u8),
    GestureSwipeUpdate(u8, f64, f64, f64, f64),
    GestureSwipeEnd(u8, bool),
    GesturePinchBegin(u8),
    GesturePinchUpdate(u8, f64, f64, f64, f64, f64, f64),
    GesturePinchEnd(u8, bool),
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
            libinput_event_type::LIBINPUT_TOUCH_DOWN => {
                unsafe {
                    let touch_event = libinput_event_get_touch_event(event_handle);
                    //let x = libinput_event_touch_get_x_transformed(t, screen_width);
                    //let y = libinput_event_touch_get_y_transformed(t, screen_height);
                    let x = libinput_event_touch_get_x(touch_event);
                    let y = libinput_event_touch_get_y(touch_event);

                    EventType::TouchDown(x, y)
                }
            },
            libinput_event_type::LIBINPUT_TOUCH_MOTION => {
                unsafe {
                    let touch_event = libinput_event_get_touch_event(event_handle);
                    let x = libinput_event_touch_get_x(touch_event);
                    let y = libinput_event_touch_get_y(touch_event);

                    EventType::TouchMotion(x, y)
                }
            },
            libinput_event_type::LIBINPUT_TOUCH_UP => EventType::TouchUp,
            libinput_event_type::LIBINPUT_TOUCH_CANCEL => EventType::TouchCancel,
            libinput_event_type::LIBINPUT_TOUCH_FRAME => EventType::TouchFrame,
	        libinput_event_type::LIBINPUT_EVENT_GESTURE_SWIPE_BEGIN => {
                let gesture_event = unsafe { libinput_event_get_gesture_event(event_handle) };
                let fingers = unsafe { libinput_event_gesture_get_finger_count(gesture_event) };
                EventType::GestureSwipeBegin(fingers)
            },
	        libinput_event_type::LIBINPUT_EVENT_GESTURE_SWIPE_UPDATE => {
                unsafe {
                    let gesture_event = libinput_event_get_gesture_event(event_handle);
                    let fingers = unsafe { libinput_event_gesture_get_finger_count(gesture_event) };
                    let dx = libinput_event_gesture_get_dx(gesture_event);
                    let dy = libinput_event_gesture_get_dy(gesture_event);
                    let dx_unaccel = libinput_event_gesture_get_dx_unaccelerated(gesture_event);
                    let dy_unaccel = libinput_event_gesture_get_dy_unaccelerated(gesture_event);

                    EventType::GestureSwipeUpdate(fingers, dx, dy, dx_unaccel, dy_unaccel)
                }
            },
	        libinput_event_type::LIBINPUT_EVENT_GESTURE_SWIPE_END => {
                let gesture_event = unsafe { libinput_event_get_gesture_event(event_handle) };
                let fingers = unsafe { libinput_event_gesture_get_finger_count(gesture_event) };
                let cancelled = unsafe { libinput_event_gesture_get_cancelled(gesture_event) };
                EventType::GestureSwipeEnd(fingers, cancelled)
            },
	        libinput_event_type::LIBINPUT_EVENT_GESTURE_PINCH_BEGIN => {
                let gesture_event = unsafe { libinput_event_get_gesture_event(event_handle) };
                let fingers = unsafe { libinput_event_gesture_get_finger_count(gesture_event) };
                EventType::GesturePinchEnd(fingers)
            },
	        libinput_event_type::LIBINPUT_EVENT_GESTURE_PINCH_UPDATE => {
                unsafe {
                    let gesture_event = libinput_event_get_gesture_event(event_handle);
                    let fingers = unsafe { libinput_event_gesture_get_finger_count(gesture_event) };
                    let dx = libinput_event_gesture_get_dx(gesture_event);
                    let dy = libinput_event_gesture_get_dy(gesture_event);
                    let dx_unaccel = libinput_event_gesture_get_dx_unaccelerated(gesture_event);
                    let dy_unaccel = libinput_event_gesture_get_dy_unaccelerated(gesture_event);
                    let scale = libinput_event_gesture_get_scale(gesture_event);
                    let angle = libinput_event_gesture_get_angle_delta(gesture_event);

                    EventType::GestureSwipeUpdate(fingers, dx, dy, dx_unaccel, dy_unaccel, scale, angle)
                }
            },
            fn tablet_event_axes(libinput_event_tablet_tool) {
                unsafe {
                    let tool = libinput_event_tablet_tool_get_tool(tablet_event);
                    let x = libinput_event_tablet_tool_get_x(t);
                    let y = libinput_event_tablet_tool_get_y(t);

                    if libinput_tablet_tool_has_tilt(tool) {
                        x = libinput_event_tablet_tool_get_tilt_x(t);
                        y = libinput_event_tablet_tool_get_tilt_y(t);
                        printq("\ttilt: %.2f%s/%.2f%s",
                               x, changed_sym(t, tilt_x),
                               y, changed_sym(t, tilt_y));
                    }

                    if (libinput_tablet_tool_has_distance(tool) ||
                        libinput_tablet_tool_has_pressure(tool)) {
                        dist = libinput_event_tablet_tool_get_distance(t);
                        pressure = libinput_event_tablet_tool_get_pressure(t);
                        if (dist)
                            printq("\tdistance: %.2f%s",
                                   dist, changed_sym(t, distance));
                        else
                            printq("\tpressure: %.2f%s",
                                   pressure, changed_sym(t, pressure));
                    }

                    if (libinput_tablet_tool_has_rotation(tool)) {
                        rotation = libinput_event_tablet_tool_get_rotation(t);
                        printq("\trotation: %.2f%s",
                               rotation, changed_sym(t, rotation));
                    }

                    if (libinput_tablet_tool_has_slider(tool)) {
                        slider = libinput_event_tablet_tool_get_slider_position(t);
                        printq("\tslider: %.2f%s",
                               slider, changed_sym(t, slider));
                    }

                    if (libinput_tablet_tool_has_wheel(tool)) {
                        wheel = libinput_event_tablet_tool_get_wheel_delta(t);
                        delta = libinput_event_tablet_tool_get_wheel_delta_discrete(t);
                        printq("\twheel: %.2f%s (%d)",
                               wheel, changed_sym(t, wheel),
                               (int)delta);
                    }
                }
            }
	        libinput_event_type::LIBINPUT_EVENT_GESTURE_PINCH_END => {
                let gesture_event = unsafe { libinput_event_get_gesture_event(event_handle) };
                let fingers = unsafe { libinput_event_gesture_get_finger_count(gesture_event) };
                let cancelled = unsafe { libinput_event_gesture_get_cancelled(gesture_event) };
                EventType::GesturePinchEnd(fingers, cancelled)
            },
	        libinput_event_type::LIBINPUT_EVENT_TABLET_TOOL_AXIS => EventType::Gesture,
	        libinput_event_type::LIBINPUT_EVENT_TABLET_TOOL_PROXIMITY => EventType::Gesture,
	        libinput_event_type::LIBINPUT_EVENT_TABLET_TOOL_TIP => EventType::Gesture,
	        libinput_event_type::LIBINPUT_EVENT_TABLET_TOOL_BUTTON => EventType::Gesture,
	        libinput_event_type::LIBINPUT_EVENT_TABLET_PAD_BUTTON => EventType::Gesture,
	        libinput_event_type::LIBINPUT_EVENT_TABLET_PAD_RING => EventType::Gesture,
	        libinput_event_type::LIBINPUT_EVENT_TABLET_PAD_STRIP => EventType::Gesture,
	        libinput_event_type::LIBINPUT_EVENT_SWITCH_TOGGLE => EventType::Gesture,
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
