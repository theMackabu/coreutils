#![cfg_attr(feature = "bin", feature(start))]

#[cfg(feature = "bin")]
#[macro_use]
extern crate macros;
extern crate curl;
extern crate entry;

#[cfg(feature = "bin")]
extern crate prelude;

use prelude::*;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::io::{Read, SeekFrom, Write};
use std::os::raw::{c_char, c_double, c_int, c_long, c_void};
use std::{path::Path, slice};

const USAGE: &str = "Usage: curl [options...] <url>
 -d, --data <data>           HTTP POST data
 -f, --fail                  Fail fast with no output on HTTP errors
 -i, --include               Include response headers in output
 -o, --output <file>         Write to file instead of stdout
 -O, --remote-name           Write output to file named as remote file
 -v, --verbose               Verbose mode
 -T, --upload-file <file>    Transfer local FILE to destination
 -u, --user <user:password>  Server user and password
 -A, --user-agent <name>     Send User-Agent <name> to server";

pub const DESCRIPTION: &str = "Transfer data from or to a server";

use self::curl::{
    curl_easy_cleanup, curl_easy_init, curl_easy_perform, curl_easy_setopt, curl_easy_strerror, curl_slist, curl_socket_t, panic, size_t, CURLcode, CURLoption, Error, List, __enum_ty, CURL, CURLE_OK,
    CURLOPT_CONNECTTIMEOUT, CURLOPT_FOLLOWLOCATION, CURLOPT_HEADER, CURLOPT_HTTPHEADER, CURLOPT_MAXREDIRS, CURLOPT_POSTFIELDS, CURLOPT_PROXY, CURLOPT_READDATA, CURLOPT_READFUNCTION,
    CURLOPT_SSL_VERIFYHOST, CURLOPT_SSL_VERIFYPEER, CURLOPT_UPLOAD, CURLOPT_URL, CURLOPT_USERAGENT, CURLOPT_USERPWD, CURLOPT_VERBOSE, CURLOPT_WRITEDATA, CURLOPT_WRITEFUNCTION,
};

#[derive(Debug)]
enum CurlError {
    InitError,
    OptionError(CURLcode),
    PerformError(CURLcode),
}

#[derive(Clone, Copy)]
pub enum SeekResult {
    Ok = curl::CURL_SEEKFUNC_OK as isize,
    Fail = curl::CURL_SEEKFUNC_FAIL as isize,
    CantSeek = curl::CURL_SEEKFUNC_CANTSEEK as isize,
}

pub enum WriteError {
    Pause,
}

pub enum ReadError {
    Abort,
    Pause,
}

type CurlResult<T> = Result<T, CurlError>;
type Socket = curl::curl_socket_t;

struct Collector(Vec<u8>);

impl Handler for Collector {
    fn result(&self) -> Vec<u8> {
        self.0.to_owned()
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

trait Handler {
    fn result(&self) -> Vec<u8> {
        Vec::new()
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        Ok(data.len())
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        let _ = data;
        Ok(0)
    }

    fn seek(&mut self, whence: SeekFrom) -> SeekResult {
        let _ = whence;
        SeekResult::CantSeek
    }

    fn header(&mut self, data: &[u8]) -> bool {
        let _ = data;
        true
    }

    fn progress(&mut self, dltotal: f64, dlnow: f64, ultotal: f64, ulnow: f64) -> bool {
        let _ = (dltotal, dlnow, ultotal, ulnow);
        true
    }

    fn ssl_ctx(&mut self, cx: *mut c_void) -> Result<(), Error> {
        Ok(())
    }
}

impl std::error::Error for CurlError {}

impl std::fmt::Display for CurlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CurlError::InitError => write!(f, "Failed to initialize libcurl"),
            CurlError::OptionError(code) => unsafe {
                let msg = CStr::from_ptr(curl_easy_strerror(*code));
                write!(f, "Failed to set libcurl option: {}", msg.to_string_lossy())
            },
            CurlError::PerformError(code) => unsafe {
                let msg = CStr::from_ptr(curl_easy_strerror(*code));
                write!(f, "Failed to perform request: {}", msg.to_string_lossy())
            },
        }
    }
}

