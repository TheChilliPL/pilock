use proc_macro2::Span;
use quote::quote;
use syn::{parse, parse_macro_input, Ident};

/// A procedural macro that generates a pub enum for musical notes.
///
/// It generates notes from C0 to B8, including sharps, and provides methods to get the frequency in Hz and the string representation of each note.
#[proc_macro]
pub fn music_proc_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !input.is_empty() {
        return syn::Error::new(Span::call_site(), "This macro does not take any input")
            .to_compile_error()
            .into();
    }

    const NOTE_IDENTS: [&str; 12] = [
        "C", "CSharp", "D", "DSharp", "E", "F", "FSharp", "G", "GSharp", "A", "ASharp", "B",
    ];

    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    fn note_to_freq(octave: usize, note: usize) -> f64 {
        const A4_NOTE_NUMBER: i32 = 9;
        const A4_OCTAVE: i32 = 4;
        const A4_FREQUENCY: f64 = 440.0;

        let n = (octave as i32 - A4_OCTAVE) * 12 + (note as i32 - A4_NOTE_NUMBER);
        A4_FREQUENCY * 2_f64.powf(n as f64 / 12.0)
    }

    let mut all_note_idents = Vec::with_capacity(NOTE_IDENTS.len() * 9);
    let mut all_note_names = Vec::with_capacity(NOTE_NAMES.len() * 9);
    let mut all_note_freqs = Vec::with_capacity(NOTE_IDENTS.len() * 9);

    for octave in 0..9 {
        for (i, &note_ident) in NOTE_IDENTS.iter().enumerate() {
            let ident_str = format!("{}{}", note_ident, octave);
            let note_ident = Ident::new(&ident_str, Span::call_site());
            let note_name = format!("{}{}", NOTE_NAMES[i], octave);
            let freq = note_to_freq(octave, i);
            all_note_idents.push(quote! { #note_ident });
            all_note_names.push(quote! { #note_name });
            all_note_freqs.push(quote! { #freq });
        }
    }

    quote!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum MusicalNote {
            #(#all_note_idents),*
        }

        impl MusicalNote {
            pub fn as_freq_hz(&self) -> f64 {
                match self {
                    #(MusicalNote::#all_note_idents => #all_note_freqs),*
                }
            }
        }

        impl ToString for MusicalNote {
            fn to_string(&self) -> String {
                match self {
                    #(MusicalNote::#all_note_idents => #all_note_names.to_string()),*
                }
            }
        }
    ).into()
}

/// A procedural macro that converts a string literal representing a musical note into the corresponding `MusicalNote` enum variant.
///
/// It *does not* handle importing the `MusicalNote` enum, but assumes it has been defined using the `music_proc_macro!()` macro and
/// is available in the current scope.
#[proc_macro]
pub fn note(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // It should change note!(C#4) to MusicalNote::CSharp4

    // First we need to parse the input token string literal
    let input_str = parse_macro_input!(input as syn::LitStr);
    let ident_str = input_str.value().replace("#", "Sharp");
    let output_ident = Ident::new(&ident_str, Span::call_site());

    quote!(
        MusicalNote::#output_ident
    ).into()
}
