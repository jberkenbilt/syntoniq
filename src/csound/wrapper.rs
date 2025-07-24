use crate::{events, to_anyhow};
use anyhow::anyhow;
use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;
use tokio::task;
use tokio::task::JoinHandle;

mod cs {
    #![allow(clippy::all)]
    #![allow(unnecessary_transmutes)]
    #![allow(improper_ctypes)]
    #![allow(unused_imports)]
    #![allow(dead_code)]
    #![allow(trivial_casts)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/csound_bindings.rs"));
}

unsafe impl Sync for CsoundPtr {}
unsafe impl Send for CsoundPtr {}
#[repr(transparent)]
struct CsoundPtr(*mut cs::CSOUND);

pub struct CsoundApi {
    tx: flume::Sender<CsoundMessage>,
    performance_handle: Option<JoinHandle<()>>,
}

#[derive(Debug)]
enum CsoundMessage {
    Shutdown,
    InputMessage(String),
}

fn with_c_str<T>(s: &str, f: impl FnOnce(*const c_char) -> T) -> T {
    f(CString::new(s).unwrap().as_c_str().as_ptr())
}

extern "C" fn csound_message_callback(_: *mut cs::CSOUND, attr: c_int, msg: *const c_char) {
    let s = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    // Set RUST_LOG=qlaunchpad::csound::wrapper to see these messages.
    log::debug!("csound: {attr} {s}");
}

impl CsoundApi {
    pub async fn new(csound_file: &str, events_tx: events::WeakSender) -> anyhow::Result<Self> {
        let (tx, rx) = flume::unbounded();
        let csound = Self::start(csound_file)?;
        let h = task::spawn_blocking(|| {
            Self::main_loop(csound, rx, events_tx);
        });
        Ok(Self {
            tx,
            performance_handle: Some(h),
        })
    }

    pub async fn shutdown(mut self) {
        let _ = self.tx.send_async(CsoundMessage::Shutdown).await;
        if let Some(h) = self.performance_handle.take() {
            _ = h.await;
        }
    }

    pub async fn input_message<M: Into<String>>(&self, msg: M) -> anyhow::Result<()> {
        self.tx
            .send_async(CsoundMessage::InputMessage(msg.into()))
            .await
            .map_err(to_anyhow)
    }

    fn start(csound_file: &str) -> anyhow::Result<CsoundPtr> {
        unsafe {
            cs::csoundInitialize(
                (cs::CSOUNDINIT_NO_ATEXIT | cs::CSOUNDINIT_NO_SIGNAL_HANDLER) as c_int,
            );
            let csound = cs::csoundCreate(ptr::null_mut());
            cs::csoundSetMessageStringCallback(csound, Some(csound_message_callback));
            #[cfg(target_os = "linux")]
            cs::csoundSetOption(csound, c"-+rtaudio=pulse".as_ptr());

            let result = with_c_str(csound_file, |s| cs::csoundCompileCsdText(csound, s));
            if result != 0 {
                return Err(anyhow!("error compiling csound text"));
            }
            if cs::csoundStart(csound) != 0 {
                return Err(anyhow!("error starting csound"));
            }
            Ok(CsoundPtr(csound))
        }
    }

    fn main_loop(
        csound: CsoundPtr,
        rx: flume::Receiver<CsoundMessage>,
        events_tx: events::WeakSender,
    ) {
        'top: while unsafe { cs::csoundPerformKsmps(csound.0) } == 0 {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    CsoundMessage::Shutdown => break 'top,
                    CsoundMessage::InputMessage(data) => {
                        with_c_str(&data, |s| unsafe {
                            cs::csoundInputMessage(csound.0, s);
                        });
                    }
                }
            }
        }
        unsafe {
            // Calling csoundCleanup causes final diagnostics to be printed. Additional cleanup
            // is possible but unnecessary and may cause segmentation fault on exit.
            _ = cs::csoundCleanup(csound.0);
        }
        if let Some(tx) = events_tx.upgrade() {
            // This means csound stopped before we shut down
            log::warn!("csound has stopped");
            let _ = tx.send(events::Event::Shutdown);
        }
    }
}
