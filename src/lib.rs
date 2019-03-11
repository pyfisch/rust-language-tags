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
//! let langtag = LanguageTag::parse("fr-BE").unwrap();
//! assert_eq!(format!("{}", langtag), "fr-BE");
//! ```
//!
//! Parse a tag representing a special type of English specified by private agreement:
//!
//! ```rust
//! use language_tags::LanguageTag;
//! use std::iter::FromIterator;
//! let langtag: LanguageTag = "en-x-twain".parse().unwrap();
//! assert_eq!(langtag.primary_language(), "en");
//! assert_eq!(Vec::from_iter(langtag.private_use_subtags()), vec!["twain"]);
//! ```
//!
//! You can check for equality, but more often you should test if two tags match.
//! In this example we check if the resource in German language is suitable for
//! a user from Austria. While people speaking Austrian German normally understand
//! standard German the opposite is not always true. So the resource can be presented
//! to the user but if the resource was in `de-AT` and a user asked for a representation
//! in `de` the request should be rejected.
//!
//!
//! ```rust
//! use language_tags::LanguageTag;
//! let mut langtag_server = LanguageTag::parse("de-AT").unwrap();
//! let mut langtag_user = LanguageTag::parse("de").unwrap();
//! assert!(langtag_user.matches(&langtag_server));
//! ```

use std::error::Error;
use std::fmt;
use std::iter::once;
use std::str::Split;

/// Contains all grandfathered tags.
pub const GRANDFATHERED: [(&str, Option<&str>); 26] = [
    ("art-lojban", Some("jbo")),
    ("cel-gaulish", None),
    ("en-GB-oed", Some("en-GB-oxendict")),
    ("i-ami", Some("ami")),
    ("i-bnn", Some("bnn")),
    ("i-default", None),
    ("i-enochian", None),
    ("i-hak", Some("hak")),
    ("i-klingon", Some("tlh")),
    ("i-lux", Some("lb")),
    ("i-mingo", None),
    ("i-navajo", Some("nv")),
    ("i-pwn", Some("pwn")),
    ("i-tao", Some("tao")),
    ("i-tay", Some("tay")),
    ("i-tsu", Some("tsu")),
    ("no-bok", Some("nb")),
    ("no-nyn", Some("nn")),
    ("sgn-BE-FR", Some("sfb")),
    ("sgn-BE-NL", Some("vgt")),
    ("sgn-CH-DE", Some("sgg")),
    ("zh-guoyu", Some("cmn")),
    ("zh-hakka", Some("hak")),
    ("zh-min", None),
    ("zh-min-nan", Some("nan")),
    ("zh-xiang", Some("hsn")),
];

const DEPRECATED_LANGUAGE: [(&str, &str); 53] = [
    ("in", "id"),
    ("iw", "he"),
    ("ji", "yi"),
    ("jw", "jv"),
    ("mo", "ro"),
    ("aam", "aas"),
    ("adp", "dz"),
    ("aue", "ktz"),
    ("ayx", "nun"),
    ("bjd", "drl"),
    ("ccq", "rki"),
    ("cjr", "mom"),
    ("cka", "cmr"),
    ("cmk", "xch"),
    ("drh", "khk"),
    ("drw", "prs"),
    ("gav", "dev"),
    ("gfx", "vaj"),
    ("gti", "nyc"),
    ("hrr", "jal"),
    ("ibi", "opa"),
    ("ilw", "gal"),
    ("kgh", "kml"),
    ("koj", "kwv"),
    ("kwq", "yam"),
    ("kxe", "tvd"),
    ("lii", "raq"),
    ("lmm", "rmx"),
    ("meg", "cir"),
    ("mst", "mry"),
    ("mwj", "vaj"),
    ("myt", "mry"),
    ("nnx", "ngv"),
    ("oun", "vaj"),
    ("pcr", "adx"),
    ("pmu", "phr"),
    ("ppr", "lcq"),
    ("puz", "pub"),
    ("sca", "hle"),
    ("thx", "oyb"),
    ("tie", "ras"),
    ("tkk", "twm"),
    ("tlw", "weo"),
    ("tnf", "prs"),
    ("tsf", "taj"),
    ("uok", "ema"),
    ("xia", "acn"),
    ("xsj", "suj"),
    ("ybd", "rki"),
    ("yma", "lrr"),
    ("ymt", "mtm"),
    ("yos", "zom"),
    ("yuu", "yug"),
];

