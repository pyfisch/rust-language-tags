#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! Language tags can be used identify human languages, scripts e.g. Latin script, countries and
//! other regions.
//!
//! Language tags are defined in [BCP47](http://tools.ietf.org/html/bcp47), an introduction is
//! ["Language tags in HTML and XML"](http://www.w3.org/International/articles/language-tags/) by
//! the W3C. They are commonly used in HTML and HTTP `Content-Language` and `Accept-Language`
//! header fields.
//!
//! This package currently supports parsing (fully conformant parser), formatting and comparing
//! language tags.
//!
//! # Examples
//! Create a simple language tag representing the French language as spoken
//! in Belgium and print it:
//!
//! ```rust
//! use language_tags::LanguageTag;
//! let mut langtag: LanguageTag = Default::default();
//! langtag.language = Some("fr".to_owned());
//! langtag.region = Some("BE".to_owned());
//! assert_eq!(format!("{}", langtag), "fr-BE");
//! ```
//!
//! Parse a tag representing a special type of English specified by private agreement:
//!
//! ```rust
//! use language_tags::LanguageTag;
//! let langtag: LanguageTag = "en-x-twain".parse().unwrap();
//! assert_eq!(format!("{}", langtag.language.unwrap()), "en");
//! assert_eq!(format!("{:?}", langtag.privateuse), "[\"twain\"]");
//! ```
//!
//! You can check for equality, but more often you should test if two tags match.
//!
//! ```rust
//! use language_tags::LanguageTag;
//! let mut langtag_server: LanguageTag = Default::default();
//! langtag_server.language = Some("de".to_owned());
//! langtag_server.region = Some("AT".to_owned());
//! let mut langtag_user: LanguageTag = Default::default();
//! langtag_user.language = Some("de".to_owned());
//! assert!(langtag_user.matches(&langtag_server));
//! ```
//!
//! There is also the `langtag!` macro for creating language tags.

use std::ascii::AsciiExt;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error as ErrorTrait;
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;
use std::str::FromStr;

fn is_alphabetic(s: &str) -> bool {
    s.chars().all(|x| x >= 'A' && x <= 'Z' || x >= 'a' && x <= 'z')
}

fn is_numeric(s: &str) -> bool {
    s.chars().all(|x| x >= '0' && x <= '9')
}

fn is_alphanumeric_or_dash(s: &str) -> bool {
    s.chars().all(|x| x >= 'A' && x <= 'Z' || x >= 'a' && x <= 'z' ||
                      x >= '0' && x <= '9' || x == '-')
}

/// Defines an Error type for langtags.
///
/// Errors occur mainly during parsing of language tags.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// The same extension subtag is only allowed once in a tag before the private use part.
    DuplicateExtension,
    /// If an extension subtag is present, it must not be empty.
    EmptyExtension,
    /// If the `x` subtag is present, it must not be empty.
    EmptyPrivateUse,
    /// The langtag contains a char that is not A-Z, a-z, 0-9 or the dash.
    ForbiddenChar,
    /// A subtag fails to parse, it does not match any other subtags.
    InvalidSubtag,
    /// The given language subtag is invalid.
    InvalidLanguage,
    /// A subtag may be eight characters in length at maximum.
    SubtagTooLong,
    /// At maximum three extlangs are allowed, but zero to one extlangs are preferred.
    TooManyExtlangs,
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::DuplicateExtension => "The same extension subtag is only allowed once in a tag",
            Error::EmptyExtension => "If an extension subtag is present, it must not be empty",
            Error::EmptyPrivateUse => "If the `x` subtag is present, it must not be empty",
            Error::ForbiddenChar => "The langtag contains a char not allowed",
            Error::InvalidSubtag => "A subtag fails to parse, it does not match any other subtags",
            Error::InvalidLanguage => "The given language subtag is invalid",
            Error::SubtagTooLong => "A subtag may be eight characters in length at maximum",
            Error::TooManyExtlangs => "At maximum three extlangs are allowed",
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// Result type used for this library.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Contains the 17 irregular old language tags not matching the standard grammer of tags.
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

/// Contains the 9 regular grandfathered tags having special semantics.
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

