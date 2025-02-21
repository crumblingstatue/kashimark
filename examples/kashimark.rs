use {clap::Parser, std::path::PathBuf};

#[derive(clap::Parser)]
struct Args {
    path: PathBuf,
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Subcommand, Clone)]
enum Cmd {
    /// Dump parsed contents of kashimark file
    Dump,
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let s = std::fs::read_to_string(args.path).unwrap();
    match args.cmd {
        Cmd::Dump => dump(s)?,
    }
    Ok(())
}

fn dump(s: String) -> Result<(), kashimark::ParseError> {
    let lines = kashimark::parse(&s)?;
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
    Ok(())
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Fatal error: {e}");
    }
}