const DEPRECATED_REGION: [(&str, &str); 6] = [
    ("BU", "MM"),
    ("DD", "DE"),
    ("FX", "FR"),
    ("TP", "TL"),
    ("YD", "YE"),
    ("ZR", "CD"),
];

/// A language tag as described in [RFC 5646](https://tools.ietf.org/html/rfc5646).
///
/// Language tags are used to help identify languages, whether spoken,
/// written, signed, or otherwise signaled, for the purpose of
/// communication.  This includes constructed and artificial languages
/// but excludes languages not intended primarily for human
/// communication, such as programming languages.
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct LanguageTag {
    /// Syntax described in [RFC 5646 2.1](https://tools.ietf.org/html/rfc5646#section-2.1)
    serialization: String,
    language_end: usize,
    extlang_end: usize,
    script_end: usize,
    region_end: usize,
    variant_end: usize,
    extension_end: usize,
}

impl LanguageTag {
    /// Return the serialization of this language tag.
    ///
    /// This is fast since that serialization is already stored in the `LanguageTag` struct.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.serialization
    }

    /// Return the serialization of this language tag.
    ///
    /// This consumes the `LanguageTag` and takes ownership of the `String` stored in it.
    #[inline]
    pub fn into_string(self) -> String {
        self.serialization
    }

    /// Return the [primary language subtag](https://tools.ietf.org/html/rfc5646#section-2.2.1).
    #[inline]
    pub fn primary_language(&self) -> &str {
        &self.serialization[..self.language_end]
    }

    /// Return the [extended language subtags](https://tools.ietf.org/html/rfc5646#section-2.2.2).
    ///
    /// Valid language tags have at most one extended language.
    #[inline]
    pub fn extended_language(&self) -> Option<&str> {
        if self.language_end == self.extlang_end {
            None
        } else {
            Some(&self.serialization[self.language_end + 1..self.extlang_end])
        }
    }

    /// Iterate on the [extended language subtags](https://tools.ietf.org/html/rfc5646#section-2.2.2).
    ///
    /// Valid language tags have at most one extended language.
    #[inline]
    pub fn extended_language_subtags(&self) -> impl Iterator<Item = &str> {
        match self.extended_language() {
            Some(parts) => SubtagListIterator::new(parts),
            None => SubtagListIterator::new(""),
        }
    }

    /// Return the [primary language subtag](https://tools.ietf.org/html/rfc5646#section-2.2.1)
    /// and its [extended language subtags](https://tools.ietf.org/html/rfc5646#section-2.2.2).
    #[inline]
    pub fn full_language(&self) -> &str {
        &self.serialization[..self.extlang_end]
    }

    /// Return the [script subtag](https://tools.ietf.org/html/rfc5646#section-2.2.3).
    #[inline]
    pub fn script(&self) -> Option<&str> {
        if self.extlang_end == self.script_end {
            None
        } else {
            Some(&self.serialization[self.extlang_end + 1..self.script_end])
        }
    }

    /// Return the [region subtag](https://tools.ietf.org/html/rfc5646#section-2.2.4).
    #[inline]
    pub fn region(&self) -> Option<&str> {
        if self.script_end == self.region_end {
            None
        } else {
            Some(&self.serialization[self.script_end + 1..self.region_end])
        }
    }

    /// Return the [variant subtags](https://tools.ietf.org/html/rfc5646#section-2.2.5).
    #[inline]
    pub fn variant(&self) -> Option<&str> {
        if self.region_end == self.variant_end {
            None
        } else {
            Some(&self.serialization[self.region_end + 1..self.variant_end])
        }
    }

    /// Iterate on the [variant subtags](https://tools.ietf.org/html/rfc5646#section-2.2.5).
    #[inline]
    pub fn variant_subtags(&self) -> impl Iterator<Item = &str> {
        match self.variant() {
            Some(parts) => SubtagListIterator::new(parts),
            None => SubtagListIterator::new(""),
        }
    }

    /// Return the [extension subtags](https://tools.ietf.org/html/rfc5646#section-2.2.6).
    #[inline]
    pub fn extension(&self) -> Option<&str> {
        if self.variant_end == self.extension_end {
            None
        } else {
            Some(&self.serialization[self.variant_end + 1..self.extension_end])
        }
    }

    /// Iterate on the [extension subtags](https://tools.ietf.org/html/rfc5646#section-2.2.6).
    #[inline]
    pub fn extension_subtags(&self) -> impl Iterator<Item = (char, &str)> {
        match self.extension() {
            Some(parts) => ExtensionsIterator::new(parts),
            None => ExtensionsIterator::new(""),
        }
    }

    /// Return the [private use subtags](https://tools.ietf.org/html/rfc5646#section-2.2.7).
    #[inline]
    pub fn private_use(&self) -> Option<&str> {
        if self.serialization.starts_with("x-") {
            Some(&self.serialization)
        } else if self.extension_end == self.serialization.len() {
            None
        } else {
            Some(&self.serialization[self.extension_end + 1..])
        }
    }

    /// Iterate on the [private use subtags](https://tools.ietf.org/html/rfc5646#section-2.2.7).
    #[inline]
    pub fn private_use_subtags(&self) -> impl Iterator<Item = &str> {
        match self.private_use() {
            Some(parts) => SubtagListIterator::new(&parts[2..]),
            None => SubtagListIterator::new(""),
        }
    }

    /// Create a `LanguageTag` from its serialization.
    ///
    /// This parser accepts the language tags that are "well-formed" according to
    /// [RFC 5646](https://tools.ietf.org/html/rfc5646#section-2.2.9).
    /// Full validation could be done with the `validate` method.
    ///
    ///
    /// # Errors
    ///
    /// If the language tag is not "well-formed" a `ParseError` variant will be returned.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        //grandfathered tags
        if let Some((tag, _)) = GRANDFATHERED
            .iter()
            .find(|(x, _)| x.eq_ignore_ascii_case(input))
        {
            // grandfathered tag
            Ok(tag_from_primary_language(*tag))
        } else if input.starts_with("x-") || input.starts_with("X-") {
            // private use
            if !is_alphanumeric_or_dash(input) {
                Err(ParseError::ForbiddenChar)
            } else if input.len() == 2 {
                Err(ParseError::EmptyPrivateUse)
            } else {
                Ok(tag_from_primary_language(input.to_ascii_lowercase()))
            }
        } else {
            parse_language_tag(input)
        }
    }

    /// Check if the language tag is "valid" according to
    /// [RFC 5646](https://tools.ietf.org/html/rfc5646#section-2.2.9).
    ///
    /// Warning: validation against IANA Language Subtag Registry is not implemented yet
    ///
    /// # Errors
    ///
    /// If the language tag is not "valid" a `ValidationError` variant will be returned.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // The tag is well-formed.
        // always ok

        // Either the tag is in the list of grandfathered tags or all of its
        // primary language, extended language, script, region, and variant
        // subtags appear in the IANA Language Subtag Registry as of the
        // particular registry date.
        // TODO

        // There are no duplicate variant subtags.
        if self.variant_subtags().enumerate().any(|(id1, variant1)| {
            self.variant_subtags()
                .enumerate()
                .any(|(id2, variant2)| id1 != id2 && variant1 == variant2)
        }) {
            return Err(ValidationError::DuplicateVariant);
        }

        // There are no duplicate singleton (extension) subtags.
        if let Some(extension) = self.extension() {
            let mut seen_extensions = AlphanumericLowerCharSet::new();
            if extension.split('-').any(|subtag| {
                if subtag.len() == 1 {
                    let extension = subtag.chars().next().unwrap();
                    if seen_extensions.contains(extension) {
                        true
                    } else {
                        seen_extensions.insert(extension);
                        false
                    }
                } else {
                    false
                }
            }) {
                return Err(ValidationError::DuplicateExtension);
            }
        }

        // There is no more than one extended language subtag.
        // From [errata 5457](https://www.rfc-editor.org/errata/eid5457).
        if let Some(extended_language) = self.extended_language() {
            if extended_language.contains('-') {
                return Err(ValidationError::MultipleExtendedLanguageSubtags);
            }
        }

        Ok(())
    }

    /// Check if the language tag is valid according to
    /// [RFC 5646](https://tools.ietf.org/html/rfc5646#section-2.2.9).
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Returns the canonical version of the language tag.
    ///
    /// It currently applies the following steps:
    ///
    /// * Grandfathered tags are replaced with the canonical version if possible.
    /// * Extension languages are promoted to primary language.
    /// * Deprecated languages are replaced with modern equivalents.
    /// * Deprecated regions are replaced with new country names.
    /// * The `heploc` variant is replaced with `alalc97`.
    ///
    /// The returned language tags may not be completly canonical and they are
    /// not validated.
    ///
    /// Warning: This function does not fully implement yet the canonization algorithm described in
    /// [RFC 5646 4.5](https://tools.ietf.org/html/rfc5646#section-4.5)
    pub fn canonicalize(&self) -> LanguageTag {
        let mut language = self.primary_language();

        // Grandfathered
        if let Some((_, Some(tag))) = GRANDFATHERED.iter().find(|(x, _)| *x == language) {
            return tag_from_primary_language(*tag);
        }

        // Extended language
        if let Some(extended_language) = self.extended_language() {
            language = extended_language;
        }

        //Deprecated language
        if let Some((_, l)) = DEPRECATED_LANGUAGE.iter().find(|(x, _)| *x == language) {
            language = l;
        }

        let mut serialization = String::with_capacity(self.serialization.len());
        serialization.push_str(language);
        let language_end = language.len();
        let extlang_end = language.len();

        // Script
        if let Some(script) = self.script() {
            serialization.push('-');
            serialization.push_str(script);
        }
        let script_end = serialization.len();

        // Region
        if let Some(region) = self.region() {
            serialization.push('-');
            serialization.push_str(
                if let Some(&(_, r)) = DEPRECATED_REGION.iter().find(|&&(x, _)| x == region) {
                    r
                } else {
                    region
                },
            );
        }
        let region_end = serialization.len();

        // Variant
        for variant in self.variant_subtags() {
            serialization.push('-');
            serialization.push_str(if variant == "heploc" {
                "alalc97"
            } else {
                variant
            });
        }
        let variant_end = serialization.len();

        //Extension
        if let Some(extension) = self.extension() {
            serialization.push('-');
            serialization.push_str(extension);
        }
        let extension_end = serialization.len();

        //Private use
        if let Some(private_use) = self.private_use() {
            serialization.push('-');
            serialization.push_str(private_use);
        }

        LanguageTag {
            serialization,
            language_end,
            extlang_end,
            script_end,
            region_end,
            variant_end,
            extension_end,
        }
    }

    /// Matches language tags. The first language acts as a language range, the second one is used
    /// as a normal language tag. None fields in the language range are ignored. If the language
    /// tag has more extlangs than the range these extlangs are ignored. Matches are
    /// case-insensitive.
    ///
    /// For example the range `en-GB` matches only `en-GB` and `en-Arab-GB` but not `en`.
    /// The range `en` matches all language tags starting with `en` including `en`, `en-GB`,
    /// `en-Arab` and `en-Arab-GB`.
    ///
    /// # Panics
    /// If the language range has extensions or private use tags.
    ///
    /// # Examples
    /// ```rust
    /// use language_tags::LanguageTag;
    /// let range_italian = LanguageTag::parse("it").unwrap();
    /// let tag_german = LanguageTag::parse("de").unwrap();
    /// let tag_italian_switzerland = LanguageTag::parse("it-CH").unwrap();
    /// assert!(!range_italian.matches(&tag_german));
    /// assert!(range_italian.matches(&tag_italian_switzerland));
    ///
    /// let range_spanish_brazil = LanguageTag::parse("es-BR").unwrap();
    /// let tag_spanish = LanguageTag::parse("es").unwrap();
    /// assert!(!range_spanish_brazil.matches(&tag_spanish));
    /// ```
    pub fn matches(&self, other: &LanguageTag) -> bool {
        fn matches_option(a: Option<&str>, b: Option<&str>) -> bool {
            match (a, b) {
                (Some(a), Some(b)) => a == b,
                (None, _) => true,
                (_, None) => false,
            }
        }
        fn matches_iter<'a>(
            a: impl Iterator<Item = &'a str>,
            b: impl Iterator<Item = &'a str>,
        ) -> bool {
            a.zip(b).all(|(x, y)| x == y)
        }
        assert!(self.is_language_range());
        self.full_language() == other.full_language()
            && matches_option(self.script(), other.script())
            && matches_option(self.region(), other.region())
            && matches_iter(self.variant_subtags(), other.variant_subtags())
    }

    /// Checks if it is a language range, meaning that there are no extension and privateuse tags.
    pub fn is_language_range(&self) -> bool {
        self.extension().is_none() && self.private_use().is_none()
    }
}

