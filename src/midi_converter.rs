use midly::{Smf, Timing};

pub struct MidiConverter {
    pub song_path: String,
}

impl MidiConverter {
    pub async fn load_midi(self) {
        println!("Printing Midi Results");

        let bpm = 146;

        let smf = Smf::parse(include_bytes!("../assets/goldn.mid")).unwrap();
        let blah =  match smf.header.timing {
            Timing::Metrical(i) => {i.as_int() as f32}
            Timing::Timecode(_, _) => { panic!() }
        };
        let mut tick_counter = 0;
        let track = smf.tracks[0].clone();
        for note in &track {
            tick_counter += note.delta.as_int();
            if note.delta.as_int() != 0 {
                let timestamp = (((tick_counter as f32 * 60000.0) / bpm as f32) * blah) / 500000.0;
                println!("{}, {:?}", timestamp, note.kind);
            }
        }
    }
}