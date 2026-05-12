use std::sync::mpsc;
use std::thread;

use rodio::source::SineWave;
use rodio::{OutputStream, Sink, Source};
use std::time::Duration;

pub enum AudioCommand {
    PlayPreset(String),
    Shutdown,
}

pub struct AudioEngine {
    cmd_tx: mpsc::Sender<AudioCommand>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<AudioCommand>();

        thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(_) => return,
            };

            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => s,
                Err(_) => return,
            };

            loop {
                match cmd_rx.recv() {
                    Ok(AudioCommand::PlayPreset(name)) => {
                        if let Some(source) = generate_sound(&name) {
                            sink.append(source);
                        }
                    }
                    Ok(AudioCommand::Shutdown) | Err(_) => break,
                }
            }
        });

        AudioEngine { cmd_tx }
    }

    pub fn play(&self, preset: &str) {
        let _ = self.cmd_tx.send(AudioCommand::PlayPreset(preset.to_string()));
    }
}

fn generate_sound(name: &str) -> Option<Box<dyn Source<Item = f32> + Send>> {
    match name {
        "beep" => Some(Box::new(
            SineWave::new(880.0)
                .take_duration(Duration::from_millis(200))
                .amplify(0.3),
        )),
        "alert" => Some(Box::new(
            SineWave::new(660.0)
                .take_duration(Duration::from_millis(400))
                .amplify(0.4),
        )),
        "alarm" => Some(Box::new(
            SineWave::new(440.0)
                .take_duration(Duration::from_millis(600))
                .amplify(0.5),
        )),
        "notification" => Some(Box::new(
            SineWave::new(1047.0)
                .take_duration(Duration::from_millis(120))
                .amplify(0.2),
        )),
        "critical" => Some(Box::new(
            SineWave::new(880.0)
                .take_duration(Duration::from_millis(800))
                .amplify(0.6),
        )),
        "warnsiren" => Some(Box::new(
            SineWave::new(750.0)
                .take_duration(Duration::from_millis(500))
                .amplify(0.4),
        )),
        "chime" => Some(Box::new(
            SineWave::new(659.0)
                .take_duration(Duration::from_millis(300))
                .amplify(0.3),
        )),
        _ => None,
    }
}
