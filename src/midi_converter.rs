use std::fs::File;
use std::io::Write;
use midly::{MidiMessage, Smf, Timing, TrackEventKind};
use crate::note_gameplay_scene::song::Song;

pub struct MidiConverter {
    pub song_path: String,
}

impl MidiConverter {
    pub async fn load_midi(self) {
        let min_spacing = 1.0 / 8.0; // A 16th note

        println!("Printing Midi Results");

        let bpm = 146.0;

        let smf = Smf::parse(include_bytes!("../assets/goldn.mid")).unwrap();
        let ppq =  match smf.header.timing {
            Timing::Metrical(i) => {i.as_int() as f32}
            Timing::Timecode(_, _) => { panic!() }
        };
        let millis_per_tick = 60000.0 / (bpm * ppq);
        let mut prints = 0;

        let mut last_beat = 2.0;

        let save_path = "assets/songs/test.json";

        let mut song = Song {
            song_filepath: "assets/songs/music_files/Goldn.wav".to_string(),
            song_length: 159.0,
            bpm,
            credits: "Goldn".to_string(),
            high_score: 0,
            notes: vec![],
            attacks: vec![],
        };

        for track in &smf.tracks {
            let mut tick_counter = 0;
            for note in track {
                tick_counter += note.delta.as_int();
                match note.kind {
                    TrackEventKind::Midi {channel: _chan, message} => {
                        match message {
                            MidiMessage::NoteOn { .. } => {
                                let physical_second = (millis_per_tick * tick_counter as f32) / 1000.0;
                                let beat = physical_second * (bpm / 60.0);
                                if beat - last_beat > min_spacing {
                                    last_beat = beat;
                                    prints += 1;
                                    song.notes.push((beat, 1.0, 0.0));
                                }
                            }
                            _ => {}
                        }
                    }
                    TrackEventKind::SysEx(_) => {}
                    TrackEventKind::Escape(_) => {}
                    TrackEventKind::Meta(_) => {}
                }
            }
        }

        let mut file = File::create(save_path.clone()).unwrap();
        let cloned_song = song.clone();
        file.write_all(serde_json::to_string_pretty(&cloned_song).unwrap().as_ref()).unwrap();

        println!("{prints}");
    }
}