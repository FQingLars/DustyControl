use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use rodio::source::SineWave;
use rodio::{OutputStream, Sink, Source};

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

        thread::Builder::new()
            .name("audio_engine".into())
            .spawn(move || {
                const MAX_QUEUE_SIZE: usize = 8; // Защита от flooding
                let mut sound_queue: Vec<Box<dyn Source<Item = f32> + Send>> = Vec::new();
                let mut stream_and_handle: Option<(OutputStream, rodio::OutputStreamHandle)> = None;
                let mut sink: Option<Sink> = None;

                let mut init_audio = || -> Result<(OutputStream, rodio::OutputStreamHandle, Sink), String> {
                    let (stream, handle) = OutputStream::try_default()
                        .map_err(|e| format!("OutputStream: {}", e))?;
                    let sink = Sink::try_new(&handle)
                        .map_err(|e| format!("Sink: {}", e))?;
                    Ok((stream, handle, sink))
                };

                // Инициализация при старте
                match init_audio() {
                    Ok((s, h, sk)) => {
                        stream_and_handle = Some((s, h));
                        sink = Some(sk);
                    }
                    Err(e) => eprintln!("[Audio] Initial setup failed: {}", e),
                }

                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        AudioCommand::Shutdown => break,
                        AudioCommand::PlayPreset(name) => {
                            if let Some(source) = generate_sound(&name) {
                                // Ограничиваем очередь, чтобы не перегрузить буфер
                                if sound_queue.len() < MAX_QUEUE_SIZE {
                                    sound_queue.push(source);
                                } else {
                                    // Сбрасываем очередь, если она переполнилась
                                    sound_queue.clear();
                                    sound_queue.push(source);
                                }

                                // Пробуем добавить в Sink, если он жив
                                let mut stream_broken = false;
                                if let Some(ref mut s) = sink {
                                    for src in sound_queue.drain(..) {
                                        s.append(src);
                                    }
                                } else {
                                    stream_broken = true;
                                }

                                // Если поток сломался (или изначально не инициализировался), пересоздаём
                                if stream_broken || sound_queue.is_empty() {
                                    if let Ok((stream, handle, new_sink)) = init_audio() {
                                        stream_and_handle = Some((stream, handle));
                                        sink = Some(new_sink);
                                        if let Some(ref mut s) = sink {
                                            s.append(generate_sound(&name).unwrap());
                                        }
                                    } else {
                                        eprintln!("[Audio] Failed to recover stream, retrying in 1s...");
                                        thread::sleep(Duration::from_secs(1));
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .expect("Failed to spawn audio thread");

        AudioEngine { cmd_tx }
    }

    pub fn play(&self, preset: &str) {
        let _ = self.cmd_tx.send(AudioCommand::PlayPreset(preset.to_string()));
    }
}

fn generate_sound(name: &str) -> Option<Box<dyn Source<Item = f32> + Send>> {
    match name {
        "beep" => Some(Box::new(SineWave::new(880.0).take_duration(Duration::from_millis(200)).amplify(0.3))),
        "alert" => Some(Box::new(SineWave::new(660.0).take_duration(Duration::from_millis(400)).amplify(0.4))),
        "alarm" => Some(Box::new(SineWave::new(440.0).take_duration(Duration::from_millis(600)).amplify(0.5))),
        "notification" => Some(Box::new(SineWave::new(1047.0).take_duration(Duration::from_millis(120)).amplify(0.2))),
        "critical" => Some(Box::new(SineWave::new(880.0).take_duration(Duration::from_millis(800)).amplify(0.6))),
        "warnsiren" => Some(Box::new(SineWave::new(750.0).take_duration(Duration::from_millis(500)).amplify(0.4))),
        "chime" => Some(Box::new(SineWave::new(659.0).take_duration(Duration::from_millis(300)).amplify(0.3))),
        _ => None,
    }
}