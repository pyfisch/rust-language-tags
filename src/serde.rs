use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::LanguageTag;

impl Serialize for LanguageTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for LanguageTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input: &str = Deserialize::deserialize(deserializer)?;
        LanguageTag::parse(input).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let input = "\"en-Latn-gb-boont-r-extended-sequence-x-private\"";
        let deser: LanguageTag = serde_json::from_str(input).unwrap();
        deser.validate().unwrap();
        let ser = serde_json::to_string(&deser).unwrap();
        assert!(ser.eq_ignore_ascii_case(input));
    }
}
