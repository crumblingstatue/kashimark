use crate::stage1::{self, Block};

#[derive(Debug)]
pub struct Track {
    pub id: u8,
    pub data: TrackData,
}

#[derive(Debug)]
pub enum TrackData {
    Timing(TimingTrack),
    Raw(String),
}

#[derive(Debug)]
pub struct TimingTrack {
    pub segments: Vec<TimedSegOrFill>,
}

#[derive(Debug)]
pub enum TimedSegOrFill {
    Seg(TimedSegment),
    Fill,
}

#[derive(Debug, Default)]
pub struct TimedSegment {
    pub text: String,
    span: std::ops::Range<usize>,
    /// Furigana sub-segments, if any
    pub furigana: Vec<String>,
}

/// A `Line` is a group of subtitles that gets shown at once
#[derive(Debug)]
pub struct Line {
    pub tracks: Vec<Track>,
    pub segment_count: usize,
}

trait RangeExt {
    fn contains_range(&self, other: &Self) -> bool;
}

impl RangeExt for std::ops::Range<usize> {
    fn contains_range(&self, other: &Self) -> bool {
        other.start >= self.start && other.end <= self.end
    }
}

impl Line {
    fn parse(block: &Block) -> Self {
        let mut tracks: Vec<Track> = Vec::new();
        let mut seg_count = None;
        // Collect the "base" tracks that won't be merged into other tracks
        for line in &block.lines {
            match line.kind {
                stage1::LineKind::Timing => {
                    let mut segments = Vec::new();
                    let mut furi_spans = None;
                    let segs = segment_timed(line.content);
                    for line2 in &block.lines {
                        if line2.track_id == line.track_id {
                            if let stage1::LineKind::Furigana = line2.kind {
                                furi_spans = Some((line2.content, furigana_spans(line2.content)));
                            }
                        }
                    }
                    // How much to add to index to compare to prev track segment end.
                    // Increases each time we add a filler
                    let mut add_offset = 0;
                    for (i, seg) in segs.iter().enumerate() {
                        let mut furigana = Vec::new();
                        if let Some((furi_line, furi_spans)) = &furi_spans {
                            for furi_span in furi_spans {
                                if seg.contains_range(furi_span) {
                                    furigana = furi_line[furi_span.clone()]
                                        .trim()
                                        .split('｜')
                                        .filter_map(|tok| {
                                            let trimmed = tok.trim();
                                            (!trimmed.is_empty()).then(|| String::from(trimmed))
                                        })
                                        .collect();
                                }
                            }
                        }
                        segments.push(TimedSegOrFill::Seg(TimedSegment {
                            text: fw_to_hw(line.content[seg.clone()].trim()),
                            span: seg.clone(),
                            furigana,
                        }));
                        // TODO: "Filling" is done based on previous track right now
                        // Should be based on longest, right now we assume previous is longest, which
                        // is only true in very specific scenarios (one romaji track followed by one kanji track)
                        if let Some(TrackData::Timing(prev)) = tracks.last().map(|tr| &tr.data) {
                            loop {
                                // TODO: We furthermore assume it's not a filler
                                let TimedSegOrFill::Seg(prev_seg) = &prev.segments[i + add_offset]
                                else {
                                    panic!("We assumed it wouldn't be a filler");
                                };
                                if seg.end > prev_seg.span.end {
                                    segments.push(TimedSegOrFill::Fill);
                                    add_offset += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    // TODO: Ugly, but we check if the segments are the same length (they should be)
                    // Since the gaps are filled by `Fill` segments
                    if let Some(TrackData::Timing(prev)) = tracks.last().map(|tr| &tr.data) {
                        assert!(
                            prev.segments.len() == segments.len(),
                            "Segment length mismatch on block {:#?}",
                            block
                        );
                    }
                    seg_count = Some(segments.len());
                    tracks.push(Track {
                        id: line.track_id,
                        data: TrackData::Timing(TimingTrack { segments }),
                    });
                }
                stage1::LineKind::Raw => {
                    tracks.push(Track {
                        id: line.track_id,
                        data: TrackData::Raw(fw_to_hw(line.content)),
                    });
                }
                _ => {}
            }
        }
        Self {
            tracks,
            segment_count: seg_count.unwrap_or(0),
        }
    }
}

pub fn parse(blocks: Vec<Block<'_>>) -> Vec<Line> {
    let mut lines = Vec::new();
    for block in blocks {
        lines.push(Line::parse(&block));
    }
    lines
}

fn segment_timed(src: &str) -> Vec<std::ops::Range<usize>> {
    let mut segs = Vec::new();
    let mut beg = 0;
    for (idx, ch) in src.char_indices() {
        if ch == '｜' {
            segs.push(beg..idx);
            beg = idx + 3;
        }
    }
    segs
}

const FW_SPACE: char = char::from_u32(0x3000).unwrap();

fn fw_to_hw(inp: &str) -> String {
    let mut out = String::new();
    for ch in inp.chars() {
        let out_c = match ch {
            'Ａ'..='Ｚ' => char::from_u32((ch as u32 - 'Ａ' as u32) + 'A' as u32).unwrap(),
            'ａ'..='ｚ' => char::from_u32((ch as u32 - 'ａ' as u32) + 'a' as u32).unwrap(),
            FW_SPACE => ' ',
            // TODO: Technically this doesn't belong here, this is a special character that stands in for whitespace
            '／' => ' ',
            _ => ch,
        };
        out.push(out_c);
    }
    out
}

#[test]
fn test_segment_timed() {
    let src = "ｉ｜ｅ｜ｎａｉ｜ｉ｜ｔａ｜ｍｉ｜ｋａ｜ｎａ｜ｓｈｉ｜ｍｉ｜ｄｅ｜ｋｉ｜ｚｕ｜ｔｓｕｉ｜ｔａ｜ｋｉ｜ｍｉ｜ｙｏ｜";
    let segs = segment_timed(src);
    assert_eq!(segs.len(), 18);
    assert_eq!(&src[segs[0].clone()], "ｉ");
    assert_eq!(&src[segs[1].clone()], "ｅ");
    assert_eq!(&src[segs[2].clone()], "ｎａｉ");
    assert_eq!(&src[segs[17].clone()], "ｙｏ");
    let src = "癒｜え｜ない　｜　痛　　｜み　｜　　悲　　｜し　　｜み　｜で　｜キ　｜ズ　｜つい　　｜た　｜　　君　　｜よ　｜";
    let segs = segment_timed(src);
    assert_eq!(segs.len(), 15);
    assert_eq!(&src[segs[0].clone()], "癒");
    assert_eq!(&src[segs[14].clone()], "よ　");
}

fn furigana_spans(src: &str) -> Vec<std::ops::Range<usize>> {
    enum Status {
        Init,
        InWs,
    }
    let mut spans = Vec::new();
    let mut status = Status::Init;
    let mut beg = 0;
    for (idx, ch) in src.char_indices() {
        match status {
            Status::Init => {
                if ch == '　' {
                    status = Status::InWs;
                    spans.push(beg..idx);
                    beg = idx;
                }
            }
            Status::InWs => {
                if ch != '　' {
                    status = Status::Init;
                    beg = idx;
                }
            }
        }
    }
    if let Status::Init = status {
        let range = beg..src.len();
        if !range.is_empty() {
            spans.push(range);
        }
    }
    spans
}

#[test]
fn test_furigana_spans() {
    let src = "い　　　　　　　い｜た　　　　　　か｜な　　　　　　　　　　　　　　　　　　　　　　　　　　　き｜み";
    let spans = furigana_spans(src);
    assert_eq!(spans.len(), 4);
    assert_eq!(&src[spans[0].clone()], "い");
    assert_eq!(&src[spans[1].clone()], "い｜た");
    assert_eq!(&src[spans[2].clone()], "か｜な");
    assert_eq!(&src[spans[3].clone()], "き｜み");
}