/// A language tag as described in [BCP47](http://tools.ietf.org/html/bcp47).
///
/// Language tags are used to help identify languages, whether spoken,
/// written, signed, or otherwise signaled, for the purpose of
/// communication.  This includes constructed and artificial languages
/// but excludes languages not intended primarily for human
/// communication, such as programming languages.
#[derive(Debug, Default, Eq, Clone)]
pub struct LanguageTag {
    /// Language subtags are used to indicate the language, ignoring all
    /// other aspects such as script, region or spefic invariants.
    pub language: Option<String>,
    /// Extended language subtags are used to identify certain specially
    /// selected languages that, for various historical and compatibility
    /// reasons, are closely identified with or tagged using an existing
    /// primary language subtag.
    pub extlangs: Vec<String>,
    /// Script subtags are used to indicate the script or writing system
    /// variations that distinguish the written forms of a language or its
    /// dialects.
    pub script: Option<String>,
    /// Region subtags are used to indicate linguistic variations associated
    /// with or appropriate to a specific country, territory, or region.
    /// Typically, a region subtag is used to indicate variations such as
    /// regional dialects or usage, or region-specific spelling conventions.
    /// It can also be used to indicate that content is expressed in a way
    /// that is appropriate for use throughout a region, for instance,
    /// Spanish content tailored to be useful throughout Latin America.
    pub region: Option<String>,
    /// Variant subtags are used to indicate additional, well-recognized
    /// variations that define a language or its dialects that are not
    /// covered by other available subtags.
    pub variants: Vec<String>,
    /// Extensions provide a mechanism for extending language tags for use in
    /// various applications.  They are intended to identify information that
    /// is commonly used in association with languages or language tags but
    /// that is not part of language identification.
    pub extensions: BTreeMap<u8, Vec<String>>,
    /// Private use subtags are used to indicate distinctions in language
    /// that are important in a given context by private agreement.
    pub privateuse: Vec<String>
}

impl LanguageTag {
    /// Matches language tags. The first language acts as a language range, the second one is used
    /// as a normal language tag. None fields in the language range are ignored. If the language
    /// tag has more extlangs than the range these extlangs are ignored. Matches are
    /// case-insensitive. `*` in language ranges are represented using `None` values. The language
    /// range `*` that matches language tags is created by the default language tag:
    /// `let wildcard: LanguageTag = Default::default();.`
    ///
    /// For example the range `en-GB` matches only `en-GB` and `en-Arab-GB` but not `en`.
    /// The range `en` matches all language tags starting with `en` including `en`, `en-GB`,
    /// `en-Arab` and `en-Arab-GB`.
    ///
    /// # Panics
    /// If the language range has extensions or private use tags.
    ///
    /// # Examples
    /// ```
    /// # #[macro_use] extern crate language_tags;
    /// # fn main() {
    /// let range_italian = langtag!(it);
    /// let tag_german = langtag!(de);
    /// let tag_italian_switzerland = langtag!(it;;;CH);
    /// assert!(!range_italian.matches(&tag_german));
    /// assert!(range_italian.matches(&tag_italian_switzerland));
    ///
    /// let range_spanish_brazil = langtag!(es;;;BR);
    /// let tag_spanish = langtag!(es);
    /// assert!(!range_spanish_brazil.matches(&tag_spanish));
    /// # }
    /// ```
    pub fn matches(&self, other: &LanguageTag) -> bool {
        assert!(self.extensions.is_empty());
        assert!(self.privateuse.is_empty());
        return matches_option(&self.language, &other.language)
            && matches_vec(&self.extlangs, &other.extlangs)
            && matches_option(&self.script, &other.script)
            && matches_option(&self.region, &other.region)
            && matches_vec(&self.variants, &other.variants);

        fn matches_option(a: &Option<String>, b: &Option<String>) -> bool {
            match (a, b) {
                (&Some(ref a), &Some(ref b)) => a.eq_ignore_ascii_case(b),
                (&None, _) => true,
                (_, &None) => false,
            }
        }
        fn matches_vec(a: &Vec<String>, b: &Vec<String>) -> bool {
            a.iter().zip(b.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y))
        }
    }
}

impl PartialEq for LanguageTag {
    fn eq(&self, other: &LanguageTag) -> bool {
        return eq_option(&self.language, &other.language)
            && eq_vec(&self.extlangs, &other.extlangs)
            && eq_option(&self.script, &other.script)
            && eq_option(&self.region, &other.region)
            && eq_vec(&self.variants, &other.variants)
            && BTreeSet::from_iter(&self.extensions) == BTreeSet::from_iter(&other.extensions)
            && self.extensions.keys().all(|a|
                eq_vec(self.extensions.get(a).unwrap(), other.extensions.get(a).unwrap()))
            && eq_vec(&self.privateuse, &other.privateuse);

        fn eq_option(a: &Option<String>, b: &Option<String>) -> bool {
            match (a, b) {
                (&Some(ref a), &Some(ref b)) => a.eq_ignore_ascii_case(b),
                (&None, &None) => true,
                _ => false,
            }
        }
        fn eq_vec(a: &Vec<String>, b: &Vec<String>) -> bool {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.eq_ignore_ascii_case(y))
        }
    }
}

