"""Parseur OpenStep minimal + vérifications d'intégrité du project.pbxproj."""
import os, re, sys

class Parser:
    def __init__(self, text):
        self.s = text
        self.i = 0

    def ws(self):
        while self.i < len(self.s):
            if self.s[self.i] in " \t\n\r":
                self.i += 1
            elif self.s.startswith("//", self.i):
                self.i = self.s.find("\n", self.i)
                if self.i == -1: self.i = len(self.s)
            elif self.s.startswith("/*", self.i):
                end = self.s.find("*/", self.i)
                if end == -1: raise ValueError("commentaire non fermé")
                self.i = end + 2
            else:
                return

    def value(self):
        self.ws()
        c = self.s[self.i]
        if c == "{": return self.dict()
        if c == "(": return self.array()
        if c == '"': return self.quoted()
        return self.bare()

    def dict(self):
        assert self.s[self.i] == "{"; self.i += 1
        out = {}
        while True:
            self.ws()
            if self.s[self.i] == "}":
                self.i += 1
                return out
            key = self.value()
            self.ws()
            if self.s[self.i] != "=":
                raise ValueError(f"'=' attendu après {key!r} en {self.i}")
            self.i += 1
            out[key] = self.value()
            self.ws()
            if self.s[self.i] != ";":
                raise ValueError(f"';' attendu après {key!r} en {self.i}")
            self.i += 1

    def array(self):
        assert self.s[self.i] == "("; self.i += 1
        out = []
        while True:
            self.ws()
            if self.s[self.i] == ")":
                self.i += 1
                return out
            out.append(self.value())
            self.ws()
            if self.s[self.i] == ",":
                self.i += 1

    def quoted(self):
        self.i += 1
        out = []
        while self.s[self.i] != '"':
            if self.s[self.i] == "\\":
                self.i += 1
                out.append(self.s[self.i])
            else:
                out.append(self.s[self.i])
            self.i += 1
        self.i += 1
        return "".join(out)

    def bare(self):
        start = self.i
        while self.s[self.i] not in ' \t\n\r;,=(){}"':
            self.i += 1
        if start == self.i:
            raise ValueError(f"jeton vide en {self.i}: {self.s[self.i-30:self.i+30]!r}")
        return self.s[start:self.i]

path = sys.argv[1]
text = open(path).read()
if not text.startswith("// !$*UTF8*$!"):
    sys.exit("en-tête UTF8 manquant")

root = Parser(text[text.index("{"):]).value()
objects = root["objects"]
print(f"plist analysé : {len(objects)} objets, rootObject={root['rootObject']}")

errors = []

# 1. Toute référence d'identifiant pointe sur un objet défini.
def walk(node, where):
    if isinstance(node, dict):
        for k, v in node.items():
            walk(v, f"{where}.{k}")
    elif isinstance(node, list):
        for idx, v in enumerate(node):
            walk(v, f"{where}[{idx}]")
    elif isinstance(node, str) and re.fullmatch(r"[0-9A-F]{24}", node):
        if node not in objects:
            errors.append(f"référence orpheline {node} dans {where}")

for oid, obj in objects.items():
    walk(obj, f"{objects[oid].get('isa','?')}({oid})")
walk(root["rootObject"], "rootObject")

# 2. Le projet, ses cibles, ses phases sont cohérents.
project = objects[root["rootObject"]]
if project["isa"] != "PBXProject":
    errors.append("rootObject n'est pas un PBXProject")

isa_count = {}
for obj in objects.values():
    isa_count[obj["isa"]] = isa_count.get(obj["isa"], 0) + 1
print("sections :", ", ".join(f"{k}={v}" for k, v in sorted(isa_count.items())))

# 3. Chaque PBXFileReference correspond à un fichier réel sur le disque.
root_dir = os.path.dirname(os.path.dirname(os.path.abspath(path)))

def resolve(group_id, prefix):
    group = objects[group_id]
    path_component = group.get("path")
    here = os.path.join(prefix, path_component) if path_component else prefix
    for child in group.get("children", []):
        obj = objects[child]
        if obj["isa"] == "PBXGroup":
            resolve(child, here)
        elif obj["isa"] == "PBXFileReference":
            if obj.get("sourceTree") == "BUILT_PRODUCTS_DIR":
                continue
            full = os.path.join(root_dir, here, obj["path"])
            if not os.path.exists(full):
                errors.append(f"fichier absent du disque : {os.path.relpath(full, root_dir)}")
            seen_files.add(os.path.relpath(full, root_dir))

