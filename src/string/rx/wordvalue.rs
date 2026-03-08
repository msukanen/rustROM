use std::fmt::Display;

use either::Either;
use regex::Captures;

use crate::string::robust_parse::RobustParse;

#[derive(Debug)]
pub enum WordValueError {
    RegexNotMatched,
    NotKnownValue(String),
    UnrecognizedUnit(String),
}

impl Display for WordValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotKnownValue(v) => write!(f, "'{}' is not a known value-word.", v),
            Self::RegexNotMatched => write!(f, "Regex capture didn't catch one of 'value', 'word', or 'unit'."),
            Self::UnrecognizedUnit(v) => {
                log::warn!("DEV: 'unit' in regex captures more than we have match arms for… To arms!");
                write!(f, "Unrecognized unit '{v}' instead of [smh]…")
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SplitMode {
    Quarter, Half, ThreeQuarters,
}

/// Word as numeric value, if possible…
pub(crate) fn word_as_value(caps: &Captures<'_>) -> Result<Either<u64, SplitMode>, WordValueError> {
    if let Some(mtch) = caps.name("value") {
        return Ok(Either::Left(mtch.as_str().parse::<u64>().unwrap()))
    }

    let Some(mtch) = caps.name("word") else {
        return Err(WordValueError::RegexNotMatched);
    };

    match mtch.as_str().to_lowercase().replace("-", "").replace(" ", "") {
        s if s.starts_with("half") => return Ok(Either::Right(SplitMode::Half)),
        s if s.starts_with("quarter") => return Ok(Either::Right(SplitMode::Quarter)),
        s if s.starts_with("threeq") ||
        s == "3q" || s == "3/4" => return Ok(Either::Right(SplitMode::ThreeQuarters)),
        _ => ()
    }
    
    match mtch.as_str().robust_parse::<u64>() {
        Ok(v) => Ok(Either::Left(v)),
        _ => Err(WordValueError::NotKnownValue(mtch.as_str().into())),
    }
}

pub(crate) fn value_or_splitmoded_value(caps: &Captures<'_>) -> Result<u64, WordValueError> {
    let Some(multi) = caps.name("unit") else {
        return Err(WordValueError::RegexNotMatched);
    };
    let sec = match multi.as_str().chars().nth(0).unwrap() {
        's' => 1,
        'm' => 60,
        'h' => 3600,
        other => return Err(WordValueError::UnrecognizedUnit(multi.as_str().into()))
    };

    match word_as_value(caps) {
        Err(e) => Err(e),
        Ok(Either::Left(v)) => Ok(sec * v),
        Ok(Either::Right(s)) => Ok(match s {
            SplitMode::Half => sec / 2,
            SplitMode::Quarter => sec / 4,
            SplitMode::ThreeQuarters => sec / 4 * 3,
        })
    }
}

pub(crate) trait TryValueFromCapture {
    fn try_value(&self) -> Result<u64, WordValueError>;
}

impl TryValueFromCapture for Captures<'_> {
    fn try_value(&self) -> Result<u64, WordValueError> {
        value_or_splitmoded_value(self)
    }
}
