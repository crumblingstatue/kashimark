use {
    clap::Parser,
    std::{io::Write, path::PathBuf},
};

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
    /// Convert to fullwidth text (for positioning)
    ToFw,
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let s = std::fs::read_to_string(args.path).unwrap();
    match args.cmd {
        Cmd::Dump => dump(s)?,
        Cmd::ToFw => to_fw(s),
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

fn to_fw(s: String) {
    let mut f = std::fs::File::create("/tmp/kashimark-fw.txt").unwrap();
    for line in s.lines() {
        if !line.is_empty() {
            f.write_all(&line.as_bytes()[0..1]).unwrap();
            f.write_all(fw_conv::sw_to_fw(&line[1..]).as_bytes())
                .unwrap();
        }
        f.write_all(b"\n").unwrap();
    }
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Fatal error: {e}");
    }
}
