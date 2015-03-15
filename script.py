import json

with open('registry.json') as f:
    registry = json.load(f)

def process_group(name):
    items = [v["Subtag"] for v in registry if v["Type"] == name and (len(v["Subtag"]) < 5 or name == "variant")]
    items.sort()
    with open("src/data/" + name + ".rs", "w") as o:
        o.write("enoom! {\n")
        o.write("    pub enum " + name.title() + ";\n")
        o.write("    " + name.upper() + "_KEYWORDS;\n")
        o.write("    Unregistered;\n")
        for item in items:
            if not item[0].isdigit():
                o.write("    " + item.title() + ', "' + item + '", "' + item.lower() + '";\n')
            elif name == "region":
                o.write("    R" + item.title() + ', "' + item + '", "' + item.lower() + '";\n')
            else:
                o.write("    V" + item.title() + ', "' + item + '", "' + item.lower() + '";\n')
        o.write("}\n")


process_group("extlang")
process_group("language")
process_group("region")
process_group("script")
process_group("variant")
