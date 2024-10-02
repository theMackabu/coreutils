use std::ffi::CStr;
use std::ffi::{c_char, c_int, c_long};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DateTime {
    timestamp: i64,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    day_of_year: u32,
    day_of_week: u32,
    is_dst: bool,
    timezone: String,
}

#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

#[link(name = "c")]
extern "C" {
    fn gmtime_r(time_p: *const i64, result: *mut tm) -> *mut tm;
    fn localtime_r(time_p: *const i64, result: *mut tm) -> *mut tm;
}

impl DateTime {
    pub fn from_secs(timestamp: i64, utc: bool) -> DateTime {
        let mut timezone = "UTC".to_string();
        let mut tm: tm = unsafe { std::mem::zeroed() };

        if utc {
            unsafe { gmtime_r(&timestamp, &mut tm) };
        } else {
            timezone = unsafe {
                localtime_r(&timestamp, &mut tm);
                let tm_zone = tm.tm_zone;
                let timezone_cstr = CStr::from_ptr(tm_zone);
                timezone_cstr.to_str().unwrap_or("UTC").to_string()
            };
        }

        DateTime {
            timestamp,
            timezone,
            year: tm.tm_year + 1900,
            month: (tm.tm_mon + 1) as u32,
            day: tm.tm_mday as u32,
            hour: tm.tm_hour as u32,
            minute: tm.tm_min as u32,
            second: tm.tm_sec as u32,
            day_of_year: tm.tm_yday as u32 + 1,
            day_of_week: tm.tm_wday as u32 + 1,
            is_dst: tm.tm_isdst > 0,
        }
    }

    pub fn now(utc: bool) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let timestamp = now.as_secs() as i64;

        let mut timezone = "UTC".to_string();
        let mut tm: tm = unsafe { std::mem::zeroed() };

        if utc {
            unsafe { gmtime_r(&timestamp, &mut tm) };
        } else {
            timezone = unsafe {
                localtime_r(&timestamp, &mut tm);
                let tm_zone = tm.tm_zone;
                let timezone_cstr = CStr::from_ptr(tm_zone);
                timezone_cstr.to_str().unwrap_or("UTC").to_string()
            };
        }

        DateTime {
            timestamp,
            timezone,
            year: tm.tm_year + 1900,
            month: (tm.tm_mon + 1) as u32,
            day: tm.tm_mday as u32,
            hour: tm.tm_hour as u32,
            minute: tm.tm_min as u32,
            second: tm.tm_sec as u32,
            day_of_year: tm.tm_yday as u32 + 1,
            day_of_week: tm.tm_wday as u32 + 1,
            is_dst: tm.tm_isdst > 0,
        }
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn month(&self) -> u32 {
        self.month
    }

    pub fn day(&self) -> u32 {
        self.day
    }

    pub fn hour(&self) -> u32 {
        self.hour
    }

    pub fn minute(&self) -> u32 {
        self.minute
    }

    pub fn second(&self) -> u32 {
        self.second
    }

    pub fn day_of_year(&self) -> u32 {
        self.day_of_year
    }

    pub fn day_of_week(&self) -> u32 {
        self.day_of_week
    }

    pub fn dst(&self) -> bool {
        self.is_dst
    }

    pub fn format(&self, fmt: &str) -> String {
        fmt.replace("%D", &format!("{:02}/{:02}/{:02}", self.month(), self.day(), self.year() % 100))
            .replace("%Y", &self.year().to_string())
            .replace("%m", &format!("{:02}", self.month()))
            .replace("%B", &MONTH_NAMES[self.month() as usize - 1])
            .replace("%b", &SHORT_MONTH_NAMES[self.month() as usize - 1])
            .replace("%d", &format!("{:02}", self.day()))
            .replace("%r", &format!("{}{}", if self.day() < 10 { " " } else { "" }, self.day()))
            .replace("%j", &format!("{:03}", self.day_of_year()))
            .replace("%u", &self.day_of_week().to_string())
            .replace("%A", &WEEKDAY_NAMES[self.day_of_week() as usize - 2])
            .replace("%a", &SHORT_WEEKDAY_NAMES[self.day_of_week() as usize - 2])
            .replace("%H", &format!("{:02}", self.hour()))
            .replace("%I", &format!("{:02}", (self.hour() % 12).max(1)))
            .replace("%M", &format!("{:02}", self.minute()))
            .replace("%S", &format!("{:02}", self.second()))
            .replace("%Z", &self.timezone)
    }

    pub fn add_days(&mut self, days: i64) {
        self.timestamp += days * 86400;
    }
}

const MONTH_NAMES: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const SHORT_MONTH_NAMES: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const WEEKDAY_NAMES: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const SHORT_WEEKDAY_NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
