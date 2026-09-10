use crate::{Error, Result};
use serde::{
    de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::{Map, Number, Value};
use std::fmt;
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON without duplicate keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Number(v.into())))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Number(v.into())))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Unique, E> {
                Number::from_f64(v)
                    .map(|n| Unique(Value::Number(n)))
                    .ok_or(E::custom("nonfinite number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::String(v.into())))
            }
            fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::String(v)))
            }
            fn visit_none<E: de::Error>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_unit<E: de::Error>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut out = Vec::new();
                while let Some(Unique(v)) = a.next_element()? {
                    out.push(v);
                }
                Ok(Unique(Value::Array(out)))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut out = Map::new();
                while let Some((k, Unique(v))) = a.next_entry::<String, Unique>()? {
                    if out.insert(k, v).is_some() {
                        return Err(de::Error::custom("duplicate JSON key"));
                    }
                }
                Ok(Unique(Value::Object(out)))
            }
        }
        d.deserialize_any(V)
    }
}
pub fn parse<T: DeserializeOwned>(input: &str) -> Result<T> {
    let Unique(v) = serde_json::from_str(input).map_err(|e| Error(e.to_string()))?;
    serde_json::from_value(v).map_err(|e| Error(e.to_string()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_keys_rejected_recursively() {
        for s in [
            r#"{"a":1,"a":2}"#,
            r#"{"nested":{"a":1,"a":2}}"#,
            r#"[{"x":1,"x":2}]"#,
        ] {
            assert!(parse::<Value>(s).is_err());
        }
        assert!(parse::<Value>(r#"{"a":1,"b":[true,null]}"#).is_ok());
    }
}
