#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;

use std::fmt;
use std::str::FromStr;
use std::default::Default;
use std::ascii::AsciiExt;

#[cfg(feature="testdata")]
pub use testdata::*;

#[cfg(not(feature="testdata"))]
pub use data::*;


macro_rules! inspect(
    ($s:expr, $t:expr) => ({
        let t = $t;
        t
    })
);

macro_rules! enoom {
    (pub enum $enum_id:ident; $keywords:ident; $extension:ident; $($variant:ident, $nice:expr, $lower:expr;)*) => (

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum $enum_id {
            $($variant),*,
            $extension(String)
        }

        static $keywords: ::phf::Map<&'static str, $enum_id> = phf_map! {
            $($lower => $enum_id::$variant),*,
        };

        impl ::std::fmt::Display for $enum_id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str(match *self {
                    $($enum_id::$variant => $nice),*,
                    $enum_id::$extension(ref s) => &**s
                })
            }
        }

        impl ::std::str::FromStr for $enum_id {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, <Self as ::std::str::FromStr>::Err> {
                Ok(match $keywords.get(s) {
                    Some(s) => s.clone(),
                    None => $enum_id::$extension(inspect!(stringify!($extension), s).to_string())
                })
            }
        }
    )
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LanguageTag {
    language: Option<Language>,
    extlang: Option<Extlang>,
    script: Option<Script>,
    region: Option<Region>,
    variants: Vec<Variant>
}

impl LanguageTag {
    // Client: en-GB; matches: en-GB
    // Client: en; matches: en-US
    // TODO: Match canonical forms?
    fn matches(&self, other: &LanguageTag) -> bool {
        if self.language.is_some() && self.language != other.language {
            return false;
        } else if self.extlang.is_some() && self.extlang != other.extlang {
            return false;
        }  else if self.script.is_some() && self.script != other.script {
            return false;
        } else if self.region.is_some() && self.region != other.region {
            return false;
        }
        self.variants.iter().all(|v: &Variant| other.variants.iter().any(|o: &Variant| v == o))
    }
}

impl Default for LanguageTag {
    fn default() -> LanguageTag {
        LanguageTag{
            language: None,
            extlang: None,
            script: None,
            region: None,
            variants: vec![]}
    }
}

fn is_number(string: &str) -> bool {
    string.chars().any(|c| c.is_digit(10))
}

impl std::str::FromStr for LanguageTag {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s.is_ascii() {
            true => {
                let mut tag: LanguageTag = Default::default();
                let a = s.to_ascii_lowercase();
                for (i, code) in a.split('-').enumerate() {
                    if i == 0 {
                        tag.language = code.parse().ok();
                    } else if i == 1 && code.len() == 3 && !(is_number(code)) {
                        tag.extlang = code.parse().ok();
                    } else if code.len() == 4 {
                        tag.script = code.parse().ok();
                    } else if (code.len() == 2 && !(is_number(code))) ||
                            (code.len() == 3 && is_number(code)) {
                        tag.region = code.parse().ok();
                    } else if code.len() > 3 {
                        match code.parse::<Variant>().ok() {
                            Some(v) => tag.variants.push(v),
                            None => {},
                            }
                    } else {
                        return Err(());
                    }
                }
                Ok(tag)
            },
            false => Err(()),
        }

    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = vec![];
        match self.language {
            Some(ref language) => parts.push(format!("{}", language)),
            None => {},
        }
        match self.extlang {
            Some(ref extlang) => parts.push(format!("{}", extlang)),
            None => {},
        }
        match self.script {
            Some(ref script) => parts.push(format!("{}", script)),
            None => {},
        }
        match self.region {
            Some(ref region) => parts.push(format!("{}", region)),
            None => {},
        }
        for variant in self.variants.iter() {
            parts.push(format!("{}", variant))
        }
        let last = parts.len() - 1;
        for (i, part) in parts.iter().enumerate() {
            try!(write!(f, "{}", part));
            if i < last {
                try!(write!(f, "-"));
            }
        }
        Ok(())
    }
}

#[cfg(feature="testdata")]
mod testdata;

#[cfg(not(feature="testdata"))]
mod data;

#[cfg(test)]
mod tests {
    use std::default::Default;
    use super::*;
    // All tests here may be completly nonsensical.
    #[test]
    fn test_lang_from_str() {
        let a: LanguageTag = "de".parse().unwrap();
        let mut b: LanguageTag = Default::default();
        b.language = Some(Language::De);
        assert_eq!(a, b);
    }

    #[test]
    fn test_extlang_from_str() {
        let a: LanguageTag = "ar-afb".parse().unwrap();
        let mut b: LanguageTag = Default::default();
        b.language = Some(Language::Ar);
        b.extlang = Some(Extlang::Afb);
        assert_eq!(a, b);
    }

    #[test]
    fn test_script_from_str() {
        let a: LanguageTag = "ar-afb-Latn".parse().unwrap();
        let mut b: LanguageTag = Default::default();
        b.language = Some(Language::Ar);
        b.extlang = Some(Extlang::Afb);
        b.script = Some(Script::Latn);
        assert_eq!(a, b);
    }

    #[test]
    fn test_region_from_str() {
        let a: LanguageTag = "ar-DE".parse().unwrap();
        let mut b: LanguageTag = Default::default();
        b.language = Some(Language::Ar);
        b.region = Some(Region::De);
        assert_eq!(a, b);
    }

    #[test]
    fn test_region_from_str_2() {
        let a: LanguageTag = "ar-005".parse().unwrap();
        let mut b: LanguageTag = Default::default();
        b.language = Some(Language::Ar);
        b.region = Some(Region::R005);
        assert_eq!(a, b);
    }

    #[test]
    fn test_variant_from_str() {
        let a: LanguageTag = "sl-IT-nedis".parse().unwrap();
        let mut b: LanguageTag = Default::default();
        b.language = Some(Language::Sl);
        b.region = Some(Region::It);
        b.variants = vec![Variant::Nedis];
        assert_eq!(a, b);
    }

    #[test]
    fn test_invalid_from_str() {
        assert_eq!("sl-07".parse::<LanguageTag>(), Err(()));
    }

    #[test]
    fn test_strange_case_from_str() {
        // This is a perfectly valid language code
        let a: LanguageTag = "SL-AFB-lATN-005-nEdis".parse().unwrap();
        let mut b: LanguageTag = Default::default();
        b.language = Some(Language::Sl);
        b.extlang = Some(Extlang::Afb);
        b.script = Some(Script::Latn);
        b.region = Some(Region::R005);
        b.variants = vec![Variant::Nedis];
        assert_eq!(a, b);
    }

    #[test]
    fn test_fmt() {
        let a: LanguageTag = "ar-arb-Latn-DE-nedis-foobar".parse().unwrap();
        assert_eq!(format!("{}", a), "ar-arb-Latn-DE-nedis-foobar");
    }

    #[test]
    fn test_match() {
        let de_de: LanguageTag = "de-DE".parse().unwrap();
        let de: LanguageTag = "de".parse().unwrap();
        assert!(de.matches(&de_de));
        assert!(!de_de.matches(&de));
    }
}
