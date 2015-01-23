use std::fmt;
use std::str::FromStr;
use std::default::Default;
use std::ascii::AsciiExt;


macro_rules! inspect(
    ($s:expr, $t:expr) => ({
        let t = $t;
        t
    })
);

macro_rules! enoom {
    (pub enum $en:ident; $ext:ident; $($ty:ident, $text:expr;)*) => (

        #[derive(Clone, Show)]
        pub enum $en {
            $($ty),*,
            $ext(String)
        }

        impl PartialEq for $en {
            fn eq(&self, other: &$en) -> bool {
                match (self, other) {
                    $( (&$en::$ty, &$en::$ty) => true ),*,
                    (&$en::$ext(ref a), &$en::$ext(ref b)) => a == b,
                    _ => self.to_string() == other.to_string()
                }
            }
        }

        impl fmt::String for $en {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(match *self {
                    $($en::$ty => $text),*,
                    $en::$ext(ref s) => &**s
                })
            }
        }

        impl FromStr for $en {
            fn from_str(s: &str) -> Option<$en> {
                Some(match s {
                    $(_s if _s == $text.to_ascii_lowercase() => $en::$ty),*,
                    s => $en::$ext(inspect!(stringify!($ext), s).to_string())
                })
            }
        }
    )
}

enoom! {
    pub enum Language;
    Ext;
    Ar, "ar";
    De, "de";
    En, "en";
    Fr, "fr";
    Sl, "sl";
}

enoom! {
    pub enum Extlang;
    Ext;
    Yue, "yue";
    Afb, "afb";
}