struct Options {
    data: Option<String>,
    fail_fast: bool,
    include_headers: bool,
    output: Option<String>,
    remote_name: bool,
    verbose: bool,
    upload_file: Option<String>,
    user: Option<String>,
    user_agent: Option<String>,
}

pub struct Client<H> {
    inner: Box<Inner<H>>,
}

struct Inner<H> {
    handle: *mut curl::CURL,
    header_list: Option<List>,
    resolve_list: Option<List>,
    connect_to_list: Option<List>,
    error_buf: RefCell<Vec<u8>>,
    handler: H,
}

unsafe impl<H: Send> Send for Inner<H> {}

impl<H> Drop for Client<H> {
    fn drop(&mut self) {
        unsafe {
            curl::curl_easy_cleanup(self.inner.handle);
        }
    }
}

impl<H: Handler> Client<H> {
    fn new(handler: H) -> Client<H> {
        unsafe {
            let handle = curl_easy_init();
            assert!(!handle.is_null());

            let mut ret = Client {
                inner: Box::new(Inner {
                    handle,
                    header_list: None,
                    resolve_list: None,
                    connect_to_list: None,
                    error_buf: RefCell::new(vec![0; curl::CURL_ERROR_SIZE]),
                    handler,
                }),
            };

            ret.default_configure();
            ret
        }
    }

    fn reset(&mut self) {
        unsafe {
            curl::curl_easy_reset(self.inner.handle);
        }
        self.default_configure();
    }

    fn default_configure(&mut self) {
        self.setopt_ptr(curl::CURLOPT_ERRORBUFFER, self.inner.error_buf.borrow().as_ptr() as *const _)
            .expect("failed to set error buffer");

        let _ = self.signal(false);
        let ptr = &*self.inner as *const _ as *const _;

        let cb: extern "C" fn(*mut c_char, size_t, size_t, *mut c_void) -> size_t = header_cb::<H>;
        self.setopt_ptr(curl::CURLOPT_HEADERFUNCTION, cb as *const _).expect("failed to set header callback");
        self.setopt_ptr(curl::CURLOPT_HEADERDATA, ptr).expect("failed to set header callback");

        let cb: curl::curl_write_callback = write_cb::<H>;
        self.setopt_ptr(curl::CURLOPT_WRITEFUNCTION, cb as *const _).expect("failed to set write callback");
        self.setopt_ptr(curl::CURLOPT_WRITEDATA, ptr).expect("failed to set write callback");

        let cb: curl::curl_read_callback = read_cb::<H>;
        self.setopt_ptr(curl::CURLOPT_READFUNCTION, cb as *const _).expect("failed to set read callback");
        self.setopt_ptr(curl::CURLOPT_READDATA, ptr).expect("failed to set read callback");

        let cb: curl::curl_seek_callback = seek_cb::<H>;
        self.setopt_ptr(curl::CURLOPT_SEEKFUNCTION, cb as *const _).expect("failed to set seek callback");
        self.setopt_ptr(curl::CURLOPT_SEEKDATA, ptr).expect("failed to set seek callback");

        let cb: curl::curl_progress_callback = progress_cb::<H>;
        self.setopt_ptr(curl::CURLOPT_PROGRESSFUNCTION, cb as *const _).expect("failed to set progress callback");
        self.setopt_ptr(curl::CURLOPT_PROGRESSDATA, ptr).expect("failed to set progress callback");

        let cb: curl::curl_ssl_ctx_callback = ssl_ctx_cb::<H>;
        drop(self.setopt_ptr(curl::CURLOPT_SSL_CTX_FUNCTION, cb as *const _));
        drop(self.setopt_ptr(curl::CURLOPT_SSL_CTX_DATA, ptr));
    }