impl std::str::FromStr for LanguageTag {
    type Err = ParseError;

    #[inline]
    fn from_str(input: &str) -> Result<Self, ParseError> {
        Self::parse(input)
    }
}

impl fmt::Display for LanguageTag {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Builds a tag from its primary language
fn tag_from_primary_language(tag: impl Into<String>) -> LanguageTag {
    let serialization = tag.into();
    let end = serialization.len();
    LanguageTag {
        serialization,
        language_end: end,
        extlang_end: end,
        script_end: end,
        region_end: end,
        variant_end: end,
        extension_end: end,
    }
}

/// Handles normal tags.
fn parse_language_tag(input: &str) -> Result<LanguageTag, ParseError> {
    #[derive(PartialEq, Eq)]
    enum State {
        Start,
        AfterLanguage,
        AfterExtLang,
        AfterScript,
        AfterRegion,
        InExtension { expected: bool },
        InPrivateUse { expected: bool },
    }

    let mut serialization = String::with_capacity(input.len());

    let mut state = State::Start;
    let mut language_end = 0;
    let mut extlang_end = 0;
    let mut script_end = 0;
    let mut region_end = 0;
    let mut variant_end = 0;
    let mut extension_end = 0;
    let mut extlangs_count = 0;
    for (subtag, end) in SubTagIterator::new(input) {
        if subtag.is_empty() {
            // All subtags have a maximum length of eight characters.
            return Err(ParseError::EmptySubtag);
        }
        if subtag.len() > 8 {
            // All subtags have a maximum length of eight characters.
            return Err(ParseError::SubtagTooLong);
        }
        if state == State::Start {
            // Primary language
            if subtag.len() < 2 || !is_alphabetic(subtag) {
                return Err(ParseError::InvalidLanguage);
            }
            language_end = end;
            serialization.extend(to_lowercase(subtag));
            if subtag.len() < 4 {
                // extlangs are only allowed for short language tags
                state = State::AfterLanguage;
            } else {
                state = State::AfterExtLang;
            }
        } else if let State::InPrivateUse { .. } = state {
            if !is_alphanumeric(subtag) {
                return Err(ParseError::InvalidSubtag);
            }
            serialization.push('-');
            serialization.extend(to_lowercase(subtag));
            state = State::InPrivateUse { expected: false };
        } else if subtag == "x" || subtag == "X" {
            // We make sure extension is found
            if let State::InExtension { expected: true } = state {
                return Err(ParseError::EmptyExtension);
            }
            serialization.push('-');
            serialization.push('x');
            state = State::InPrivateUse { expected: true };
        } else if subtag.len() == 1 && is_alphanumeric(subtag) {
            // We make sure extension is found
            if let State::InExtension { expected: true } = state {
                return Err(ParseError::EmptyExtension);
            }
            let extension_tag = subtag.chars().next().unwrap().to_ascii_lowercase();
            serialization.push('-');
            serialization.push(extension_tag);
            state = State::InExtension { expected: true };
        } else if let State::InExtension { .. } = state {
            if !is_alphanumeric(subtag) {
                return Err(ParseError::InvalidSubtag);
            }
            extension_end = end;
            serialization.push('-');
            serialization.extend(to_lowercase(subtag));
            state = State::InExtension { expected: false };
        } else if state == State::AfterLanguage && subtag.len() == 3 && is_alphabetic(subtag) {
            extlangs_count += 1;
            if extlangs_count > 3 {
                return Err(ParseError::TooManyExtlangs);
            }
            // valid extlangs
            extlang_end = end;
            serialization.push('-');
            serialization.extend(to_lowercase(subtag));
        } else if (state == State::AfterLanguage || state == State::AfterExtLang)
            && subtag.len() == 4
            && is_alphabetic(subtag)
        {
            // Script
            script_end = end;
            serialization.push('-');
            serialization.extend(to_uppercase_first(subtag));
            state = State::AfterScript;
        } else if (state == State::AfterLanguage
            || state == State::AfterExtLang
            || state == State::AfterScript)
            && (subtag.len() == 2 && is_alphabetic(subtag)
                || subtag.len() == 3 && is_numeric(subtag))
        {
            // Region
            region_end = end;
            serialization.push('-');
            serialization.extend(to_uppercase(subtag));
            state = State::AfterRegion;
        } else if (state == State::AfterLanguage
            || state == State::AfterExtLang
            || state == State::AfterScript
            || state == State::AfterRegion)
            && is_alphanumeric(subtag)
            && (subtag.len() >= 5 && is_alphabetic(&subtag[0..1])
                || subtag.len() >= 4 && is_numeric(&subtag[0..1]))
        {
            // Variant
            variant_end = end;
            serialization.push('-');
            serialization.extend(to_lowercase(subtag));
            state = State::AfterRegion;
        } else {
            return Err(ParseError::InvalidSubtag);
        }
    }

    //We make sure we are in a correct final state
    if let State::InExtension { expected: true } = state {
        return Err(ParseError::EmptyExtension);
    }
    if let State::InPrivateUse { expected: true } = state {
        return Err(ParseError::EmptyPrivateUse);
    }

    //We make sure we have not skipped anyone
    if extlang_end < language_end {
        extlang_end = language_end;
    }
    if script_end < extlang_end {
        script_end = extlang_end;
    }
    if region_end < script_end {
        region_end = script_end;
    }
    if variant_end < region_end {
        variant_end = region_end;
    }
    if extension_end < variant_end {
        extension_end = variant_end;
    }

    Ok(LanguageTag {
        serialization,
        language_end,
        extlang_end,
        script_end,
        region_end,
        variant_end,
        extension_end,
    })
}

struct SubtagListIterator<'a> {
    split: Split<'a, char>,
}

