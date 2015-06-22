//#![deny(missing_docs)]

//! Language tags can be used identify human languages, scripts e.g. Latin script, countries and
//! other regions.
//!
//! Language tags are defined in [BCP47](http://tools.ietf.org/html/bcp47), an introduction is
//! ["Language tags in HTML and XML"](http://www.w3.org/International/articles/language-tags/) by
//! the W3C. They are commonly used in HTML and HTTP `Content-Language` and `Accept-Language`
//! header fields.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::default::Default;
use std::ascii::AsciiExt;
use std::collections::BTreeMap;

#[derive(Debug, Eq, PartialEq)]
pub struct Error;

fn is_alphabetic(s: &str) -> bool {
    s.chars().all(|x| x >= 'A' && x <= 'Z' || x >= 'a' && x <= 'z')
}

fn is_numeric(s: &str) -> bool {
    s.chars().all(|x| x >= '0' && x <= '9')
}

fn is_alphanumeric_or_dash(s: &str) -> bool {
    s.chars().all(|x| x >= 'A' && x <= 'Z' || x >= 'a' && x <= 'z' || x >= '0' && x <= '9' || x == '-')
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub const GRANDFATHERED_IRREGULAR: [&'static str; 17] = [
    "en-GB-oed",
    "i-ami",
    "i-bnn",
    "i-default",
    "i-enochian",
    "i-hak",
    "i-klingon",
    "i-lux",
    "i-mingo",
    "i-navajo",
    "i-pwn",
    "i-tao",
    "i-tay",
    "i-tsu",
    "sgn-BE-FR",
    "sgn-BE-NL",
    "sgn-CH-DE"];

pub const GRANDFATHERED_REGULAR: [&'static str; 9] = [
    "art-lojban",
    "cel-gaulish",
    "no-bok",
    "no-nyn",
    "zh-guoyu",
    "zh-hakka",
    "zh-min",
    "zh-min-nan",
    "zh-xiang"];

/// A language tag as desribed in [BCP47](http://tools.ietf.org/html/bcp47)
#[derive(Debug, Eq, Clone)]
pub struct LanguageTag {
    pub language: Option<String>,
    pub extlang: Option<String>,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
    pub extensions: BTreeMap<u8, Vec<String>>,
    pub privateuse: Vec<String>
}

impl LanguageTag {
    /// Matches language tags like described in
    /// [rfc4647#Extended filtering](https://tools.ietf.org/html/rfc4647#section-3.3.2)
    ///
    /// For example `en-GB` matches only `en-GB` and `en-Arab-GB` but not `en`. While `en` matches
    /// all of `en`, `en-GB` ,`en-Arab` and `en-Arab-GB`.
    pub fn matches(&self, other: &LanguageTag) -> bool {
        return matches_option_ignore_ascii_case(&self.language, &other.language) &&
        matches_option_ignore_ascii_case(&self.extlang, &other.extlang) &&
        matches_option_ignore_ascii_case(&self.script, &other.script) &&
        matches_option_ignore_ascii_case(&self.region, &other.region) &&
        self.variants.iter().all(|x| other.variants.iter().all(|y| x.eq_ignore_ascii_case(y))) &&
        self.privateuse.len() == other.privateuse.len() &&
        self.privateuse.iter().zip(other.privateuse.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y));

        fn matches_option_ignore_ascii_case(a: &Option<String>, b: &Option<String>) -> bool {
            match (a.is_some(), b.is_some()) {
                (true, true) => a.as_ref().unwrap().eq_ignore_ascii_case(b.as_ref().unwrap()),
                (false, false) => true,
                (true, false) => false,
                (false, true) => true,
            }

        }
    }
}

impl PartialEq for LanguageTag {
    fn eq(&self, other: &LanguageTag) -> bool {
        return eq_option_ignore_ascii_case(&self.language, &other.language) &&
        eq_option_ignore_ascii_case(&self.extlang, &other.extlang) &&
        eq_option_ignore_ascii_case(&self.script, &other.script) &&
        eq_option_ignore_ascii_case(&self.region, &other.region) &&
        self.variants.iter().all(|x| other.variants.iter().all(|y| x.eq_ignore_ascii_case(y))) &&
        self.privateuse.len() == other.privateuse.len() &&
        self.privateuse.iter().zip(other.privateuse.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y));

        fn eq_option_ignore_ascii_case(a: &Option<String>, b: &Option<String>) -> bool {
            match (a.is_some(), b.is_some()) {
                (true, true) => a.as_ref().unwrap().eq_ignore_ascii_case(b.as_ref().unwrap()),
                (false, false) => true,
                _ => false,
            }

        }
    }
}

