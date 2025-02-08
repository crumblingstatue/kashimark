#[derive(Debug)]
pub struct Block<'s> {
    pub lines: Vec<Line<'s>>,
}

#[derive(Debug)]
pub struct Line<'s> {
    pub kind: LineKind,
    pub track_id: u8,
    pub content: &'s str,
}

#[derive(Debug)]
pub enum LineKind {
    Timing,
    Furigana,
    Raw,
}

impl LineKind {
    fn try_from_ascii(ascii: u8) -> Option<Self> {
        match ascii {
            b't' => Some(Self::Timing),
            b'f' => Some(Self::Furigana),
            b'r' => Some(Self::Raw),
            _ => None,
        }
    }
}

pub fn parse(src: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut lines = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() && !lines.is_empty() {
            blocks.push(Block {
                lines: std::mem::take(&mut lines),
            });
        } else {
            let (id, content) = line.split_once('　').unwrap();
            let [kind_ch, track_ch] = id.as_bytes() else {
                panic!("Invalid track id format @ {line}");
            };
            let kind = LineKind::try_from_ascii(*kind_ch).unwrap();
            let track_id: u8 = track_ch.checked_sub(b'0').unwrap();
            lines.push(Line {
                kind,
                track_id,
                content,
            });
        }
    }
    if !lines.is_empty() {
        blocks.push(Block {
            lines: std::mem::take(&mut lines),
        });
    }
    blocks
}