impl<'a> SubtagListIterator<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            split: input.split('-'),
        }
    }
}

impl<'a> Iterator for SubtagListIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let tag = self.split.next()?;
        if tag.is_empty() {
            None
        } else {
            Some(tag)
        }
    }
}

struct ExtensionsIterator<'a> {
    split: Split<'a, char>,
    singleton: Option<char>,
}

impl<'a> ExtensionsIterator<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            split: input.split('-'),
            singleton: None,
        }
    }
}

impl<'a> Iterator for ExtensionsIterator<'a> {
    type Item = (char, &'a str);

    fn next(&mut self) -> Option<(char, &'a str)> {
        let tag = self.split.next()?;
        if tag.is_empty() {
            None
        } else if tag.len() == 1 {
            self.singleton = tag.chars().next();
            self.next()
        } else if let Some(singleton) = self.singleton {
            Some((singleton, tag))
        } else {
            panic!("No singleton found in extension")
        }
    }
}

struct SubTagIterator<'a> {
    split: Split<'a, char>,
    position: usize,
}

impl<'a> SubTagIterator<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            split: input.split('-'),
            position: 0,
        }
    }
}

impl<'a> Iterator for SubTagIterator<'a> {
    type Item = (&'a str, usize);

    fn next(&mut self) -> Option<(&'a str, usize)> {
        let tag = self.split.next()?;
        let tag_end = self.position + tag.len();
        self.position = tag_end + 1;
        Some((tag, tag_end))
    }
}

struct AlphanumericLowerCharSet {
    alphabetic_set: [bool; 26],
    numeric_set: [bool; 10],
}

impl AlphanumericLowerCharSet {
    fn new() -> Self {
        Self {
            alphabetic_set: [false; 26],
            numeric_set: [false; 10],
        }
    }

