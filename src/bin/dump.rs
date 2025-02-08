fn main() {
    let path = std::env::args_os().nth(1).expect("Need path");
    let s = std::fs::read_to_string(path).unwrap();
    let lines = kashimark::parse(&s);
    for line in lines {
        for track in line.tracks {
            match track.data {
                kashimark::TrackData::Timing(timing_track) => {
                    for seg in timing_track.segments {
                        match seg {
                            kashimark::TimedSegOrFill::Seg(timed_segment) => {
                                eprint!(" |{}", timed_segment.text);
                                if !timed_segment.furigana.is_empty() {
                                    eprint!("(");
                                    for furi in timed_segment.furigana {
                                        eprint!("{furi}");
                                    }
                                    eprint!(")");
                                }
                                eprint!("| ");
                            }
                            kashimark::TimedSegOrFill::Fill => eprint!(" |--| "),
                        }
                    }
                }
                kashimark::TrackData::Raw(s) => eprintln!("{s}"),
            }
            eprintln!();
        }
        eprintln!("===\n");
    }
}
