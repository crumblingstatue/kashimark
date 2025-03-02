use {
    crate::{ParseError, ParseErrorKind},
    fw_conv::StrExt,
};

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

pub fn parse(src: &str) -> Result<Vec<Block>, ParseError> {
    let mut blocks = Vec::new();
    let mut lines = Vec::new();
    for (i, line) in src.lines().enumerate() {
        macro_rules! ret_err {
            ($kind:expr) => {
                return Err(ParseError {
                    line: i + 1,
                    kind: $kind,
                })
            };
        }
        let line = line.trim();
        if line.is_empty() && !lines.is_empty() {
            blocks.push(Block {
                lines: std::mem::take(&mut lines),
            });
        } else {
            let Some((id, content)) = line.split_once('　') else {
                ret_err!(ParseErrorKind::IdContentSplit);
            };
            let id_sw = id.to_sw();
            let [kind_ch, track_ch] = id_sw.as_bytes() else {
                ret_err!(ParseErrorKind::InvalidTrackIdFormat);
            };
            let Some(kind) = LineKind::try_from_ascii(*kind_ch) else {
                ret_err!(ParseErrorKind::InvalidLineKind { raw: *kind_ch });
            };
            let Some(track_id): Option<u8> = track_ch.checked_sub(b'0') else {
                ret_err!(ParseErrorKind::InvalidTrackId { raw: *track_ch });
            };
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
    Ok(blocks)
}
