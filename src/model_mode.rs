use std::str::FromStr;
/// The modes for the interaction with Open AI
#[derive(Debug, PartialEq)]
pub enum ModelMode {
    Completions,
    Chat,
    Image,
    ImageEdit,
    AudioTranscription,
}
impl std::fmt::Display for ModelMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ModelMode::Completions => "completions",
            ModelMode::Chat => "chat",
            ModelMode::Image => "image",
            ModelMode::ImageEdit => "image_edit",
            ModelMode::AudioTranscription => "audio_transcription",
        };
        write!(f, "{str}")
    }
}

#[derive(Debug)]
pub struct ModelModeParseErr;

impl FromStr for ModelMode {
    type Err = ModelModeParseErr;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "completions" => Ok(ModelMode::Completions),
            "chat" => Ok(ModelMode::Chat),
            "image" => Ok(ModelMode::Image),
            "image_edit" => Ok(ModelMode::ImageEdit),
            "audio_transcription" => Ok(ModelMode::AudioTranscription),
            _ => Err(ModelModeParseErr),
        }
    }
}
