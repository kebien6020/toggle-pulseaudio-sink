use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct PulseAudioInfo {
    default_sink_name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct PulseAudioSink {
    index: u32,
    name: String,
    ports: Vec<SinkPort>,
}

#[derive(Debug, Deserialize, Clone)]
struct SinkPort {
    availability: String,
}

fn get_pactl_info() -> PulseAudioInfo {
    let output = Command::new("pactl")
        .arg("-f")
        .arg("json")
        .arg("info")
        .output()
        .expect("failed to execute pactl info command");

    let pactl_info: PulseAudioInfo =
        serde_json::from_slice(&output.stdout).expect("no valid json from pactl info");

    pactl_info
}

fn get_pactl_sinks() -> Vec<PulseAudioSink> {
    let output = Command::new("pactl")
        .arg("-f")
        .arg("json")
        .arg("list")
        .arg("sinks")
        .output()
        .expect("failed to execute pactl list sinks command");

    let pactl_sinks: Vec<PulseAudioSink> =
        serde_json::from_slice(&output.stdout).expect("no valid json from pactl list sinks");

    pactl_sinks
}

fn get_current_active_sink(sinks: &[PulseAudioSink]) -> &PulseAudioSink {
    let current_sink_name = get_pactl_info().default_sink_name;

    let current_sink = sinks
        .iter()
        .find(|sink| sink.name == current_sink_name)
        .expect("no active sink found");

    current_sink
}

fn get_sink_with_next_index<'a>(
    current_sink: &'a PulseAudioSink,
    sinks: &'a [PulseAudioSink],
) -> &'a PulseAudioSink {
    let next_sink = sinks.iter().find(|sink| sink.index > current_sink.index);

    match next_sink {
        Some(sink) => sink,
        None => &sinks[0],
    }
}

fn set_default_sink(next_sink: &PulseAudioSink) {
    let out = Command::new("pactl")
        .arg("set-default-sink")
        .arg(&next_sink.name)
        .output()
        .expect("failed to execute pactl set-default-sink command");
    let out = String::from_utf8(out.stdout).expect("no valid utf8 from pactl set-default-sink");
    println!("{}", out);
}

fn filter_sinks_without_unavailable_port(sinks: &[PulseAudioSink]) -> Vec<PulseAudioSink> {
    sinks
        .iter()
        .filter(|sink| {
            sink.ports.is_empty()
                || !sink
                    .ports
                    .iter()
                    .all(|port| port.availability == "not available")
        })
        .cloned()
        .collect()
}

fn main() {
    let sinks = get_pactl_sinks();
    let current_sink = get_current_active_sink(&sinks);
    let available_sinks = filter_sinks_without_unavailable_port(&sinks);
    let next_sink = get_sink_with_next_index(current_sink, &available_sinks);
    set_default_sink(next_sink);
}
