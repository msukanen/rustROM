//! Some word-to-numeric value parsing.
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ROBUST_TIME_RX: Regex = 
        Regex::new(&{let rx: String = r#"^\s*
            (?:
                (?:(?P<word>(
                    zero|none|nada|null|
                    one|once|
                    two|twice|
                    (?:three(?:[-]|\s)?quarters?)|thrice|3/4|3q|
                    four|five|six|seven|eight|nine|ten|
                    dozen|quarter|
                    fifteen|thirty|
                    fifty|sixty|
                    half
                ))\s+(?:of\s+(an?\s+)?)?)
              | (?:(?P<value>\d+)\s*)
            )
            (?P<unit>
                [smh]
                |
                sec(?:ond)?s?
                |
                min(?:ute)?s?
                |
                h(?:our)?s?
            )?
        \s*$"#
        .chars()
        .filter(|c| !c.is_whitespace()).collect(); rx })
        .unwrap_or_else(|e| {
            panic!("{e} while constructing ROBUST_TIME_RX… Fix it!")
        });
}

pub trait RobustFromStr: Sized {
    type Err;

    fn robust_from_str(s: &str) -> Result<Self, Self::Err>;
}

fn str_to_num(s: &str) -> Option<u64> {
    match s.trim().to_lowercase().as_str() {
        "zero"|"none"|"null"|"nada" => Some(0),
        "one"|"once" => Some(1),
        "two"|"twice" => Some(2),
        "three"|"thrice" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        
        // other...
        "dozen" => Some(12),
        "quarter"|"fifteen" => Some(15),
        "thirty" => Some(30),
        "minute"|"sixty" => Some(60),

        _ => None
    }
}

impl RobustFromStr for u64 {
    type Err = String;

    fn robust_from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(n) = str_to_num(s) {
            return Ok(n);
        }
        
        s.parse::<u64>().map_err(|e| e.to_string())
    }
}

pub trait RobustParse {
    fn robust_parse<T: RobustFromStr>(&self) -> Result<T, T::Err>;
}

impl RobustParse for str {
    fn robust_parse<T: RobustFromStr>(&self) -> Result<T, T::Err> {
        T::robust_from_str(self)
    }
}

#[cfg(test)]
mod robust_parse_tests {
    use either::Either;
    use regex::Captures;

    use crate::string::{robust_parse::{ROBUST_TIME_RX, RobustParse}, rx::wordvalue::{SplitMode, TryValueFromCapture, word_as_value}};

    #[test]
    fn robust_parse_known_str() {
        const STRS: [&'static str;23] = [
            "zero", "none", "null", "nada",
            "one", "once",
            "two", "twice",
            "three", "thrice",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "dozen",
            "quarter", "fifteen",
            "thirty",
            "minute", "sixty",
        ];
        const VALS: [u64;23] = [
            0,0,0,0,
            1,1,
            2,2,
            3,3,
            4,5,6,7,8,9,10,
            12,
            15,15,
            30,
            60,60
        ];

        for (idx, s) in STRS.iter().enumerate() {
            let _ = s.robust_parse::<u64>()
                .map_err(|e| panic!("{e}"))
                .and_then(|v|{ assert_eq!(VALS[idx], v); Ok(v)});
        }
    }

    #[test]
    fn regex_robust_parse() {
        let rx = &ROBUST_TIME_RX;// just to save some typing here...
        assert_eq!(3600, rx.captures("    1 hour   ").unwrap_or_else(|| panic!("1 hour didn't match?!")).try_value().unwrap());
        assert_eq!(1, rx.captures("   one   second   ").unwrap_or_else(|| panic!("'one hour' didn't match?!")).try_value().unwrap());
        assert_eq!(30, rx.captures("  half    min   ").unwrap_or_else(|| panic!("'half min' didn't match?!")).try_value().unwrap());
        assert_eq!(45*60, rx.captures("  3/4    of     hour ").unwrap_or_else(|| panic!("'3/4 of hour' didn't match?!")).try_value().unwrap());
        assert_eq!(45, rx.captures("  3q    of    a     min  ").unwrap_or_else(|| panic!("'3q of a min' didn't match?!")).try_value().unwrap());
        assert_eq!(45*60, rx.captures("  three-quarters   of   hour ").unwrap_or_else(|| panic!("'three-quarters of hour didn't match?!")).try_value().unwrap());
    }
}
