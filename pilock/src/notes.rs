use crabtime::output;
use time::Duration;

#[crabtime::function]
fn create_musical_note() {
    fn note_freq(octave: usize, note_number: usize) -> f64 {
        const BASE_FREQ: f64 = 440.0;
        const BASE_OCTAVE: usize = 4;
        const BASE_NOTE: usize = 9; // A4 is the 9th note in the octave (C4 is 0)

        let note_index = (octave - BASE_OCTAVE) * 12 + (note_number - BASE_NOTE);
        BASE_FREQ * 2f64.powf(note_index as f64 / 12.0)
    }

    const NOTES_IN_OCTAVE: [&'static str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
    ];

    const OCTAVE_COUNT: usize = 9;

    let mut notes = Vec::with_capacity(NOTES_IN_OCTAVE.len() * OCTAVE_COUNT);
    for octave in 0..OCTAVE_COUNT {
        for (i, note) in NOTES_IN_OCTAVE.iter().enumerate() {
            let note_name = format!("{}{}", note, octave);
            let note_ident = note_name.replace("#", "Sharp");
            let note_freq = note_freq(octave, i);

            let note_definition = crabtime::quote!(
                {{note_ident}},
            );

            let note_to_str = crabtim

            notes.push((note_name, note_ident, note_freq))
        }
    }
}
