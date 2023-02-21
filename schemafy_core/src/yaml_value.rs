use std::ops::{AddAssign, Neg, SubAssign};

use serde::Serialize
use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum YamlValue {
    Null,
    Bool(bool),
    Number(i64),
    String(String),
    Sequence(Vec<YamlValue>),
}

impl YamlValue {
    pub fn new(value: Value) -> Self {
        match &value {
            Value::Null => Self::Null,
            Value::Number(value) => {
                let value = if let Some(value) = value.as_i64() {
                    value
                } else {
                    let Some(value_f64) = value.as_f64() else {
                        panic!("Not allowed yaml value type: {:?}", value);
                    };
                    value_f64 as i64
                };
                Self::Number(value)
            }
            Value::Bool(value) => Self::Bool(*value),
            Value::String(value) => Self::String(value.clone()),
            Value::Sequence(value) => Self::Sequence(
                value
                    .iter()
                    .map(|value| Self::new(value.clone()))
                    .collect::<Vec<_>>(),
            ),
            Value::Mapping(value) => Self::String(serde_yaml::to_string(value).unwrap()),
        }
    }

    pub fn parse<'a, T: TryFrom<&'a YamlValue>>(&'a self) -> Option<T> {
        T::try_from(self).ok()
    }
}

impl Default for YamlValue {
    fn default() -> Self {
        Self::Null
    }
}

impl TryFrom<&YamlValue> for i64 {
    type Error = ();

    fn try_from(value: &YamlValue) -> Result<Self, Self::Error> {
        match value {
            YamlValue::Number(value) => Ok(*value),
            YamlValue::String(value) => Ok(value.parse().unwrap_or_else(|_| {
                panic!("Failed to parse i64 from string: {}", value)
            })),
            YamlValue::Bool(value) => Ok(if *value { 1 } else { 0 }),
            _ => Err(()),
        }
    }
}

impl TryFrom<&YamlValue> for i32 {
    type Error = ();

    fn try_from(value: &YamlValue) -> Result<Self, Self::Error> {
        match value {
            YamlValue::Number(value) => Ok(*value as i32),
            YamlValue::String(value) => Ok(value.parse().unwrap_or_else(|_| {
                panic!("Failed to parse i32 from string: {}", value)
            })),
            YamlValue::Bool(value) => Ok(if *value { 1 } else { 0 }),
            _ => Err(()),
        }
    }
}

impl TryFrom<&YamlValue> for f64 {
    type Error = ();

    fn try_from(value: &YamlValue) -> Result<Self, Self::Error> {
        match value {
            YamlValue::Number(value) => Ok(*value as f64),
            YamlValue::String(value) => Ok(value.parse().unwrap_or_else(|_| {
                panic!("Failed to parse f64 from string: {}", value)
            })),
            YamlValue::Bool(value) => Ok(if *value { 1.0 } else { 0.0 }),
            _ => Err(()),
        }
    }
}

impl TryFrom<&YamlValue> for u64 {
    type Error = ();

    fn try_from(value: &YamlValue) -> Result<Self, Self::Error> {
        match value {
            YamlValue::Number(value) => Ok(*value as u64),
            YamlValue::String(value) => Ok(value.parse().unwrap_or_else(|_| {
                panic!("Failed to parse u64 from string: {}", value)
            })),
            YamlValue::Bool(value) => Ok(if *value { 1 } else { 0 }),
            _ => Err(()),
        }
    }
}

impl TryFrom<&YamlValue> for u32 {
    type Error = ();

    fn try_from(value: &YamlValue) -> Result<Self, Self::Error> {
        match value {
            YamlValue::Number(value) => Ok(*value as u32),
            YamlValue::String(value) => Ok(value.parse().unwrap_or_else(|_| {
                panic!("Failed to parse u32 from string: {}", value)
            })),
            YamlValue::Bool(value) => Ok(if *value { 1 } else { 0 }),
            _ => Err(()),
        }
    }
}

impl TryFrom<&YamlValue> for String {
    type Error = ();

    fn try_from(value: &YamlValue) -> Result<Self, Self::Error> {
        match value {
            YamlValue::Number(value) => Ok(value.to_string()),
            YamlValue::String(value) => Ok(value.clone()),
            YamlValue::Bool(value) => Ok(value.to_string()),
            _ => Err(()),
        }
    }
}