    fn signal(&mut self, signal: bool) -> Result<(), Error> {
        self.setopt_long(curl::CURLOPT_NOSIGNAL, (!signal) as c_long)
    }

    fn setopt_path(&mut self, opt: curl::CURLoption, val: &Path) -> Result<(), Error> {
        use std::os::unix::prelude::*;
        let s = CString::new(val.as_os_str().as_bytes())?;
        self.setopt_str(opt, &s)
    }

    fn setopt_long(&mut self, opt: curl::CURLoption, val: c_long) -> Result<(), Error> {
        unsafe { self.cvt(curl_easy_setopt(self.inner.handle, opt, val)) }
    }

    fn setopt_str(&mut self, opt: curl::CURLoption, val: &CStr) -> Result<(), Error> {
        self.setopt_ptr(opt, val.as_ptr())
    }

    fn setopt_string(&mut self, opt: curl::CURLoption, data: &str) -> Result<(), Error> {
        self.setopt_str(opt, &CString::new(data)?)
    }

    fn useragent(&mut self, useragent: &str) -> Result<(), Error> {
        let useragent = CString::new(useragent)?;
        self.setopt_str(curl::CURLOPT_USERAGENT, &useragent)
    }

    fn setopt_ptr(&self, opt: CURLoption, val: *const c_char) -> Result<(), Error> {
        unsafe { self.cvt(curl_easy_setopt(self.inner.handle, opt, val)) }
    }

    fn setopt_off_t(&mut self, opt: curl::CURLoption, val: curl::curl_off_t) -> Result<(), Error> {
        unsafe {
            let rc = curl::curl_easy_setopt(self.inner.handle, opt, val);
            self.cvt(rc)
        }
    }

    fn setopt_blob(&mut self, opt: curl::CURLoption, val: &[u8]) -> Result<(), Error> {
        let blob = curl::curl_blob {
            data: val.as_ptr() as *const c_void as *mut c_void,
            len: val.len(),
            flags: curl::CURL_BLOB_COPY,
        };
        let blob_ptr = &blob as *const curl::curl_blob;
        unsafe { self.cvt(curl::curl_easy_setopt(self.inner.handle, opt, blob_ptr)) }
    }

    fn version(&self) -> Result<&'static str, std::str::Utf8Error> {
        let char_ptr = unsafe { curl::curl_version() };
        let c_str = unsafe { CStr::from_ptr(char_ptr) };
        c_str.to_str()
    }

    fn take_error_buf(&self) -> Option<String> {
        let mut buf = self.inner.error_buf.borrow_mut();
        if buf[0] == 0 {
            return None;
        }
        let pos = buf.iter().position(|i| *i == 0).unwrap_or(buf.len());
        let msg = String::from_utf8_lossy(&buf[..pos]).into_owned();
        buf[0] = 0;
        Some(msg)
    }

    fn cvt(&self, rc: curl::CURLcode) -> Result<(), Error> {
        if rc == curl::CURLE_OK {
            return Ok(());
        }
        let mut err = Error::new(rc);
        if let Some(msg) = self.take_error_buf() {
            err.set_extra(msg);
        }
        Err(err)
    }

    fn perform(&self) -> Result<(), Error> {
        let ret = unsafe { self.cvt(curl::curl_easy_perform(self.inner.handle)) };
        panic::propagate();
        ret
    }
}

