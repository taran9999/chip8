use sdl2::audio::AudioCallback;

pub struct SquareWave {
    pub phase: f32,
    pub phase_increment: f32,
    pub volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            // Square wave: phase is half max amplitude, half min amplitude
            *x = if self.phase <= 0.5 { self.volume } else { -self.volume };

            self.phase = (self.phase + self.phase_increment) % 1.0;
        }
    }
}