extern crate libc;

use libc::{c_char, c_void};
use std::ffi::CString;
use std::fmt;
use std::mem::forget;

#[allow(non_snake_case)]
#[repr(C)]
pub struct DriverData {
    SOFCnt: u32,      // Number of SOF bytes received
    ACKWaiting: u32,  // Number of unsolicited messages while waiting for an ACK
    readAborts: u32,  // Number of times read were aborted due to timeouts
    badChecksum: u32, // Number of bad checksums
    readCnt: u32,     // Number of messages successfully read
    writeCnt: u32,    // Number of messages successfully sent
    CANCnt: u32,      // Number of CAN bytes received
    NAKCnt: u32,      // Number of NAK bytes received
    ACKCnt: u32,      // Number of ACK bytes received
    OOFCnt: u32,      // Number of bytes out of framing
    dropped: u32,     // Number of messages dropped & not delivered
    retries: u32,     // Number of messages retransmitted
    callbacks: u32,   // Number of unexpected callbacks
    badroutes: u32,   // Number of failed messages due to bad route response
    noack: u32,       // Number of no ACK returned errors
    netbusy: u32,     // Number of network busy/failure messages
    notidle: u32,
    nondelivery: u32,       // Number of messages not delivered to network
    routedbusy: u32,        // Number of messages received with routed busy status
    broadcastReadCnt: u32,  // Number of broadcasts read
    broadcastWriteCnt: u32, // Number of broadcasts sent
}

#[derive(Debug)]
#[repr(C)]
pub enum NotificationType {
    ValueAdded,
    ValueRemoved,
    ValueChanged,
    ValueRefreshed,
    Group,
    NodeNew,
    NodeAdded,
    NodeRemoved,
    NodeProtocolInfo,
    NodeNaming,
    NodeEvent,
    PollingDisabled,
    PollingEnabled,
    SceneEvent,
    CreateButton,
    DeleteButton,
    ButtonOn,
    ButtonOff,
    DriverReady,
    DriverFailed,
    DriverReset,
    EssentialNodeQueriesComplete,
    NodeQueriesComplete,
    AwakeNodeQueries,
    AllNodeQueriedSomeDead,
    AllNodeQueried,
    Notification,
    DriverRemoved,
    ControllerCommand,
    NodeReset,
}
#[derive(Debug)]
#[repr(C)]
pub enum NotificationCode {
    MsgComplete,
    Timeout,
    NoOperation,
    Awake,
    Sleep,
    Dead,
    Alive,
}

#[link(name = "openzwave")]
extern "C" {
    fn Notification_GetType(n: *const Notification) -> NotificationType;
    fn Notification_GetNodeId(n: *const Notification) -> u8;
    fn Notification_GetHomeId(n: *const Notification) -> u32;
    fn Notification_GetValueID(n: *const Notification, v: *const ValueID);
    fn Notification_GetNotification(n: *const Notification) -> NotificationCode;
    fn Manager_GetValueLabel(v: *const ValueID) -> *mut c_char;
    fn Manager_GetNodeName(homeId: u32, nodeId: u8) -> *mut c_char;
    fn Manager_GetNodeManufacturerName(homeId: u32, nodeId: u8) -> *mut c_char;
    fn Manager_StartInit(
        port: *const c_char,
        cbk: Option<unsafe extern "C" fn(*const Notification, *mut c_void)>,
        ctx: *mut c_void,
    );
    fn Manager_SetValueBool(v: *const ValueID, state: bool);
    fn Manager_GetValueAsString(v: *const ValueID) -> *mut c_char;
}

macro_rules! from_c_str {
    ($x:expr) => {{
        let cstr = $x;
        if cstr.is_null() {
            None
        } else {
            let string = CString::from_raw(cstr).to_string_lossy().to_string();
            if &string == "" {
                None
            } else {
                Some(string)
            }
        }
    }};
}

