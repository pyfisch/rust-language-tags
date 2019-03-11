import os
import re
import urllib.request
from collections import defaultdict
from typing import Dict, Iterable, List

registry_url = 'https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry'

struct_definitions = """
pub struct LanguageRecord {
    pub subtag: &'static str,
    pub preferred_value: Option<&'static str>,
    pub suppress_script: Option<&'static str>,
    pub macrolanguage: Option<&'static str>
}

pub struct ExtlangRecord {
    pub subtag: &'static str,
    pub preferred_value: &'static str,
    pub prefix: &'static str
}

pub struct ScriptRecord {
    pub subtag: &'static str,
    pub preferred_value: Option<&'static str>
}

pub struct RegionRecord {
    pub subtag: &'static str,
    pub preferred_value: Option<&'static str>
}

pub struct VariantRecord {
    pub subtag: &'static str,
    pub preferred_value: Option<&'static str>,
    pub prefixes: &'static str
}

pub struct GrandfatheredRecord {
    pub tag: &'static str,
    pub preferred_value: Option<&'static str>,
}

pub struct RedundantRecord {
    pub tag: &'static str,
    pub preferred_value: Option<&'static str>,
}


"""


def parse_registry(url: str) -> Iterable[Dict[str, List[str]]]:
    """
    Parses the registry content from url and returns records as dict
    """
    field_regex = re.compile('^([a-zA-Z0-9-]+) *: *(.*)$')
    current_record = defaultdict(list)
    current_field = None
    for line in urllib.request.urlopen(url):
        line = line.decode().strip()
        if line == '%%':
            yield current_record
            current_record = defaultdict(list)
            continue

        match = field_regex.match(line)
        if match:
            current_field = match.group(1)
            current_record[current_field].append(match.group(2))
        else:
            current_record[current_field][-1] += ' ' + line
    if current_record:
        yield current_record


def serialize_mandatory(l: List[str]):
    if len(l) == 1:
        return '"{}"'.format(l[0])
    else:
        raise ValueError('multiple values: {}'.format(l))


def serialize_option(l: List[str]):
    if len(l) > 1:
        raise ValueError('multiple values: {}'.format(l))
    elif len(l) == 1:
        return 'Some("{}")'.format(l[0])
    else:
        return 'None'


def serialize_struct(name: str, values: Dict[str, str]):
    return '{} {{\n{}\n}}'.format(name, '\n'.join('{}: {},'.format(k, v) for k, v in values.items()))


def serialize_static_array(name, elements: List[str]):
    return 'pub const {}: [{}; {}] = [\n{}\n];\n\n'.format(name, elements[0].split(' ')[0], len(elements), ',\n'.join(elements))


file_date = None
values = defaultdict(list)

for record in parse_registry(registry_url):
    if 'Type' not in record:
        if 'File-Date' in record:
            file_date = record['File-Date'][0]
        else:
            print('Unexpected record: {}'.format(record))
    elif record['Type'][0] == 'language':
        values['LANGUAGES'].append(serialize_struct('LanguageRecord', {
            'subtag': serialize_mandatory(record['Subtag']),
            'preferred_value': serialize_option(record['Preferred-Value']),
            'suppress_script': serialize_option(record['Suppress-Script']),
            'macrolanguage': serialize_option(record['Macrolanguage'])
        }))
    elif record['Type'][0] == 'extlang':
        values['EXTLANGS'].append(serialize_struct('ExtlangRecord', {
            'subtag': serialize_mandatory(record['Subtag']),
            'preferred_value': serialize_mandatory(record['Preferred-Value']),
            'prefix': serialize_mandatory(record['Prefix'])[:-1] + '-"'
        }))
    elif record['Type'][0] == 'script':
        values['SCRIPTS'].append(serialize_struct('ScriptRecord', {
            'subtag': serialize_mandatory(record['Subtag']),
            'preferred_value': serialize_option(record['Preferred-Value'])
        }))
    elif record['Type'][0] == 'region':
        values['REGIONS'].append(serialize_struct('RegionRecord', {
            'subtag': serialize_mandatory(record['Subtag']),
            'preferred_value': serialize_option(record['Preferred-Value'])
        }))
    elif record['Type'][0] == 'variant':
        values['VARIANTS'].append(serialize_struct('VariantRecord', {
            'subtag': serialize_mandatory(record['Subtag']),
            'preferred_value': serialize_option(record['Preferred-Value']),
            'prefixes': '"{}"'.format(' '.join(p + '-' for p in record['Prefix'] if p))
        }))
    elif record['Type'][0] == 'grandfathered':
        values['GRANDFATHEREDS'].append(serialize_struct('GrandfatheredRecord', {
            'tag': serialize_mandatory(record['Tag']),
            'preferred_value': serialize_option(record['Preferred-Value'])
        }))
    elif record['Type'][0] == 'redundant':
        values['REDUNDANTS'].append(serialize_struct('RedundantRecord', {
            'tag': serialize_mandatory(record['Tag']),
            'preferred_value': serialize_option(record['Preferred-Value'])
        }))
    else:
        print('Unexpected record: {}'.format(record))

target_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), '../src/iana_registry.rs')
with open(target_path, 'wt') as fp:
    fp.write(struct_definitions)
    for k, v in values.items():
        v.sort()
        fp.write(serialize_static_array(k, v))
