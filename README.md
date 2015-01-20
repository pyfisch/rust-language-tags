# rust-language-tags
Typesafe  [BCP47](http://tools.ietf.org/html/bcp47) language tag handling.

See also http://www.w3.org/International/articles/language-tags/

## Features
* Parsing and formatting tags ☑
* Parts
 * Language ☑
 * Extlang ☑
 * Script ☑
 * Variant ☑
 * Region ☑
 * Extension ☐
 * Privateuse ☐
* Comparing tags ☐
* IANA registered tags ☐ (currently only example data)
* Check if tag is valid ☐
* much more... ☐


## How to use it
Create tags:
```rust
use std::default::Default;
let mut tag: LanguageTag = Default::default();
tag.language = Some(Language::De);
tag.region = Some(Region::De);
println!("{}", tag) // "de-DE", German as spoken in Germany
```

Parse tags:
```rust
// Parse Arabic written in Latin script
let tag: LanguageTag = "ar-Latn".parse().unwrap();
println!("{}", tag.script.unwrap()); // "Latn"
```