pub enum Notification {}
impl Notification {
    pub fn get_type(&self) -> NotificationType {
        unsafe { Notification_GetType(&*self) }
    }
    pub fn node_id(&self) -> u8 {
        unsafe { Notification_GetNodeId(&*self) }
    }
    pub fn home_id(&self) -> u32 {
        unsafe { Notification_GetHomeId(&*self) }
    }
    pub fn node_name(&self) -> Option<String> {
        unsafe { from_c_str!(Manager_GetNodeName(self.home_id(), self.node_id())) }
    }
    pub fn code(&self) -> Result<NotificationCode, ()> {
        match self.get_type() {
            NotificationType::Notification | NotificationType::ControllerCommand => {
                Ok(unsafe { Notification_GetNotification(&*self) })
            }
            _ => Err(()),
        }
    }
    pub fn node_manufacturer_name(&self) -> Option<String> {
        unsafe {
            from_c_str!(Manager_GetNodeManufacturerName(
                self.home_id(),
                self.node_id()
            ))
        }
    }
    pub fn value_id(&self) -> ValueID {
        unsafe {
            let v = ValueID::new();
            Notification_GetValueID(&*self, &v);
            v
        }
    }
}
///
/// Value Genres
/// The classification of a value to enable low level system or configuration parameters to be filtered by the application.
///
#[derive(Debug)]
#[repr(C)]
pub enum ValueGenre {
    Basic, /* < The 'level' as controlled by basic commands.  Usually duplicated by another command class. */
    User,  /* < Basic values an ordinary user would be interested in. */
    Config, /* < Device-specific configuration parameters.  These cannot be automatically discovered via Z-Wave, and are usually described in the user manual instead. */
    System, /* < Values of significance only to users who understand the Z-Wave protocol */
}

///
/// Value Types
///The type of data represented by the value object.
///
#[derive(Debug)]
#[repr(C)]
pub enum ValueType {
    Bool,     /* < Boolean, true or false */
    Byte,     /* < 8-bit unsigned value */
    Decimal, /* < Represents a non-integer value as a string, to avoid floating point accuracy issues. */
    Int,     /* < 32-bit signed value */
    List,    /* < List from which one item can be selected */
    Schedule, /* < Complex type used with the Climate Control Schedule command class */
    Short,   /* < 16-bit signed value */
    String,  /* < Text string */
    Button, /* < A write-only value that is the equivalent of pressing a button to send a command to a device */
    Raw,    /* < A collection of bytes */
}

#[repr(C)]
pub struct ValueID {
    pub home_id: u32,
    pub node_id: u8,
    pub genre: ValueGenre,
    pub command_class_id: u8,
    pub instance: u8,
    pub value_index: u8,
    pub value_type: ValueType,
}

impl ValueID {
    pub fn new() -> ValueID {
        Self {
            home_id: 0,
            node_id: 0,
            genre: ValueGenre::Basic,
            command_class_id: 0,
            instance: 0,
            value_index: 0,
            value_type: ValueType::Bool,
        }
    }
    pub fn label(&self) -> String {
        unsafe {
            let label = Manager_GetValueLabel(&*self);
            if label.is_null() {
                "unamed".to_string()
            } else {
                CString::from_raw(label).to_string_lossy().to_string()
            }
        }
    }
    pub fn set_bool(&self, state: bool) {
        unsafe {
            Manager_SetValueBool(&*self, state);
        }
    }
    pub fn get_string(&self) -> String {
        unsafe {
            let v = Manager_GetValueAsString(&*self);
            if v.is_null() {
                panic!("woops");
            } else {
                CString::from_raw(v).to_string_lossy().to_string()
            }
        }
    }
}
impl fmt::Debug for ValueID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ValueID(node_id: {}, genre: {:?}, cmd_class_id: {}, instance: {}, type: {:?}, index: {}, label: '{}')",
            self.node_id,
            self.genre,
            self.command_class_id,
            self.instance,
            self.value_type,
            self.value_index,
            self.label()
        )
    }
}
#[link(name = "stdc++")]
extern "C" {}

type Handler = dyn Fn(&Notification) + 'static;

unsafe extern "C" fn handler(n: *const Notification, ctx: *mut c_void) {
    let cbk = Box::from_raw(ctx as *mut Box<Handler>);
    cbk(&*n);
    forget(cbk); // we don't actually want to free the closure.
}

pub fn init<F: Fn(&Notification) + 'static>(port: &str, cbk: F) {
    unsafe {
        let p = CString::new(port).unwrap();

        // The type here 'triggers' a conversion from F to the Trait object
        // Fn<(*const Notification) + 'static
        //                             ! thin pointer to the inner box
        //                             !        ! fat pointer to the function/closure
        let cbk: Box<Box<Handler>> = Box::new(Box::new(cbk));
        Manager_StartInit(p.as_ptr(), Some(handler), Box::into_raw(cbk) as *mut c_void);
    }
}