fn send_request(url: &str, options: &Options) -> Result<(), Error> {
    let headers = CString::new("Accept: */*").expect("expected CString to be valid");
    let mut client = Client::new(Collector(Vec::new()));

    client.setopt_string(CURLOPT_URL, url)?;

    if let Some(ref data) = options.data {
        client.setopt_string(CURLOPT_POSTFIELDS, data)?;
    }

    if options.include_headers {
        client.setopt_long(CURLOPT_HEADER, 1)?;
    }

    // let result = curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, write_callbac2 as *const c_void);
    // if result != CURLE_OK {
    //     curl_easy_cleanup(handle);
    //     return Err(CurlError::OptionError(result));
    // }

    // let result = curl_easy_setopt(handle, CURLOPT_WRITEDATA, &mut response_data as *mut _ as *mut c_void);
    // if result != CURLE_OK {
    //     curl_easy_cleanup(handle);
    //     return Err(CurlError::OptionError(result));
    // }

    // if let Some(ref output) = options.output {
    //     let mut file = std::fs::File::create(output)?;
    //     unsafe {
    //         let file_ptr = &mut file as *mut _ as *mut c_void;
    //         curl_easy_setopt(handle, CURLOPT_WRITEDATA, file_ptr);
    //         curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, write_callback as *const c_void);
    //     }
    // } else {
    //     unsafe {
    //         curl_easy_setopt(handle, CURLOPT_WRITEDATA, std::ptr::null_mut::<c_void>());
    //         curl_easy_setopt(handle, CURLOPT_WRITEFUNCTION, stdout_callback as *const c_void);
    //     }
    // }

    // if let Some(ref upload_file) = options.upload_file {
    //     set_curl_option(handle, CURLOPT_UPLOAD, b"1")?;
    //     let mut file = std::fs::File::open(upload_file)?;
    //     unsafe {
    //         let file_ptr = &mut file as *mut _ as *mut c_void;
    //         curl_easy_setopt(handle, CURLOPT_READDATA, file_ptr);
    //         curl_easy_setopt(handle, CURLOPT_READFUNCTION, read_callback as *const c_void);
    //     }
    // }

    if options.remote_name {
        unimplemented!(); // handle writing to file with remote name
    }

    if let Some(ref user) = options.user {
        client.setopt_string(CURLOPT_USERPWD, user)?;
    }

    if let Some(ref user_agent) = options.user_agent {
        client.useragent(user_agent)?;
    } else {
        let user_agent = format!(
            "rs-coreutils/{} (Build {}; hash:{}) curl/{}",
            env!("PKG_VERSION"),
            env!("BUILD_DATE"),
            env!("GIT_HASH"),
            client.version()?
        );

        client.useragent(&user_agent)?;
    }

    client.setopt_long(CURLOPT_SSL_VERIFYPEER, 1)?;
    client.setopt_long(CURLOPT_SSL_VERIFYHOST, 2)?;

    if options.verbose {
        client.setopt_long(CURLOPT_VERBOSE, 1)?;
    }

    // curl_option! { handle => (CURLOPT_HTTPHEADER, headers.as_bytes()) };

    client.setopt_long(CURLOPT_FOLLOWLOCATION, 2)?;
    client.setopt_long(CURLOPT_MAXREDIRS, 5)?;
    client.setopt_long(CURLOPT_CONNECTTIMEOUT, 30)?;

    if let Ok(proxy) = std::env::var("HTTP_PROXY") {
        client.setopt_string(CURLOPT_PROXY, &proxy)?;
    }

    client.perform()?;

    Ok(io::stdout().write_all(&client.inner.handler.result())?)
}

extern "C" fn header_cb<H: Handler>(buffer: *mut c_char, size: size_t, nitems: size_t, userptr: *mut c_void) -> size_t {
    let keep_going = panic::catch(|| unsafe {
        let data = slice::from_raw_parts(buffer as *const u8, size * nitems);
        (*(userptr as *mut Inner<H>)).handler.header(data)
    })
    .unwrap_or(false);
    if keep_going {
        size * nitems
    } else {
        !0
    }
}

extern "C" fn write_cb<H: Handler>(ptr: *mut c_char, size: size_t, nmemb: size_t, data: *mut c_void) -> size_t {
    panic::catch(|| unsafe {
        let input = slice::from_raw_parts(ptr as *const u8, size * nmemb);
        match (*(data as *mut Inner<H>)).handler.write(input) {
            Ok(s) => s,
            Err(WriteError::Pause) => curl::CURL_WRITEFUNC_PAUSE,
        }
    })
    .unwrap_or(!0)
}

