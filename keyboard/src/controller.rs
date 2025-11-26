use crate::DeviceType;
use crate::events::{FromDevice, ToDevice};
use anyhow::bail;
use arc_swap::ArcSwap;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midly::live::LiveEvent;
use midly::live::SystemCommon::SysEx;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use syntoniq_common::to_anyhow;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

enum DeviceData {
    WaitingForInit {
        id_tx: Mutex<Option<oneshot::Sender<DeviceType>>>,
    },
    Identified(LiveDevice),
}

struct LiveDevice {
    from_device_tx: flume::Sender<FromDevice>,
    device: Arc<dyn Device>,
}

pub trait Device: Sync + Send + 'static {
    fn on_midi(&self, event: LiveEvent) -> Option<FromDevice>;
    fn handle_event(
        &self,
        event: ToDevice,
        output_connection: &mut MidiOutputConnection,
    ) -> anyhow::Result<()>;
    fn init(&self, output_connection: &mut MidiOutputConnection) -> anyhow::Result<()>;
    fn shutdown(&self, output_connection: &mut MidiOutputConnection);
}

pub struct Controller {
    input_connection: Option<MidiInputConnection<Arc<ArcSwap<DeviceData>>>>,
    output_connection: MidiOutputConnection,
    device: Arc<ArcSwap<DeviceData>>,
}

pub(crate) fn find_port<T: MidiIO>(ports: &T, name: &str) -> anyhow::Result<T::Port> {
    let mut port_names = Vec::new();
    let result = ports.ports().into_iter().find(|p| {
        ports
            .port_name(p)
            .inspect(|n| {
                port_names.push(n.clone());
            })
            .map(|n| n.contains(name))
            .unwrap_or(false)
    });
    match result {
        None => {
            if port_names.is_empty() {
                eprintln!("no valid ports found");
            } else {
                eprintln!("Valid ports:");
                for p in port_names {
                    println!(" {p}");
                }
            }
            bail!("no port found containing '{name}'");
        }
        Some(r) => Ok(r),
    }
}

fn identify_device(sys_ex: Vec<u8>) -> Option<DeviceType> {
    // Response is 7E ?? 06 02 manufacturer-ID.
    // Manufacturer ID is XX or 00 XX YY. 0x7D is reserved for education/development.
    // Then we have family (2 bytes), model (two bytes), version (4 bytes), all LSB-first.
    log::trace!("identity candidate sysex: {sys_ex:?}");

    // My MK3 pro returns 00, 00 as model and 00, 04, 08, 03 as version.
    static LAUNCHPAD: &[u8] = &[
        0, 0x20, 0x29, // novation
        0x23, 0x01, // returned by my MK3 pro
    ];

    if sys_ex.len() < 5 {
        return None;
    }
    if sys_ex[0] != 0x7E || sys_ex[2] != 0x06 || sys_ex[3] != 0x02 {
        // Not an identity response
        return None;
    }
    let id = &sys_ex[4..];
    if id.starts_with(LAUNCHPAD) {
        Some(DeviceType::Launchpad)
    } else {
        Some(DeviceType::Empty)
    }
}

fn on_midi(_stamp_ms: u64, event: &[u8], device: &mut Arc<ArcSwap<DeviceData>>) {
    let Ok(event) = LiveEvent::parse(event) else {
        log::error!("invalid midi event received and ignored");
        return;
    };
    if matches!(event, LiveEvent::Realtime(_)) {
        return;
    }
    match device.load().as_ref() {
        DeviceData::WaitingForInit { id_tx } => {
            if let LiveEvent::Common(SysEx(sys_ex)) = event
                && let Some(identity) =
                    identify_device(sys_ex.iter().copied().map(u8::from).collect())
                && let Some(tx) = id_tx.lock().unwrap().take()
            {
                let _ = tx.send(identity);
            }
        }
        DeviceData::Identified(d) => {
            if let Some(event) = d.device.on_midi(event)
                && let Err(e) = d.from_device_tx.send(event)
            {
                log::error!("error notifying of device event: {e}")
            }
        }
    }
}

impl Controller {
    pub fn new(port_name: &str, id_tx: oneshot::Sender<DeviceType>) -> anyhow::Result<Self> {
        let device = Arc::new(ArcSwap::new(Arc::new(DeviceData::WaitingForInit {
            id_tx: Mutex::new(Some(id_tx)),
        })));
        let d2 = device.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(2));
            if let DeviceData::WaitingForInit { id_tx } = d2.load().as_ref()
                && let Some(tx) = id_tx.lock().unwrap().take()
            {
                let _ = tx.send(DeviceType::Empty);
            }
        });
        let midi_in = MidiInput::new("Syntoniq Keyboard")?;
        let in_port = find_port(&midi_in, port_name)?;
        let full_port_name = midi_in.port_name(&in_port)?;
        log::debug!("opening input port: {full_port_name}",);
        // Handler keeps running until connection is dropped
        let input_connection = midi_in
            .connect(
                &in_port,
                &format!("{} to Syntoniq Keyboard", in_port.id()),
                on_midi,
                device.clone(),
            )
            .map_err(to_anyhow)?;

        let midi_out = MidiOutput::new("Syntoniq Keyboard")?;
        let out_port = find_port(&midi_out, port_name)?;
        let full_port_name = midi_out.port_name(&out_port)?;
        log::debug!("opening output port: {full_port_name}");
        let mut output_connection = midi_out
            .connect(
                &out_port,
                &format!("Syntoniq Keyboard to {}", out_port.id()),
            )
            .map_err(to_anyhow)?;

        // Ask the device to identify itself.
        output_connection.send(&[0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7])?;
        Ok(Self {
            input_connection: Some(input_connection),
            output_connection,
            device,
        })
    }

    pub fn run(
        mut self,
        to_device_rx: flume::Receiver<ToDevice>,
        from_device_tx: flume::Sender<FromDevice>,
        device: Arc<dyn Device>,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        // Ensure init is called synchronously before we return.
        device.init(&mut self.output_connection)?;
        self.device
            .store(Arc::new(DeviceData::Identified(LiveDevice {
                from_device_tx,
                device: device.clone(),
            })));
        let handle: JoinHandle<anyhow::Result<()>> =
            tokio::task::spawn_blocking(move || self.relay_to_device(to_device_rx, device));
        Ok(handle)
    }

    fn relay_to_device(
        mut self,
        to_device_rx: flume::Receiver<ToDevice>,
        device: Arc<dyn Device>,
    ) -> anyhow::Result<()> {
        while let Ok(e) = to_device_rx.recv() {
            device.handle_event(e, &mut self.output_connection)?;
        }
        log::debug!("device received shutdown request");
        // Dropping the input connection triggers the series events that leads
        // to clean shutdown: the on_midi loop closes, which closes the transmit
        // side of from_device, which causes all subscribers to exit.
        self.input_connection.take();
        device.shutdown(&mut self.output_connection);
        Ok(())
    }
}
