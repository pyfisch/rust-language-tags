import json


with open('registry.json') as f:
    registry = json.load(f)

languages = [v["Subtag"] for v in registry if v["Type"] == "language" and len(v["Subtag"]) < 4]
languages.sort()

print("""use std::fmt;
use std::str::FromStr;
use std::ascii::AsciiExt;""")
print("")

print("""
#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum Language {""")
for v in languages:
    print("    " + v.title() + ",")
print("""    Extension(String),
}""")

print("""type NamedVariants = &'static [(&'static str, Language)];

static LANGUAGE_VARIANTS: NamedVariants = &[""")
for v in languages:
    print('    ("' + v + '", Language::' + v.title() + '),')
print("""];""")

print("""
fn find_variant(variants: NamedVariants, name: &str) -> Option<Language> {
    match variants.binary_search_by(|&(s, _)| s.cmp(name)) {
        Ok(i) => Some(variants[i].1.clone()),
        Err(_) => None,
    }
}

fn find_string(variants: NamedVariants, variant: &Language) -> Option<String> {
    match variants.binary_search_by(|&(_, ref v)| v.partial_cmp(variant).unwrap()) {
        Ok(i) => Some(variants[i].0.to_string()),
        Err(_) => None,
    }
}

impl FromStr for Language {
    fn from_str(mut s: &str) -> Option<Language> {
        let slice = &s.to_ascii_lowercase()[];
        Some(match find_variant(LANGUAGE_VARIANTS, slice) {
            Some(lang) => lang,
            None => Language::Extension(slice.to_string()),
        })
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match find_string(LANGUAGE_VARIANTS, self) {
            Some(ref string) => string.as_slice(),
            None => match *self {
                Language::Extension(ref string) => &**string,
                _ => panic!("WTF"),
            },
        })
    }
}""")
