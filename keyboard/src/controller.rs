use crate::events::{FromDevice, ToDevice};
use anyhow::bail;
use arc_swap::ArcSwap;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midly::live::LiveEvent;
use std::sync::{Arc, LazyLock};
use syntoniq_common::to_anyhow;
use tokio::task::JoinHandle;

struct DeviceData {
    from_device_tx: flume::Sender<FromDevice>,
    device: Arc<dyn Device>,
}

static DEVICE: LazyLock<ArcSwap<Option<DeviceData>>> =
    LazyLock::new(|| ArcSwap::from_pointee(None));

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
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: MidiOutputConnection,
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

fn on_midi(_stamp_ms: u64, event: &[u8], _data: &mut ()) {
    let Ok(event) = LiveEvent::parse(event) else {
        log::error!("invalid midi event received and ignored");
        return;
    };
    if let Some(d) = DEVICE.load().as_ref() {
        if let Some(event) = d.device.on_midi(event)
            && let Err(e) = d.from_device_tx.send(event)
        {
            log::error!("error notifying of device event: {e}")
        }
    } else {
        // TODO
    }
}

impl Controller {
    pub fn new(port_name: &str) -> anyhow::Result<Self> {
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
                (),
            )
            .map_err(to_anyhow)?;

        let midi_out = MidiOutput::new("Syntoniq Keyboard")?;
        let out_port = find_port(&midi_out, port_name)?;
        let full_port_name = midi_out.port_name(&out_port)?;
        log::debug!("opening output port: {full_port_name}");
        let output_connection = midi_out
            .connect(
                &out_port,
                &format!("Syntoniq Keyboard to {}", out_port.id()),
            )
            .map_err(to_anyhow)?;
        Ok(Self {
            input_connection: Some(input_connection),
            output_connection,
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
        DEVICE.store(Arc::new(Some(DeviceData {
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