    fn contains(&mut self, c: char) -> bool {
        if c.is_ascii_digit() {
            self.numeric_set[char_sub(c, '0')]
        } else if c.is_ascii_lowercase() {
            self.alphabetic_set[char_sub(c, 'a')]
        } else if c.is_ascii_uppercase() {
            self.alphabetic_set[char_sub(c, 'A')]
        } else {
            false
        }
    }

    fn insert(&mut self, c: char) {
        if c.is_ascii_digit() {
            self.numeric_set[char_sub(c, '0')] = true
        } else if c.is_ascii_lowercase() {
            self.alphabetic_set[char_sub(c, 'a')] = true
        } else if c.is_ascii_uppercase() {
            self.alphabetic_set[char_sub(c, 'A')] = true
        }
    }
}

fn char_sub(c1: char, c2: char) -> usize {
    (c1 as usize) - (c2 as usize)
}

fn is_alphabetic(s: &str) -> bool {
    s.chars().all(|x| x.is_ascii_alphabetic())
}

fn is_numeric(s: &str) -> bool {
    s.chars().all(|x| x.is_ascii_digit())
}

fn is_alphanumeric(s: &str) -> bool {
    s.chars().all(|x| x.is_ascii_alphanumeric())
}

fn is_alphanumeric_or_dash(s: &str) -> bool {
    s.chars().all(|x| x.is_ascii_alphanumeric() || x == '-')
}

fn to_uppercase<'a>(s: &'a str) -> impl Iterator<Item = char> + 'a {
    s.chars().map(|c| c.to_ascii_uppercase())
}

