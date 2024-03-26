use serde::de::{self, Visitor};
use serde::Deserializer;
use std::fmt;

// Custom field deserializer for hex-encoded string to Vec<u8>.
pub fn hex_to_vec<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct HexVisitor;

    impl<'de> Visitor<'de> for HexVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing hex-encoded data")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            hex::decode(value).map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_str(HexVisitor)
}

// Custom field serializer for Vec<u8> to hex-encoded string.
pub fn vec_to_hex<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&hex::encode(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // Define a dummy struct to use for (de)serialization testing.
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct HexEncoded {
        #[serde(deserialize_with = "hex_to_vec", serialize_with = "vec_to_hex")]
        data: Vec<u8>,
    }

    #[test]
    fn test_parsing_valid_hex() {
        let json_str = r#"{"data": "48656c6c6f"}"#; // "Hello" in hex.
        let expected = HexEncoded {
            data: b"Hello".to_vec(),
        };

        let result: HexEncoded = serde_json::from_str(json_str).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parsing_invalid_hex() {
        let json_str = r#"{"data": "foobar"}"#; // Invalid hex characters.

        let result: Result<HexEncoded, _> = serde_json::from_str(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_serializing_to_hex() {
        let encoded = HexEncoded {
            data: b"Hello".to_vec(),
        };

        let result = serde_json::to_string(&encoded).unwrap();
        assert_eq!(result, r#"{"data":"48656c6c6f"}"#); // "Hello" in hex.
    }
}
