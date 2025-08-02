use rodio::{source::SineWave, OutputStream, Sink, Source as _};

const SINEWAVE_FREQUENCY: f32 = 440.0; // A4


pub struct Audio {
    audio: Sink
}

impl Audio {
    pub(super) fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let audio = Sink::try_new(&stream_handle).unwrap();
        let source = SineWave::new(SINEWAVE_FREQUENCY).repeat_infinite();
        audio.append(source);
        audio.pause();
        Audio {audio}
    }

    pub(super) fn pause(&self) {
        self.audio.pause();
    }

    pub(super) fn play(&self) {
        self.audio.play();
    }

    pub(super) fn is_playing(&self) -> bool {
        !self.audio.is_paused()
    }
}