impl Default for LanguageTag {
    fn default() -> LanguageTag {
        LanguageTag {
            language: None,
            extlang: None,
            script: None,
            region: None,
            variants: Vec::new(),
            extensions: BTreeMap::new(),
            privateuse: Vec::new(),
        }
    }
}

impl std::str::FromStr for LanguageTag {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let t = s.trim();
        if !is_alphanumeric_or_dash(t)  {
            return Err(Error);
        }
        // Handle grandfathered tags
        if let Some(tag) = GRANDFATHERED_IRREGULAR.iter().find(|x| x.eq_ignore_ascii_case(t)) {
            return Ok(simple_langtag(tag))
        }
        if let Some(tag) = GRANDFATHERED_REGULAR.iter().find(|x| x.eq_ignore_ascii_case(t)) {
            return Ok(simple_langtag(tag))
        }
        // Handle normal tags
        // The parser has a position from 0 to 6. Bigger positions reepresent the ASCII codes of
        // single character extensions
        // language-extlang-script-region-variant-extension-privateuse
        // --- 0 -- -- 1 -- -- 2 - -- 3 - -- 4 -- --- x --- ---- 6 ---
        let mut langtag: LanguageTag = Default::default();
        let mut position: u8 = 0;
        for subtag in t.split('-') {
            if subtag.len() > 8 {
                // > All subtags have a maximum length of eight characters.
                return Err(Error);
            }
            if position == 6 {
                langtag.privateuse.push(subtag.to_owned());
            } else if subtag.eq_ignore_ascii_case("x") {
                position = 6;
            } else if position == 0 {
                // Primary language
                if subtag.len() < 2 || !is_alphabetic(subtag) {
                    return Err(Error)
                }
                langtag.language = Some(subtag.to_owned());
                if subtag.len() < 4 {
                    // Extlangs are only allowed for short language tags
                    position = 1;
                } else {
                    position = 2;
                }
            } else if position == 1 && subtag.len() == 3 && is_alphabetic(subtag) {
                // Extlang
                langtag.extlang = Some(subtag.to_owned());
                position = 2;
            } else if position == 2 && subtag.len() == 3 && is_alphabetic(subtag)
                    && langtag.extlang.is_some() {
                // Multiple extlangs
                let x = [langtag.extlang.unwrap(), subtag.to_owned()].connect("-");
                if x.len() > 11 {
                    // maximum 3 extlangs
                    return Err(Error);
                }
                langtag.extlang = Some(x);
            } else if position <= 2 && subtag.len() == 4 && is_alphabetic(subtag) {
                // Script
                langtag.script = Some(subtag.to_owned());
                position = 3;
            } else if position <= 3 && (subtag.len() == 2 && is_alphabetic(subtag) ||
                    subtag.len() == 3 && is_numeric(subtag)) {
                langtag.region = Some(subtag.to_owned());
                position = 4;
            } else if position <= 4 && (subtag.len() >= 5 && is_alphabetic(&subtag[0..1]) ||
                    subtag.len() >= 4 && is_numeric(&subtag[0..1])) {
                // Variant
                langtag.variants.push(subtag.to_owned());
                position = 4;
            } else if subtag.len() == 1 {
                position = subtag.chars().next().unwrap() as u8;
                if langtag.extensions.contains_key(&position) {
                    return Err(Error);
                }
                langtag.extensions.insert(position, Vec::new());
            } else if position > 6 {
                langtag.extensions.get_mut(&position).unwrap().push(subtag.to_owned());
            } else {
                return Err(Error);
            }
        }
        if langtag.extensions.values().any(|x| x.is_empty()) || position == 6 && langtag.privateuse.is_empty() {
            // Extensions and privateuse must not be empty if present
            return Err(Error);
        }
        return Ok(langtag);

        fn simple_langtag(s: &str) -> LanguageTag {
            let mut x: LanguageTag = Default::default();
            x.language = Some(s.to_owned());
            x
        }
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref x) = self.language {
            try!(Display::fmt(x, f))
        }
        if let Some(ref x) = self.extlang {
            try!(write!(f, "-{}", x));
        }
        if let Some(ref x) = self.script {
            try!(write!(f, "-{}", x));
        }
        if let Some(ref x) = self.region {
            try!(write!(f, "-{}", x));
        }
        for x in self.variants.iter() {
            try!(write!(f, "-{}", x));
        }
        for (raw_key, values) in self.extensions.iter() {
            let mut key = String::new();
            key.push(*raw_key as char);
            try!(write!(f, "-{}", key));
            //parts.push(key);
            for value in values {
                try!(write!(f, "-{}", value));
            }
        }
        if !self.privateuse.is_empty() {
            if self.language.is_none() {
                try!(f.write_str("x"));
            } else {
                try!(f.write_str("-x"));
            }
            for value in self.privateuse.iter() {
                try!(write!(f, "-{}", value));
            }
        }
        Ok(())
    }
}
