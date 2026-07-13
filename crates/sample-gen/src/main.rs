use std::f32::consts::PI;
use std::path::Path;

const SAMPLE_RATE: u32 = 44100;
const DURATION_SECS: f32 = 2.5;
const AMPLITUDE: f32 = 0.7;

/// Formant frequencies (Hz) and bandwidths for the "ah" vowel.
struct Formant {
    freq: f32,
    bw: f32,
}

const FORMANT_AH: [Formant; 3] = [
    Formant { freq: 730.0, bw: 80.0 },
    Formant { freq: 1090.0, bw: 90.0 },
    Formant { freq: 2440.0, bw: 120.0 },
];

fn main() {
    let steps: &[(i32, i32, &str, &str)] = &[
        // Male: C2–C5  (MIDI 36–72, 37 notes)
        (36, 72, "male", "samples/male"),
        // Female: C3–C5 (MIDI 48–72, 25 notes)
        (48, 72, "female", "samples/female"),
    ];

    let mut total = 0;
    for &(start_midi, end_midi, voice, dir) in steps {
        for midi in start_midi..=end_midi {
            let hz = midi_to_hz(midi);
            let note = midi_to_note_name(midi);
            let path = format!("{dir}/{note}.wav");
            match generate_wav(&path, hz, voice) {
                Ok(()) => println!("OK  {voice:>6}  {note:>3}  {hz:7.2} Hz  {path}"),
                Err(e) => eprintln!("ERR {voice:>6}  {note:>3}  {hz:7.2} Hz  {e}"),
            }
            total += 1;
        }
    }
    println!("\nGenerated {total} sample files");
}

fn midi_to_hz(midi: i32) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}

fn midi_to_note_name(midi: i32) -> String {
    let notes = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = midi / 12 - 1;
    let note = notes[(midi % 12) as usize];
    format!("{note}{octave}")
}

fn generate_wav(path: &str, frequency_hz: f32, voice: &str) -> Result<(), String> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }

    let num_samples = (SAMPLE_RATE as f32 * DURATION_SECS) as usize;
    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);

    // Formant scaling: female ~17% higher formants
    let formant_scale = if voice == "female" { 1.17 } else { 1.0 };
    let formants: Vec<Formant> = FORMANT_AH
        .iter()
        .map(|f| Formant {
            freq: f.freq * formant_scale,
            bw: f.bw * formant_scale,
        })
        .collect();

    // Pre-compute filter states per formant (biquad)
    let mut filters: Vec<Biquad> = formants
        .iter()
        .map(|f| Biquad::bandpass(f.freq, f.bw, SAMPLE_RATE as f32))
        .collect();

    let attack_len = (SAMPLE_RATE as f32 * 0.05) as usize;
    let release_start = num_samples.saturating_sub((SAMPLE_RATE as f32 * 0.15) as usize);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Rich source: sawtooth + triangle (adds warmth)
        let saw = 2.0 * (t * frequency_hz - (t * frequency_hz + 0.5).floor()) - 0.3;
        let tri = 2.0 * (2.0 * (t * frequency_hz + 0.25) % 1.0 - 0.5).abs() - 0.5;
        let mut source = saw * 0.6 + tri * 0.4;

        // Add subtle vibrato for realism
        let vibrato = 1.0 + 0.005 * (2.0 * PI * 5.5 * t).sin();
        source *= vibrato;

        // Envelope
        let envelope = if i < attack_len {
            i as f32 / attack_len as f32
        } else if i >= release_start {
            1.0 - (i - release_start) as f32 / (num_samples - release_start) as f32
        } else {
            1.0
        };
        source *= envelope;

        // Apply formant filters
        let mut sample = source;
        for filter in &mut filters {
            sample = filter.process(sample);
        }

        // Normalize and clamp
        let sample = (sample * AMPLITUDE).clamp(-1.0, 1.0);
        samples.push((sample * i16::MAX as f32) as i16);
    }

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec).map_err(|e| format!("wav create: {e}"))?;
    for s in &samples {
        writer.write_sample(*s).map_err(|e| format!("wav write: {e}"))?;
    }
    writer.finalize().map_err(|e| format!("wav finalize: {e}"))?;

    Ok(())
}

/// Simple biquad bandpass filter (direct form I).
struct Biquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl Biquad {
    fn bandpass(freq: f32, bw: f32, sample_rate: f32) -> Self {
        let w0 = 2.0 * PI * freq / sample_rate;
        let alpha = w0.sin() * (bw / (2.0 * freq)).ln() * 2.0; // Q-based alpha
        let alpha = alpha.abs();
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;
        let b0 = alpha / a0;
        let b1 = 0.0;
        let b2 = -alpha / a0;
        let a1 = (-2.0 * cos_w0) / a0;
        let a2 = (1.0 - alpha) / a0;

        Biquad {
            b0, b1, b2, a1, a2,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}