seen_files = set()
resolve(project["mainGroup"], "")

# 4. Tout fichier source du disque est bien référencé par le projet.
on_disk = set()
for base, dirs, files in os.walk(root_dir):
    dirs[:] = [d for d in dirs if not d.endswith((".xcodeproj", ".xcassets"))]
    for name in files:
        if name.endswith(".swift"):
            on_disk.add(os.path.relpath(os.path.join(base, name), root_dir))
missing = on_disk - seen_files
for name in sorted(missing):
    errors.append(f"source non référencée dans le projet : {name}")

# 5. Chaque source référencée est compilée par une phase Sources.
compiled = set()
for obj in objects.values():
    if obj["isa"] == "PBXSourcesBuildPhase":
        for bf in obj["files"]:
            compiled.add(objects[bf]["fileRef"])
swift_refs = {oid for oid, obj in objects.items()
              if obj["isa"] == "PBXFileReference" and obj.get("path", "").endswith(".swift")}
for oid in sorted(swift_refs - compiled):
    errors.append(f"source jamais compilée : {objects[oid]['path']}")

# 6. Les cibles ont bien leurs listes de configuration, leurs produits et leurs sources.
products_by_target = {}
for target_id in project["targets"]:
    target = objects[target_id]
    name = target["name"]
    products_by_target[name] = objects[target["productReference"]]["path"]
    configs = objects[target["buildConfigurationList"]]["buildConfigurations"]
    names = sorted(objects[c]["name"] for c in configs)
    if names != ["Debug", "Release"]:
        errors.append(f"{name} : configurations {names}")

    phases = [objects[p] for p in target["buildPhases"]]
    kinds = [p["isa"] for p in phases]
    if "PBXSourcesBuildPhase" not in kinds:
        errors.append(f"{name} : pas de phase Sources")

    sources = [p for p in phases if p["isa"] == "PBXSourcesBuildPhase"]
    count = sum(len(p["files"]) for p in sources)
    if count == 0:
        errors.append(f"{name} : aucune source compilée")

    settings = objects[configs[0]]["buildSettings"]
    labels = [f"{len(p['files'])} fichiers" for p in sources]
    print(f"cible {name} ({target['productType'].split('.')[-1]})"
          f" → {objects[target['productReference']]['path']},"
          f" {count} sources, sdk {settings.get('SDKROOT', 'hérité')},"
          f" id {settings.get('PRODUCT_BUNDLE_IDENTIFIER', '?')}")

# 7. Les phases d'intégration pointent sur les bons produits, au bon endroit.
EXPECTED_EMBEDS = {
    ("13", "KliimaWidgets.appex"),
    ("16", "KliimaWatch.app"),
    ("13", "KliimaWatchWidgets.appex"),
}
seen_embeds = set()
for obj in objects.values():
    if obj["isa"] != "PBXCopyFilesBuildPhase":
        continue
    label = obj.get("name", "")
    embedded = [objects[objects[f]["fileRef"]]["path"] for f in obj["files"]]
    spec = str(obj.get("dstSubfolderSpec"))
    if len(embedded) != 1:
        errors.append(f"{label} : intègre {embedded}")
        continue
    pair = (spec, embedded[0])
    seen_embeds.add(pair)
    if pair not in EXPECTED_EMBEDS:
        errors.append(f"intégration inattendue : {label} → {embedded[0]} (dossier {spec})")
    print(f"intégration {label} → {embedded[0]} (dossier {spec})")

for pair in EXPECTED_EMBEDS - seen_embeds:
    errors.append(f"intégration absente : {pair[1]}")

# 8. Les dépendances de l'application couvrent ce qu'elle embarque.
app = next(objects[t] for t in project["targets"] if objects[t]["name"] == "Kliima")
for host, needed in (("Kliima", ("KliimaWidgets", "KliimaWatch")),
                     ("KliimaWatch", ("KliimaWatchWidgets",))):
    target = next(objects[t] for t in project["targets"] if objects[t]["name"] == host)
    deps = {objects[objects[d]["target"]]["name"] for d in target["dependencies"]}
    for name in needed:
        if name not in deps:
            errors.append(f"{host} ne dépend pas de {name}")
    print(f"dépendances de {host} :", ", ".join(sorted(deps)))

if errors:
    print("\nERREURS :")
    for e in errors:
        print(" -", e)
    sys.exit(1)
print("\nproject.pbxproj valide.")
