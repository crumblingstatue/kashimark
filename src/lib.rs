mod stage1;
mod stage2;

pub use stage2::{Line, TimedSegOrFill, TimedSegment, TimingTrack, Track};

pub fn parse(src: &str) -> Vec<Line> {
    let blocks = stage1::parse(src);
    stage2::parse(blocks)
}
