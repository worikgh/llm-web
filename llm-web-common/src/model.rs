use std::str::FromStr;
use serde::Deserialize;
use serde::Serialize;
use serde:: Serializer;
#[derive(PartialEq, Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
pub enum Model {
    Gpt3,
    Gpt4o,
    Gpt4oMini,
    O1,
    O1Mini,
}

impl TryFrom<String> for Model {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Model::from_str(&value)
    }
}

impl Model {
    pub fn variants() -> Vec<Self> {
	vec![
            Model::Gpt3,
            Model::Gpt4o,
            Model::Gpt4oMini,
            Model::O1,
            Model::O1Mini,
	]
    }
    pub fn as_str(&self) -> &str {
        match self {
            Model::Gpt3 => "gpt-3.5-turbo",
            Model::Gpt4o => "gpt-4o",
            Model::Gpt4oMini => "gpt-4o-mini",
            Model::O1 => "o1",
            Model::O1Mini => "o1-mini",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Model::Gpt3 => "GPT-3.5 Turbo",
            Model::Gpt4o => "GPT-4o",
            Model::Gpt4oMini => "GPT-4o Mini",
            Model::O1 => "O1",
            Model::O1Mini => "O1-Mini",
        }
    }

}

// Implement custom serialization
impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl FromStr for Model {
    type Err = String;
    fn from_str(s: &str) -> Result<Model, Self::Err> {
        match s {
            "gpt-3.5-turbo" => Ok(Model::Gpt3),
            "gpt-4o" => Ok(Model::Gpt4o),
            "gpt-4o-mini" => Ok(Model::Gpt4oMini),
            "o1" => Ok(Model::O1),
            "o1-mini" => Ok(Model::O1Mini),
            _ => Err(format!("Cannot create a Model from {s}")),
        }
    }
}
impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_model_serialization() {
        assert_eq!(serde_json::to_string(&Model::Gpt3).unwrap(), "\"gpt-3.5-turbo\"");
        assert_eq!(serde_json::to_string(&Model::Gpt4o).unwrap(), "\"gpt-4o\"");
        assert_eq!(serde_json::to_string(&Model::Gpt4oMini).unwrap(), "\"gpt-4o-mini\"");
        assert_eq!(serde_json::to_string(&Model::O1).unwrap(), "\"o1\"");
        assert_eq!(serde_json::to_string(&Model::O1Mini).unwrap(), "\"o1-mini\"");
    }

    #[test]
    fn test_model_display() {
        assert_eq!(format!("{}", Model::Gpt3), "GPT-3.5 Turbo");
        assert_eq!(format!("{}", Model::Gpt4o), "GPT-4o");
        assert_eq!(format!("{}", Model::Gpt4oMini), "GPT-4o Mini");
        assert_eq!(format!("{}", Model::O1), "O1");
        assert_eq!(format!("{}", Model::O1Mini), "O1-Mini");
    }
    #[test]
    fn test_from_str() {
        assert_eq!(Model::from_str("gpt-3.5-turbo"), Ok(Model::Gpt3));
        assert_eq!(Model::from_str("gpt-4o"), Ok(Model::Gpt4o));
        assert_eq!(Model::from_str("gpt-4o-mini"), Ok(Model::Gpt4oMini));
        assert_eq!(Model::from_str("o1"), Ok(Model::O1));
        assert_eq!(Model::from_str("o1-mini"), Ok(Model::O1Mini));
    }
}