// Beware: panics if s.len() == 0 (should never happen in our code)
fn to_uppercase_first<'a>(s: &'a str) -> impl Iterator<Item = char> + 'a {
    let mut chars = s.chars();
    once(chars.next().unwrap().to_ascii_uppercase()).chain(chars.map(|c| c.to_ascii_lowercase()))
}

fn to_lowercase<'a>(s: &'a str) -> impl Iterator<Item = char> + 'a {
    s.chars().map(|c| c.to_ascii_lowercase())
}

/// Errors returned by `LanguageTag` parsing
#[derive(Debug, Eq, PartialEq)]
pub enum ParseError {
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
    /// A subtag should not be empty.
    EmptySubtag,
    /// At maximum three extlangs are allowed, but zero to one extlangs are preferred.
    TooManyExtlangs,
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match self {
            ParseError::EmptyExtension => "If an extension subtag is present, it must not be empty",
            ParseError::EmptyPrivateUse => "If the `x` subtag is present, it must not be empty",
            ParseError::ForbiddenChar => "The langtag contains a char not allowed",
            ParseError::InvalidSubtag => {
                "A subtag fails to parse, it does not match any other subtags"
            }
            ParseError::InvalidLanguage => "The given language subtag is invalid",
            ParseError::SubtagTooLong => "A subtag may be eight characters in length at maximum",
            ParseError::EmptySubtag => "A subtag should not be empty",
            ParseError::TooManyExtlangs => "At maximum three extlangs are allowed",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// Errors returned by the `LanguageTag` validation
#[derive(Debug, Eq, PartialEq)]
pub enum ValidationError {
    /// The same variant subtag is only allowed once in a tag.
    DuplicateVariant,
    /// The same extension subtag is only allowed once in a tag before the private use part.
    DuplicateExtension,
    /// only one extended language subtag is allowed
    MultipleExtendedLanguageSubtags,
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        match self {
            ValidationError::DuplicateVariant => {
                "The same variant subtag is only allowed once in a tag"
            }
            ValidationError::DuplicateExtension => {
                "The same extension subtag is only allowed once in a tag"
            }
            ValidationError::MultipleExtendedLanguageSubtags => {
                "only one extended language subtag is allowed"
            }
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}
