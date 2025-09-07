use crate::string::alpha::is_a_vocal;

pub trait PigLatin {
    fn to_platin(&self) -> String;
}

impl PigLatin for &str {
    fn to_platin(&self) -> String {
        if self.is_empty() || self.len() <= 2 { return self.to_string(); }

        fn maybe_capitalized(w: &str, fst_is_uc: bool) -> String {
            if fst_is_uc {
                let mut chars = w.chars();
                return match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    _ => String::new()
                }
            }
            w.into()
        }

        let words = self.split(' ')
            .map(|w| {
                let fst = w.chars().nth(0).unwrap();
                let fst_is_uc = fst.is_uppercase();
                let last = w.chars().rev().nth(0).unwrap();
                let last_is_alpha = last.is_alphabetic();
                if is_a_vocal(fst) {
                    if last_is_alpha {
                        format!("{w}hay")
                    } else {
                        let word = maybe_capitalized(&w.chars().take(w.chars().count() - 1).collect::<String>(), fst_is_uc);
                        format!("{word}hay{last}")
                    }
                } else {
                    if last_is_alpha {
                        let word = maybe_capitalized(&w.chars().skip(1).collect::<String>(), fst_is_uc);
                        format!("{word}{}ay", fst.to_ascii_lowercase())
                    } else {
                        let word = maybe_capitalized(&w.chars().skip(1).take(w.chars().count() - 2).collect::<String>(), fst_is_uc);
                        format!("{word}{}ay{last}", fst.to_ascii_lowercase())
                    }
                }
            })
            .collect::<Vec<String>>();
        words.join(" ")
    }
}

#[cfg(test)]
mod piglatin_tests {
    use super::*;

    #[test]
    fn piglatin_helloworld() {
        let src = "Hello world! All OK?";
        let pl = src.to_platin();
        assert_eq!("Ellohay orldway! Allhay OKhay?".to_string(), pl);
    }
}
