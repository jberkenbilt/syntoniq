use crate::events::{FromDevice, ToDevice};
use anyhow::bail;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::sync::Arc;
use syntoniq_common::to_anyhow;
use tokio::task::JoinHandle;

pub trait Device: Sync + Send + 'static {
    fn on_midi(&self, stamp_ms: u64, event: &[u8]) -> Option<FromDevice>;
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
    to_device: flume::Receiver<ToDevice>,
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

impl Controller {
    pub fn run(
        port_name: String,
        to_device_rx: flume::Receiver<ToDevice>,
        from_device_tx: flume::Sender<FromDevice>,
        device: Arc<dyn Device>,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        let mut controller = Self::new(&port_name, to_device_rx, from_device_tx, device.clone())
            .map_err(to_anyhow)?;
        // Ensure init is called synchronously before we return.
        device.init(&mut controller.output_connection)?;
        let handle: JoinHandle<anyhow::Result<()>> =
            tokio::task::spawn_blocking(move || controller.relay_to_device(device));
        Ok(handle)
    }

    pub fn new(
        port_name: &str,
        to_device_rx: flume::Receiver<ToDevice>,
        from_device_tx: flume::Sender<FromDevice>,
        device: Arc<dyn Device>,
    ) -> anyhow::Result<Self> {
        let midi_in = MidiInput::new("Syntoniq Keyboard")?;
        let in_port = find_port(&midi_in, port_name)?;
        let full_port_name = midi_in.port_name(&in_port)?;
        log::debug!("opening input port: {full_port_name}",);
        // Handler keeps running until connection is dropped
        let input_connection = midi_in
            .connect(
                &in_port,
                &format!("{} to Syntoniq Keyboard", in_port.id()),
                move |stamp_ms, message, _| {
                    if let Some(event) = device.on_midi(stamp_ms, message)
                        && let Err(e) = from_device_tx.send(event)
                    {
                        log::error!("error notifying of device event: {e}")
                    }
                },
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
            to_device: to_device_rx,
        })
    }

    fn relay_to_device(mut self, device: Arc<dyn Device>) -> anyhow::Result<()> {
        while let Ok(e) = self.to_device.recv() {
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