extern "C" fn read_cb<H: Handler>(ptr: *mut c_char, size: size_t, nmemb: size_t, data: *mut c_void) -> size_t {
    panic::catch(|| unsafe {
        let input = slice::from_raw_parts_mut(ptr as *mut u8, size * nmemb);
        match (*(data as *mut Inner<H>)).handler.read(input) {
            Ok(s) => s,
            Err(ReadError::Pause) => curl::CURL_READFUNC_PAUSE,
            Err(ReadError::Abort) => curl::CURL_READFUNC_ABORT,
        }
    })
    .unwrap_or(!0)
}

extern "C" fn seek_cb<H: Handler>(data: *mut c_void, offset: curl::curl_off_t, origin: c_int) -> c_int {
    panic::catch(|| unsafe {
        let from = if origin == 0 {
            SeekFrom::Start(offset as u64)
        } else {
            panic!("unknown origin from libcurl: {}", origin);
        };
        (*(data as *mut Inner<H>)).handler.seek(from) as c_int
    })
    .unwrap_or(!0)
}

extern "C" fn progress_cb<H: Handler>(data: *mut c_void, dltotal: c_double, dlnow: c_double, ultotal: c_double, ulnow: c_double) -> c_int {
    let keep_going = panic::catch(|| unsafe { (*(data as *mut Inner<H>)).handler.progress(dltotal, dlnow, ultotal, ulnow) }).unwrap_or(false);
    if keep_going {
        0
    } else {
        1
    }
}

extern "C" fn ssl_ctx_cb<H: Handler>(_handle: *mut curl::CURL, ssl_ctx: *mut c_void, data: *mut c_void) -> curl::CURLcode {
    let res = panic::catch(|| unsafe {
        match (*(data as *mut Inner<H>)).handler.ssl_ctx(ssl_ctx) {
            Ok(()) => curl::CURLE_OK,
            Err(e) => e.code(),
        }
    });
    res.unwrap_or(curl::CURLE_SSL_CONNECT_ERROR)
}

#[entry::gen(cfg = ["bin", "mut"])]
fn entry() -> ! {
    let mut options = Options {
        data: None,
        fail_fast: false,
        include_headers: false,
        output: None,
        remote_name: false,
        verbose: false,
        upload_file: None,
        user: None,
        user_agent: None,
    };

    let mut url = String::new();

    argument! {
        args: args.to_owned(),
        options: {
            d => options.data = Some(String::from_utf8_lossy(args.next().unwrap_or_else(|| usage!("curl: option requires an argument -- 'd'"))).into_owned()),
            f => options.fail_fast = true,
            i => options.include_headers = true,
            o => options.output = Some(String::from_utf8_lossy(args.next().unwrap_or_else(|| usage!("curl: option requires an argument -- 'o'"))).into_owned()),
            O => options.remote_name = true,
            v => options.verbose = true,
            T => options.upload_file = Some(String::from_utf8_lossy(args.next().unwrap_or_else(|| usage!("curl: option requires an argument -- 'T'"))).into_owned()),
            u => options.user = Some(String::from_utf8_lossy(args.next().unwrap_or_else(|| usage!("curl: option requires an argument -- 'u'"))).into_owned()),
            A => options.user_agent = Some(String::from_utf8_lossy(args.next().unwrap_or_else(|| usage!("curl: option requires an argument -- 'A'"))).into_owned())
        },
        command: |arg| url = String::from_utf8_lossy(arg).into_owned(),
        on_invalid: |arg| usage!("curl: invalid option -- '{}'", arg as char)
    }

    if url.is_empty() {
        usage!("curl: missing URL");
    }

    if let Err(e) = send_request(&url, &options) {
        if options.fail_fast {
            std::process::exit(1);
        } else {
            error!("curl: request failed: {}", e);
        }
    }
}