impl std::str::FromStr for LanguageTag {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let t = s.trim();
        if !is_alphanumeric_or_dash(t)  {
            return Err(Error::ForbiddenChar);
        }
        let mut langtag: LanguageTag = Default::default();
        // Handle grandfathered tags
        if let Some(tag) = GRANDFATHERED_IRREGULAR.iter()
                .chain(GRANDFATHERED_REGULAR.iter())
                .find(|x| x.eq_ignore_ascii_case(t)) {
            langtag.language = Some((*tag).to_owned());
            return Ok(langtag)
        }
        // Handle normal tags
        // The parser has a position from 0 to 6. Bigger positions reepresent the ASCII codes of
        // single character extensions
        // language-extlangs-script-region-variant-extension-privateuse
        // --- 0 -- -- 1 -- -- 2 - -- 3 - -- 4 -- --- x --- ---- 6 ---
        let mut position: u8 = 0;
        for subtag in t.split('-') {
            if subtag.len() > 8 {
                // > All subtags have a maximum length of eight characters.
                return Err(Error::SubtagTooLong);
            }
            if position == 6 {
                langtag.privateuse.push(subtag.to_owned());
            } else if subtag.eq_ignore_ascii_case("x") {
                position = 6;
            } else if position == 0 {
                // Primary language
                if subtag.len() < 2 || !is_alphabetic(subtag) {
                    return Err(Error::InvalidLanguage)
                }
                langtag.language = Some(subtag.to_owned());
                if subtag.len() < 4 {
                    // extlangs are only allowed for short language tags
                    position = 1;
                } else {
                    position = 2;
                }
            } else if position == 1 && subtag.len() == 3 && is_alphabetic(subtag) {
                // extlangs
                langtag.extlangs.push(subtag.to_owned());
                position = 2;
            } else if position == 2 && subtag.len() == 3 && is_alphabetic(subtag)
                    && !langtag.extlangs.is_empty() {
                // Multiple extlangs
                if langtag.extlangs.len() > 2 {
                    // maximum 3 extlangs
                    return Err(Error::TooManyExtlangs);
                }
                langtag.extlangs.push(subtag.to_owned());
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
                    return Err(Error::DuplicateExtension);
                }
                langtag.extensions.insert(position, Vec::new());
            } else if position > 6 {
                langtag.extensions.get_mut(&position).unwrap().push(subtag.to_owned());
            } else {
                return Err(Error::InvalidSubtag);
            }
        }
        if langtag.extensions.values().any(|x| x.is_empty()) {
            // Extensions and privateuse must not be empty if present
            return Err(Error::EmptyExtension);
        }
        if position == 6 && langtag.privateuse.is_empty() {
            return Err(Error::EmptyPrivateUse);
        }
        return Ok(langtag);
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref x) = self.language {
            try!(Display::fmt(x, f))
        }
        for x in self.extlangs.iter() {
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

#[macro_export]
/// Utility for creating simple language tags.
///
/// The macro supports the language, exlang, script and region parts of language tags,
/// they are separated by semicolons, omitted parts are denoted with mulitple semicolons.
///
/// # Examples
/// * `it`: `langtag!(it)`
/// * `it-LY`: `langtag!(it;;;LY)`
/// * `it-Arab-LY`: `langtag!(it;;Arab;LY)`
/// * `ar-afb`: `langtag!(ar;afb)`
/// * `i-enochian`: `langtag!(i-enochian)`
macro_rules! langtag {
    ( $language:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlangs: Vec::new(),
            script: None,
            region: None,
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;;;$region:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlangs: Vec::new(),
            script: None,
            region: Some(stringify!($region).to_owned()),
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;;$script:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlangs: Vec::new(),
            script: Some(stringify!($script).to_owned()),
            region: None,
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;;$script:expr;$region:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlangs: Vec::new(),
            script: Some(stringify!($script).to_owned()),
            region: Some(stringify!($region).to_owned()),
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;$extlangs:expr) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlangs: vec![stringify!($extlangs).to_owned()],
            script: None,
            region: None,
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;$extlangs:expr;$script:expr) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlangs: vec![stringify!($extlangs).to_owned()],
            script: Some(stringify!($script).to_owned()),
            region: None,
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;$extlangs:expr;;$region:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlangs: vec![stringify!($extlangs).to_owned()],
            script: None,
            region: Some(stringify!($region).to_owned()),
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
    ( $language:expr;$extlangs:expr;$script:expr;$region:expr ) => {
        $crate::LanguageTag {
            language: Some(stringify!($language).to_owned()),
            extlangs: vec![stringify!($extlangs).to_owned()],
            script: Some(stringify!($script).to_owned()),
            region: Some(stringify!($region).to_owned()),
            variants: Vec::new(),
            extensions: ::std::collections::BTreeMap::new(),
            privateuse: Vec::new(),
        }
    };
}