impl TryFrom<&YamlValue> for bool {
    type Error = ();

    fn try_from(value: &YamlValue) -> Result<Self, Self::Error> {
        match value {
            YamlValue::Number(value) => Ok(*value != 0),
            YamlValue::String(value) => Ok(value.parse().unwrap_or_else(|_| {
                panic!("Failed to parse bool from string: {}", value)
            })),
            YamlValue::Bool(value) => Ok(*value),
            _ => Err(()),
        }
    }
}

impl<'a, T> TryFrom<&'a YamlValue> for Vec<T>
    where
        T: TryFrom<&'a YamlValue>,
{
    type Error = ();
    fn try_from(value: &'a YamlValue) -> Result<Self, Self::Error>
    {
        match value {
            YamlValue::Sequence(value) => {
                let result = value
                    .into_iter()
                    .map(|v|
                        T::try_from(v)
                            .unwrap_or_else(|_| panic!("Failed to convert value: {:?}", v))
                    ).collect();
                Ok(result)
            }
            _ => Err(()),
        }
    }
}

impl<'a, T, U> TryFrom<&'a YamlValue> for (T, U)
    where T: TryFrom<&'a YamlValue>,
          U: TryFrom<&'a YamlValue>,
{
    type Error = ();

    fn try_from(value: &'a YamlValue) -> Result<Self, Self::Error> {
        match value {
            YamlValue::Sequence(value) => {
                let mut iter = value.into_iter();
                let x = T::try_from(
                    iter.next().unwrap_or_else(|| panic!("Failed to convert value: {:?}", value))
                ).unwrap_or_else(|_| panic!("Failed to convert value: {:?}", value));
                let y = U::try_from(
                    iter.next().unwrap_or_else(|| panic!("Failed to convert value: {:?}", value))
                ).unwrap_or_else(|_| panic!("Failed to convert value: {:?}", value));
                Ok((x, y))
            }
            _ => Err(()),
        }
    }
}

impl From<i64> for YamlValue {
    fn from(value: i64) -> Self {
        YamlValue::Number(value)
    }
}

impl From<i32> for YamlValue {
    fn from(value: i32) -> Self {
        YamlValue::Number(value as i64)
    }
}

impl From<f64> for YamlValue {
    fn from(value: f64) -> Self {
        YamlValue::Number(value as i64)
    }
}

impl From<f32> for YamlValue {
    fn from(value: f32) -> Self {
        YamlValue::Number(value as i64)
    }
}

impl From<u64> for YamlValue {
    fn from(value: u64) -> Self {
        YamlValue::Number(value as i64)
    }
}

impl From<u32> for YamlValue {
    fn from(value: u32) -> Self {
        YamlValue::Number(value as i64)
    }
}

impl From<String> for YamlValue {
    fn from(value: String) -> Self {
        YamlValue::String(value)
    }
}

impl From<&str> for YamlValue {
    fn from(value: &str) -> Self {
        YamlValue::String(value.to_string())
    }
}

impl From<bool> for YamlValue {
    fn from(value: bool) -> Self {
        YamlValue::Bool(value)
    }
}

impl From<&YamlValue> for YamlValue {
    fn from(value: &YamlValue) -> Self {
        value.to_owned()
    }
}

impl AddAssign for YamlValue {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (YamlValue::Number(lhs), YamlValue::Number(rhs)) => {
                *lhs += rhs;
            }
            (YamlValue::String(lhs), YamlValue::String(rhs)) => {
                *lhs += &rhs;
            }
            (YamlValue::Sequence(lhs), YamlValue::Sequence(rhs)) => {
                lhs.extend(rhs);
            }
            _ => {}
        }
    }
}

impl SubAssign for YamlValue {
    fn sub_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (YamlValue::Number(lhs), YamlValue::Number(rhs)) => {
                *lhs -= rhs;
            }
            _ => {}
        }
    }
}

impl Neg for YamlValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            YamlValue::Number(value) => YamlValue::Number(-value),
            _ => panic!("Not allowed yaml value type: {:?}", self),
        }
    }
}