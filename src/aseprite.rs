use std::fmt;

use serde::Deserialize;
use serde_json::Value;

use crate::{
    AnimationClip, AnimationLibrary, AnimationState, AnimationTarget, ClipFrame, FrameTiming,
    PlaybackDirection,
};

#[derive(Debug)]
pub enum AsepriteImportError {
    Json(serde_json::Error),
    MissingFrames,
    InvalidTagRange {
        name: String,
        from: usize,
        to: usize,
        frame_count: usize,
    },
}

impl fmt::Display for AsepriteImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "failed to parse Aseprite JSON: {error}"),
            Self::MissingFrames => write!(f, "Aseprite JSON did not contain any frames"),
            Self::InvalidTagRange {
                name,
                from,
                to,
                frame_count,
            } => write!(
                f,
                "Aseprite tag '{name}' references frame range [{from}, {to}] but only {frame_count} frames were imported"
            ),
        }
    }
}

impl std::error::Error for AsepriteImportError {}

impl From<serde_json::Error> for AsepriteImportError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl AnimationLibrary {
    pub fn from_aseprite_json(
        name: impl Into<String>,
        json: &str,
    ) -> Result<Self, AsepriteImportError> {
        let AsepriteDocument { frames, meta } = serde_json::from_str(json)?;
        let tags = meta.frame_tags.unwrap_or_default();
        let frames = AsepriteDocument::ordered_frames(frames)?;
        if frames.is_empty() {
            return Err(AsepriteImportError::MissingFrames);
        }

        let mut library = AnimationLibrary::new(name);

        if tags.is_empty() {
            let clip_id = "default";
            library = library
                .add_clip(build_clip(clip_id, &frames))
                .with_default_target(AnimationTarget::clip(clip_id));
            return Ok(library);
        }

        for tag in &tags {
            if tag.to < tag.from || tag.to >= frames.len() {
                return Err(AsepriteImportError::InvalidTagRange {
                    name: tag.name.clone(),
                    from: tag.from,
                    to: tag.to,
                    frame_count: frames.len(),
                });
            }

            let mut clip = build_clip(&tag.name, &frames[tag.from..=tag.to]);
            clip.direction = tag.direction.playback_direction();
            library = library
                .add_clip(clip)
                .add_state(AnimationState::new(tag.name.as_str(), tag.name.as_str()));
        }

        if let Some(first_tag) = tags.first() {
            library = library.with_default_target(AnimationTarget::state(first_tag.name.clone()));
        }

        Ok(library)
    }
}

fn build_clip(id: &str, frames: &[AsepriteFrame]) -> AnimationClip {
    AnimationClip::from_frames(
        id,
        frames.iter().map(|frame| {
            ClipFrame::new(frame.atlas_index)
                .with_duration_seconds(frame.duration_ms as f32 / 1000.0)
        }),
    )
    .with_timing(FrameTiming::FramesPerSecond(12.0))
}

#[derive(Deserialize)]
struct AsepriteDocument {
    frames: Value,
    #[serde(default)]
    meta: AsepriteMeta,
}

impl AsepriteDocument {
    fn ordered_frames(frames: Value) -> Result<Vec<AsepriteFrame>, AsepriteImportError> {
        match frames {
            Value::Array(entries) => entries
                .into_iter()
                .enumerate()
                .map(|(atlas_index, entry)| {
                    let frame: RawAsepriteFrame = serde_json::from_value(entry)?;
                    Ok(AsepriteFrame {
                        atlas_index,
                        duration_ms: frame.duration_ms,
                    })
                })
                .collect(),
            Value::Object(entries) => {
                let mut frames = entries
                    .into_iter()
                    .map(|(filename, value)| {
                        let frame: RawAsepriteFrame = serde_json::from_value(value)?;
                        Ok((filename, frame))
                    })
                    .collect::<Result<Vec<_>, serde_json::Error>>()?;
                frames.sort_by_key(|left| frame_sort_key(&left.0));
                Ok(frames
                    .into_iter()
                    .enumerate()
                    .map(|(atlas_index, (_, frame))| AsepriteFrame {
                        atlas_index,
                        duration_ms: frame.duration_ms,
                    })
                    .collect())
            }
            _ => Ok(Vec::new()),
        }
    }
}

#[derive(Deserialize, Default)]
struct AsepriteMeta {
    #[serde(default, rename = "frameTags")]
    frame_tags: Option<Vec<AsepriteTag>>,
}

#[derive(Clone)]
struct AsepriteFrame {
    atlas_index: usize,
    duration_ms: u64,
}

#[derive(Clone, Deserialize)]
struct RawAsepriteFrame {
    #[serde(rename = "duration")]
    duration_ms: u64,
}

#[derive(Clone, Deserialize)]
struct AsepriteTag {
    name: String,
    from: usize,
    to: usize,
    #[serde(default)]
    direction: AsepriteTagDirection,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AsepriteTagDirection {
    #[default]
    Forward,
    Reverse,
    Pingpong,
}

impl AsepriteTagDirection {
    fn playback_direction(self) -> PlaybackDirection {
        match self {
            Self::Forward => PlaybackDirection::Forward,
            Self::Reverse => PlaybackDirection::Reverse,
            Self::Pingpong => PlaybackDirection::PingPong,
        }
    }
}

fn frame_sort_key(filename: &str) -> (String, Option<u32>, String) {
    let bytes = filename.as_bytes();
    for end in (0..bytes.len()).rev() {
        if !bytes[end].is_ascii_digit() {
            continue;
        }

        let mut start = end;
        while start > 0 && bytes[start - 1].is_ascii_digit() {
            start -= 1;
        }

        return (
            filename[..start].to_string(),
            filename[start..=end].parse::<u32>().ok(),
            filename.to_string(),
        );
    }

    ("".to_string(), None, filename.to_string())
}

#[cfg(test)]
#[path = "aseprite_tests.rs"]
mod tests;
