use anyhow::Result;
use std::time::Duration;
use rodio::Source; // for take_duration, amplify

// Keep rodio behind a small wrapper so UI can stay clean
pub struct SoundPlayer {
    // Keep the stream alive as long as the player lives
    _stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
}

impl SoundPlayer {
    pub fn new() -> Result<Self> {
        let (stream, handle) = rodio::OutputStream::try_default()?;
        Ok(Self { _stream: stream, handle })
    }

    pub fn play_type(&self) -> Result<()> { self.beep(1200, 20, 0.12) }
    pub fn play_miss(&self) -> Result<()> { self.beep(300, 90, 0.18) }

    fn beep(&self, hz: u32, dur_ms: u64, amp: f32) -> Result<()> {
        let sink = rodio::Sink::try_new(&self.handle)?;
        let src = rodio::source::SineWave::new(hz as f32)
            .take_duration(Duration::from_millis(dur_ms))
            .amplify(amp);
        sink.append(src);
        // Let it play asynchronously and drop
        sink.detach();
        Ok(())
    }
}
