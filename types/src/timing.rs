use {
    chrono::DateTime, chrono::Local, chrono::Timelike, chrono::Utc, serde::de::Visitor,
    serde::Deserialize, serde::Serialize, std::cmp::Ordering, std::fmt,
};

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

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}",
            date_time_to_string(&self.start),
            date_time_to_string(&self.end)
        )
    }
}

fn date_time_to_string(time: &DateTime<Utc>) -> String {
    let local = DateTime::<Local>::from(*time);
    format!(
        "{}:{}",
        prefix_number(local.hour()),
        prefix_number(local.minute())
    )
}

fn prefix_number(num: u32) -> String {
    if num < 10 {
        format!("0{}", num)
    } else {
        format!("{}", num)
    }
}

impl TimeRange {
    pub fn overlap_range(&self, other: &TimeRange) -> bool {
        (other.start >= self.start && other.start < self.end)
            || (other.end > self.start && other.end <= self.end)
            || (self.start >= other.start && self.start < other.end)
            || (self.end > other.start && self.end <= other.end)
    }

    pub fn includes(&self, other: &DateTime<Utc>) -> bool {
        other >= &self.start && other < &self.end
    }

    pub fn overlap_timing(&self, other: &Timing) -> bool {
        match other {
            Timing::Instant(o) => self.includes(o),
            Timing::Range(o) => self.overlap_range(o),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Timing {
    Range(TimeRange),
    Instant(DateTime<Utc>),
}

impl Timing {
    pub fn overlap(&self, other: &Timing) -> bool {
        match (self, other) {
            (Self::Range(s), Self::Range(o)) => s.overlap_range(o),
            (Self::Instant(s), Self::Range(o)) => o.includes(s),
            (Self::Range(s), Self::Instant(o)) => s.includes(o),
            (Self::Instant(s), Self::Instant(o)) => s == o,
        }
    }

    pub fn cmp(&self, other: &Timing) -> Ordering {
        match self {
            Timing::Instant(t) => t.cmp(match other {
                Self::Instant(t) => t,
                Self::Range(r) => &r.start,
            }),
            Timing::Range(r) => r.start.cmp(match other {
                Self::Instant(t) => t,
                Self::Range(o) => &o.start,
            }),
        }
    }
}

impl fmt::Display for Timing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Timing::Instant(v) => {
                write!(f, "{}", date_time_to_string(v))
            }
            Timing::Range(r) => {
                write!(f, "{}", r)
            }
        }
    }
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
