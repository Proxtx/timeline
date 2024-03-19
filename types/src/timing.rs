use chrono::DateTime;
use chrono::Utc;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Marker {
    pub time: DateTime<Utc>,
    pub amount: u32,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct TimeRange {
    pub start: chrono::DateTime<Utc>,
    pub end: chrono::DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Timing {
    Range(TimeRange),
    Instant(DateTime<Utc>),
}

impl Serialize for Timing {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let res = match self {
            Timing::Instant(t) => {
                let nanos = match t.timestamp_nanos_opt() {
                    Some(v) => v,
                    None => {
                        return Err(serde::ser::Error::custom(
                            "Unable to transform into nano-seconds",
                        ));
                    }
                };
                vec![nanos]
            }
            Timing::Range(range) => {
                match (
                    range.start.timestamp_nanos_opt(),
                    range.end.timestamp_nanos_opt(),
                ) {
                    (Some(start), Some(end)) => {
                        vec![start, end]
                    }
                    _ => {
                        return Err(serde::ser::Error::custom(
                            "Unable to transform into nano-seconds",
                        ));
                    }
                }
            }
        };
        serializer.collect_seq(res)
    }
}

impl<'de> Deserialize<'de> for Timing {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(TimingVisitor)
    }
}

struct TimingVisitor;

impl<'de> Visitor<'de> for TimingVisitor {
    type Value = Timing;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a list of either 1 or 2 values indicating a range or instant")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut seq_2: Vec<DateTime<Utc>> = vec![];
        while let Some(val) = seq.next_element()? {
            /*seq_2.push(match DateTime::from_timestamp_nanos(val) {
                Some(v) => v,
                None => {
                    return Err(serde::de::Error::custom(
                        "Unable to parse milliseconds into DateTime: {}",
                    ))
                }
            });*/
            seq_2.push(DateTime::from_timestamp_nanos(val))
        }

        match seq_2.len() {
            1 => Ok(Timing::Instant(seq_2[0])),
            2 => Ok(Timing::Range(TimeRange {
                start: seq_2[0],
                end: seq_2[1],
            })),
            _ => Err(serde::de::Error::custom(
                "Unable to parse timing, since too many or too few values were provided",
            )),
        }
    }
}