enoom! {
    pub enum Script;
    Ext;
    Adlm, "Adlm";
    Afak, "Afak";
    Aghb, "Aghb";
    Ahom, "Ahom";
    Arab, "Arab";
    Aran, "Aran";
    Armi, "Armi";
    Armn, "Armn";
    Avst, "Avst";
    Bali, "Bali";
    Bamu, "Bamu";
    Bass, "Bass";
    Batk, "Batk";
    Beng, "Beng";
    Blis, "Blis";
    Bopo, "Bopo";
    Brah, "Brah";
    Brai, "Brai";
    Bugi, "Bugi";
    Buhd, "Buhd";
    Cakm, "Cakm";
    Cans, "Cans";
    Cari, "Cari";
    Cham, "Cham";
    Cher, "Cher";
    Cirt, "Cirt";
    Copt, "Copt";
    Cprt, "Cprt";
    Cyrl, "Cyrl";
    Cyrs, "Cyrs";
    Deva, "Deva";
    Dsrt, "Dsrt";
    Dupl, "Dupl";
    Egyd, "Egyd";
    Egyh, "Egyh";
    Egyp, "Egyp";
    Elba, "Elba";
    Ethi, "Ethi";
    Geok, "Geok";
    Geor, "Geor";
    Glag, "Glag";
    Goth, "Goth";
    Gran, "Gran";
    Grek, "Grek";
    Gujr, "Gujr";
    Guru, "Guru";
    Hang, "Hang";
    Hani, "Hani";
    Hano, "Hano";
    Hans, "Hans";
    Hant, "Hant";
    Hatr, "Hatr";
    Hebr, "Hebr";
    Hira, "Hira";
    Hluw, "Hluw";
    Hmng, "Hmng";
    Hrkt, "Hrkt";
    Hung, "Hung";
    Inds, "Inds";
    Ital, "Ital";
    Java, "Java";
    Jpan, "Jpan";
    Jurc, "Jurc";
    Kali, "Kali";
    Kana, "Kana";
    Khar, "Khar";
    Khmr, "Khmr";
    Khoj, "Khoj";
    Kitl, "Kitl";
    Kits, "Kits";
    Knda, "Knda";
    Kore, "Kore";
    Kpel, "Kpel";
    Kthi, "Kthi";
    Lana, "Lana";
    Laoo, "Laoo";
    Latf, "Latf";
    Latg, "Latg";
    Latn, "Latn";
    Lepc, "Lepc";
    Limb, "Limb";
    Lina, "Lina";
    Linb, "Linb";
    Lisu, "Lisu";
    Loma, "Loma";
    Lyci, "Lyci";
    Lydi, "Lydi";
    Mahj, "Mahj";
    Mand, "Mand";
    Mani, "Mani";
    Marc, "Marc";
    Maya, "Maya";
    Mend, "Mend";
    Merc, "Merc";
    Mero, "Mero";
    Mlym, "Mlym";
    Modi, "Modi";
    Mong, "Mong";
    Moon, "Moon";
    Mroo, "Mroo";
    Mtei, "Mtei";
    Mult, "Mult";
    Mymr, "Mymr";
    Narb, "Narb";
    Nbat, "Nbat";
    Nkgb, "Nkgb";
    Nkoo, "Nkoo";
    Nshu, "Nshu";
    Ogam, "Ogam";
    Olck, "Olck";
    Orkh, "Orkh";
    Orya, "Orya";
    Osge, "Osge";
    Osma, "Osma";
    Palm, "Palm";
    Pauc, "Pauc";
    Perm, "Perm";
    Phag, "Phag";
    Phli, "Phli";
    Phlp, "Phlp";
    Phlv, "Phlv";
    Phnx, "Phnx";
    Plrd, "Plrd";
    Prti, "Prti";
    Rjng, "Rjng";
    Roro, "Roro";
    Runr, "Runr";
    Samr, "Samr";
    Sara, "Sara";
    Sarb, "Sarb";
    Saur, "Saur";
    Sgnw, "Sgnw";
    Shaw, "Shaw";
    Shrd, "Shrd";
    Sidd, "Sidd";
    Sind, "Sind";
    Sinh, "Sinh";
    Sora, "Sora";
    Sund, "Sund";
    Sylo, "Sylo";
    Syrc, "Syrc";
    Syre, "Syre";
    Syrj, "Syrj";
    Syrn, "Syrn";
    Tagb, "Tagb";
    Takr, "Takr";
    Tale, "Tale";
    Talu, "Talu";
    Taml, "Taml";
    Tang, "Tang";
    Tavt, "Tavt";
    Telu, "Telu";
    Teng, "Teng";
    Tfng, "Tfng";
    Tglg, "Tglg";
    Thaa, "Thaa";
    Thai, "Thai";
    Tibt, "Tibt";
    Tirh, "Tirh";
    Ugar, "Ugar";
    Vaii, "Vaii";
    Visp, "Visp";
    Wara, "Wara";
    Wole, "Wole";
    Xpeo, "Xpeo";
    Xsux, "Xsux";
    Yiii, "Yiii";
    Zinh, "Zinh";
    Zmth, "Zmth";
    Zsym, "Zsym";
    Zxxx, "Zxxx";
    Zyyy, "Zyyy";
    Zzzz, "Zzzz";
}

enoom! {
    pub enum Region;
    Ext;
    De, "DE";
    Fr, "FR";
    It, "IT";
    R005, "005";
}

enoom! {
    pub enum Variant;
    Ext;
    Nedis, "nedis";
}

#[derive(Show, PartialEq, Clone)]
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
    fn from_str(s: &str) -> Option<Self> {
        match s.is_ascii() {
            true => {
                let mut tag: LanguageTag = Default::default();
                let a = s.to_ascii_lowercase();
                for (i, code) in a.split('-').enumerate() {
                    if i == 0 {
                        tag.language = code.parse();
                    } else if i == 1 && code.len() == 3 && !(is_number(code)) {
                        tag.extlang = code.parse();
                    } else if code.len() == 4 {
                        tag.script = code.parse();
                    } else if (code.len() == 2 && !(is_number(code))) ||
                            (code.len() == 3 && is_number(code)) {
                        tag.region = code.parse();
                    } else if code.len() > 3 {
                        match code.parse::<Variant>() {
                            Some(v) => tag.variants.push(v),
                            None => {},
                            }
                    } else {
                        return None;
                    }
                }
                Some(tag)
            },
            false => None,
        }

    }
}

impl fmt::String for LanguageTag {
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
        assert_eq!("sl-07".parse::<LanguageTag>(), None);
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
