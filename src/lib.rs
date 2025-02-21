mod stage1;
mod stage2;

pub use stage2::{Line, TimedSegOrFill, TimedSegment, TimingTrack, TrackData};

#[derive(Debug, thiserror::Error)]
#[error("Parse error on line {line}: {kind}")]
pub struct ParseError {
    pub line: usize,
    pub kind: ParseErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("Couldn't split id and content (no wide whitespace?)")]
    IdContentSplit,
    #[error("Invalid track id format")]
    InvalidTrackIdFormat,
    #[error("Invalid line kind: {raw}")]
    InvalidLineKind { raw: u8 },
    #[error("Invalid track id: {raw}", raw = *raw as char)]
    InvalidTrackId { raw: u8 },
}

pub fn parse(src: &str) -> Result<Vec<Line>, ParseError> {
    let blocks = stage1::parse(src)?;
    Ok(stage2::parse(blocks))
}
