//! Home for the [MusicalNote] type and related functionality.

use time::Duration;
use pilock_music_proc_macro::{music_proc_macro, note};

music_proc_macro!();

/// Type alias representing a melody.
///
/// It's a [Vec] of tuples, where each tuple contains:
/// - An optional [MusicalNote] (the note being played, or `None` for a pause).
/// - A [time::Duration] representing how long the note or pause lasts.
///
/// It can be easily created using the [crate::melody] macro.
pub type Melody = Vec<(Option<MusicalNote>, Duration)>;

/// Makes a melody using a DSL-style macro.
///
/// # Example
///
/// ```rs
/// # use pilock_music::melody;
/// # fn main() {
/// let my_melody = melody![
///     "C4" for 500 ms,
///     pause for 250 ms,
///     "C#4" for 300 ms,
/// ];
/// # }
/// ```
///
/// The example above creates a melody with three segments:
/// - the C4 note playing for 500 ms
/// - a pause for 250 ms
/// - the C#4 note playing for 300 ms
#[macro_export]
macro_rules! melody {
    ($($e:tt for $dur:literal ms),* $(,)?) => {
        (vec![$(melody!(@note $e for $dur ms)),*] as Melody)
    };
    (@note pause for $dur:literal ms) => {
        (None, ::time::Duration::milliseconds($dur))
    };
    (@note $note:literal for $dur:literal ms) => {
        (Some(note!($note)), ::time::Duration::milliseconds($dur))
    };
}

/// Extension trait for the [Melody] type, providing additional functionality.
pub trait MelodyExt {
    /// Gets the duration of the melody.
    fn duration(&self) -> Duration;

    /// Gets the note at a specific duration in the melody, or `None` for a pause.
    ///
    /// Returns `None` if the duration is out of bounds.
    fn get_note_at(&self, duration: Duration) -> Option<MusicalNote>;
}

impl MelodyExt for Melody {
    fn duration(&self) -> Duration {
        self.iter().fold(Duration::ZERO, |acc, &(_note, dur)| acc + dur)
    }

    fn get_note_at(&self, duration: Duration) -> Option<MusicalNote> {
        if duration < Duration::ZERO || duration > self.duration() {
            return None; // Out of bounds
        }

        let mut elapsed = Duration::ZERO;
        for &(note, dur) in self {
            if elapsed + dur > duration {
                return note; // Return the note if the duration falls within this segment
            }
            elapsed += dur;
        }
        None // If no note is found for the given duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notes() {
        let note = note!("C#4");

        assert_eq!(note, MusicalNote::CSharp4);
        assert!(note.as_freq_hz() - 277.18 < 0.01); // C#4 frequency is approximately 277.18 Hz
    }

    #[test]
    fn test_melody_macro() {
        let melody = melody![
            "C4" for 500 ms,
            "D4" for 500 ms,
            pause for 250 ms,
            "E4" for 500 ms,
        ];

        assert_eq!(melody.len(), 4);
        assert_eq!(melody[0], (Some(note!("C4")), Duration::milliseconds(500)));
        assert_eq!(melody[1], (Some(note!("D4")), Duration::milliseconds(500)));
        assert_eq!(melody[2], (None, Duration::milliseconds(250)));
        assert_eq!(melody[3], (Some(note!("E4")), Duration::milliseconds(500)));

        assert_eq!(melody.duration(), Duration::milliseconds(1750));

        assert_eq!(melody.get_note_at(Duration::milliseconds(-5)), None); // Out of bounds
        assert_eq!(melody.get_note_at(Duration::milliseconds(0)), Some(note!("C4")));
        assert_eq!(melody.get_note_at(Duration::milliseconds(250)), Some(note!("C4")));
        assert_eq!(melody.get_note_at(Duration::milliseconds(500)), Some(note!("D4")));
        assert_eq!(melody.get_note_at(Duration::milliseconds(750)), Some(note!("D4")));
        assert_eq!(melody.get_note_at(Duration::milliseconds(1000)), None); // Pause
        assert_eq!(melody.get_note_at(Duration::milliseconds(1250)), Some(note!("E4")));
        assert_eq!(melody.get_note_at(Duration::milliseconds(1750)), None); // Out of bounds
    }
}
