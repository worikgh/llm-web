use std::str::FromStr;
/// The modes for the interaction with Open AI
#[derive(Debug, Clone, PartialEq)]
pub enum ModelMode {
    Completions,
    Chat,
    Image,
    ImageEdit,
    AudioTranscription,
}
const MODELS_COMPLETIONS: [&str; 6] = [
    "text-babbage-001",
    "text-curie-001",
    "text-davinci-001",
    "text-davinci-002",
    "text-davinci-003",
    "text-davinci-edit-001",
];
const MODELS_CHAT: [&str; 2] = ["gpt-3.5-turbo", "gpt-4"];

const MODELS_AUDIOTRANSCRIPTION: [&str; 1] = ["whisper-1"];

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

impl ModelMode {
    pub fn models_available(&self) -> Vec<&str> {
        match self {
            ModelMode::Completions => MODELS_COMPLETIONS.to_vec(),
            ModelMode::Chat => MODELS_CHAT.to_vec(),
            ModelMode::Image => [].to_vec(),
            ModelMode::ImageEdit => [].to_vec(),
            ModelMode::AudioTranscription => MODELS_AUDIOTRANSCRIPTION.to_vec(),
        }
    }
}
