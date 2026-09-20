"""Génère le project.pbxproj de Kliima : application iOS, extension widget
(activité en direct), application watchOS et cible de tests."""
import hashlib, os, sys

def oid(key):
    """Identifiant pbxproj : 24 caractères hexadécimaux, stables entre générations."""
    return hashlib.md5(key.encode()).hexdigest()[:24].upper()

APP, WIDGETS, WATCH, WATCH_WIDGETS, TESTS = (
    "Kliima", "KliimaWidgets", "KliimaWatch", "KliimaWatchWidgets", "KliimaTests")
ALL_TARGETS = (APP, WIDGETS, WATCH, WATCH_WIDGETS, TESTS)

# (hôte, embarqué, nom de phase, dstPath, dstSubfolderSpec)
EMBEDS = [
    (APP, WIDGETS, "Embed Foundation Extensions", '""', 13),
    (APP, WATCH, "Embed Watch Content", '"$(CONTENTS_FOLDER_PATH)/Watch"', 16),
    (WATCH, WATCH_WIDGETS, "Embed Foundation Extensions", '""', 13),
]

# Fichiers, par groupe de rangement. Un même fichier peut alimenter plusieurs cibles.
MODELS = ["AgroSamples.swift", "AgroIndicators.swift", "WeatherCondition.swift",
          "AgroFormat.swift", "Localized.swift", "SharedStore.swift", "Consensus.swift",
          "SprayActivityAttributes.swift", "Plan.swift", "Alerts.swift", "Cumuls.swift",
          "Register.swift"]
SERVICES = ["AgroWeatherService.swift", "WeatherProviders.swift", "LocationService.swift",
            "SprayActivityController.swift", "BackgroundRefresh.swift", "Subscription.swift",
            "AlertScheduler.swift"]
VIEWMODELS = ["DashboardViewModel.swift"]
VIEWS = ["DashboardView.swift", "DailyListView.swift", "DetailTile.swift", "HeroView.swift",
         "HourlyStripView.swift", "SkyBackground.swift", "SprayCardView.swift",
         "PaywallView.swift"]

# (groupe, nom) ; le groupe est le chemin relatif au dépôt.
def app(names, sub): return [(f"{APP}/{sub}", n) for n in names]

APP_FILES = {
    "App": app(["KliimaApp.swift"], "App"),
    "Models": app(MODELS, "Models"),
    "Services": app(SERVICES, "Services"),
    "ViewModels": app(VIEWMODELS, "ViewModels"),
    "Views": app(VIEWS, "Views"),
}
ASSETS = (f"{APP}/Resources", "Assets.xcassets")
STRINGS = (f"{APP}/Resources", "Localizable.xcstrings")
INFO_STRINGS = (f"{APP}/Resources", "InfoPlist.xcstrings")
APP_PLIST = (f"{APP}/Resources", "Info.plist")

WIDGET_FILES = [(WIDGETS, "KliimaWidgetsBundle.swift"), (WIDGETS, "SprayLiveActivity.swift"),
                (WIDGETS, "SprayWidget.swift")]
WIDGET_PLIST = (WIDGETS, "Info.plist")
WIDGET_ENTITLEMENTS = (WIDGETS, "KliimaWidgets.entitlements")
APP_ENTITLEMENTS = (APP, "Kliima.entitlements")

WATCH_WIDGET_FILES = [(WATCH_WIDGETS, "KliimaWatchWidgetsBundle.swift"),
                      (WATCH_WIDGETS, "SprayComplication.swift")]
WATCH_WIDGET_PLIST = (WATCH_WIDGETS, "Info.plist")
WATCH_WIDGET_ENTITLEMENTS = (WATCH_WIDGETS, "KliimaWatchWidgets.entitlements")
WATCH_ENTITLEMENTS = (WATCH, "KliimaWatch.entitlements")

WATCH_FILES = [(WATCH, "KliimaWatchApp.swift"), (WATCH, "WatchDashboardView.swift"),
               (WATCH, "WatchViewModel.swift")]
WATCH_ASSETS = (f"{WATCH}/Resources", "Assets.xcassets")
WATCH_PLIST = (f"{WATCH}/Resources", "Info.plist")

TEST_FILES = [(TESTS, n) for n in ["AgroIndicatorsTests.swift", "AgroWeatherDecodingTests.swift",
                                   "WeatherConditionTests.swift", "AgroFormatTests.swift",
                                   "LocalizationTests.swift", "SprayActivityTests.swift",
                                   "ConsensusTests.swift", "ProvidersTests.swift",
                                   "PlanTests.swift", "AlertsTests.swift",
                                   "CumulsTests.swift", "RegisterTests.swift"]]

# Sources compilées par chaque cible. Le noyau (Models) est partagé ; la montre
# ajoute le réseau et la position, le widget se limite à ce qu'il affiche.
def models(*names): return [(f"{APP}/Models", n) for n in names]
def services(*names): return [(f"{APP}/Services", n) for n in names]

WATCH_MODELS = [n for n in MODELS if n != "SprayActivityAttributes.swift"]

TARGET_SOURCES = {
    APP: sum(APP_FILES.values(), []),
    # Le widget affiche l'activité en direct et va chercher sa propre prévision.
    WIDGETS: models(*MODELS) + services("AgroWeatherService.swift", "WeatherProviders.swift")
             + WIDGET_FILES,
    WATCH: models(*WATCH_MODELS)
           + services("AgroWeatherService.swift", "WeatherProviders.swift", "LocationService.swift")
           + WATCH_FILES,
    WATCH_WIDGETS: models(*WATCH_MODELS)
                   + services("AgroWeatherService.swift", "WeatherProviders.swift")
                   + WATCH_WIDGET_FILES,
    TESTS: TEST_FILES,
}
TARGET_RESOURCES = {
    APP: [ASSETS, STRINGS, INFO_STRINGS],
    WIDGETS: [STRINGS],
    WATCH: [WATCH_ASSETS, STRINGS, INFO_STRINGS],
    WATCH_WIDGETS: [STRINGS],
    TESTS: [],
}

ids = {k: oid(k) for k in [
    "project", "group:main", "group:products", "group:app", "group:widgets", "group:watch",
    "group:watchwidgets", "group:tests", "configlist:project",
]}
for target in ALL_TARGETS:
    for kind in ("target", "product", "configlist", "phase:sources", "phase:frameworks",
                 "phase:resources", "config:Debug", "config:Release"):
        ids[f"{kind}:{target}"] = oid(f"{kind}:{target}")
for host, embedded, *_ in EMBEDS:
    ids[f"proxy:{embedded}"] = oid(f"proxy:{embedded}")
    ids[f"dependency:{embedded}"] = oid(f"dependency:{embedded}")
    ids[f"embed:{host}:{embedded}"] = oid(f"embed:{host}:{embedded}")
    ids[f"embedfile:{host}:{embedded}"] = oid(f"embedfile:{host}:{embedded}")
for group in ("App", "Models", "Services", "ViewModels", "Views", "Resources"):
    ids[f"group:app:{group}"] = oid(f"group:app:{group}")
ids["group:watch:Resources"] = oid("group:watch:Resources")

def fref(entry): return oid(f"fileref:{entry[0]}/{entry[1]}")
def bfile(target, entry): return oid(f"buildfile:{target}:{entry[0]}/{entry[1]}")

out = []
w = out.append
w("// !$*UTF8*$!")
w("{")
w("\tarchiveVersion = 1;")
w("\tclasses = {")
w("\t};")
w("\tobjectVersion = 56;")
w("\tobjects = {")
w("")

# ---- PBXBuildFile
w("/* Begin PBXBuildFile section */")
rows = []
for target, entries in TARGET_SOURCES.items():
    rows += [(bfile(target, e), e[1], "Sources", fref(e), None) for e in entries]
for target, entries in TARGET_RESOURCES.items():
    rows += [(bfile(target, e), e[1], "Resources", fref(e), None) for e in entries]
for host, embedded, phase_name, _, _ in EMBEDS:
    product = f"{embedded}.appex" if embedded in (WIDGETS, WATCH_WIDGETS) else f"{embedded}.app"
    rows.append((ids[f"embedfile:{host}:{embedded}"], product, phase_name,
                 ids[f"product:{embedded}"], "settings = {ATTRIBUTES = (RemoveHeadersOnCopy, ); }; "))
for bid, name, phase, ref, extra in sorted(rows):
    w(f"\t\t{bid} /* {name} in {phase} */ = {{isa = PBXBuildFile; fileRef = {ref} /* {name} */; "
      + (extra or "") + "};")
w("/* End PBXBuildFile section */")
w("")

# ---- PBXContainerItemProxy
w("/* Begin PBXContainerItemProxy section */")
ids[f"proxy:{TESTS}"] = oid(f"proxy:{TESTS}")
proxies = [(f"proxy:{e}", ids[f"target:{e}"], e) for _, e, *_ in EMBEDS]
proxies.append((f"proxy:{TESTS}", ids[f"target:{APP}"], APP))
for key, remote, info in proxies:
    w(f"\t\t{ids[key]} /* PBXContainerItemProxy */ = {{")
    w("\t\t\tisa = PBXContainerItemProxy;")
    w(f"\t\t\tcontainerPortal = {ids['project']} /* Project object */;")
    w("\t\t\tproxyType = 1;")
    w(f"\t\t\tremoteGlobalIDString = {remote};")
    w(f"\t\t\tremoteInfo = {info};")
    w("\t\t};")
w("/* End PBXContainerItemProxy section */")
w("")

# ---- PBXFileReference
w("/* Begin PBXFileReference section */")
refs = []
seen = set()
def add_ref(entry, kind):
    if entry in seen:
        return
    seen.add(entry)
    refs.append((fref(entry), entry[1],
                 f'isa = PBXFileReference; lastKnownFileType = {kind}; path = {entry[1]}; sourceTree = "<group>";'))

for entries in TARGET_SOURCES.values():
    for entry in entries:
        add_ref(entry, "sourcecode.swift")
for entry in (ASSETS, WATCH_ASSETS):
    add_ref(entry, "folder.assetcatalog")
for entry in (STRINGS, INFO_STRINGS):
    add_ref(entry, "text.json.xcstrings")
for entry in (APP_PLIST, WIDGET_PLIST, WATCH_PLIST, WATCH_WIDGET_PLIST):
    add_ref(entry, "text.plist.xml")
for entry in (APP_ENTITLEMENTS, WIDGET_ENTITLEMENTS, WATCH_ENTITLEMENTS, WATCH_WIDGET_ENTITLEMENTS):
    add_ref(entry, "text.plist.entitlements")

products = [
    (ids[f"product:{APP}"], f"{APP}.app", "explicitFileType = wrapper.application"),
    (ids[f"product:{WIDGETS}"], f"{WIDGETS}.appex", "explicitFileType = wrapper.app-extension"),
    (ids[f"product:{WATCH}"], f"{WATCH}.app", "explicitFileType = wrapper.application"),
    (ids[f"product:{WATCH_WIDGETS}"], f"{WATCH_WIDGETS}.appex", "explicitFileType = wrapper.app-extension"),
    (ids[f"product:{TESTS}"], f"{TESTS}.xctest", "explicitFileType = wrapper.cfbundle"),
]
for pid, name, kind in products:
    refs.append((pid, name,
                 f'isa = PBXFileReference; {kind}; includeInIndex = 0; path = {name}; sourceTree = BUILT_PRODUCTS_DIR;'))
for rid, name, body in sorted(refs):
    w(f"\t\t{rid} /* {name} */ = {{{body} }};")
w("/* End PBXFileReference section */")
w("")

# ---- PBXCopyFilesBuildPhase
w("/* Begin PBXCopyFilesBuildPhase section */")
for host, embedded, name, dst_path, spec in EMBEDS:
    product = f"{embedded}.appex" if embedded in (WIDGETS, WATCH_WIDGETS) else f"{embedded}.app"
    w(f"\t\t{ids[f'embed:{host}:{embedded}']} /* {name} */ = {{")
    w("\t\t\tisa = PBXCopyFilesBuildPhase;")
    w("\t\t\tbuildActionMask = 2147483647;")
    w(f"\t\t\tdstPath = {dst_path};")
    w(f"\t\t\tdstSubfolderSpec = {spec};")
    w("\t\t\tfiles = (")
    w(f"\t\t\t\t{ids[f'embedfile:{host}:{embedded}']} /* {product} in {name} */,")
    w("\t\t\t);")
    w(f'\t\t\tname = "{name}";')
    w("\t\t\trunOnlyForDeploymentPostprocessing = 0;")
    w("\t\t};")
w("/* End PBXCopyFilesBuildPhase section */")
w("")

# ---- PBXFrameworksBuildPhase
w("/* Begin PBXFrameworksBuildPhase section */")
for target in ALL_TARGETS:
    w(f"\t\t{ids[f'phase:frameworks:{target}']} /* Frameworks */ = {{")
    w("\t\t\tisa = PBXFrameworksBuildPhase;")
    w("\t\t\tbuildActionMask = 2147483647;")
    w("\t\t\tfiles = (")
    w("\t\t\t);")
    w("\t\t\trunOnlyForDeploymentPostprocessing = 0;")
    w("\t\t};")
w("/* End PBXFrameworksBuildPhase section */")
w("")

# ---- PBXGroup
def group(gid, children, path=None, name=None, comment=""):
    w(f"\t\t{gid} /* {comment or name or path} */ = {{")
    w("\t\t\tisa = PBXGroup;")
    w("\t\t\tchildren = (")
    for cid, cname in children:
        w(f"\t\t\t\t{cid} /* {cname} */,")
    w("\t\t\t);")
    if path:
        w(f"\t\t\tpath = {path};")
    elif name:
        w(f"\t\t\tname = {name};")
    w('\t\t\tsourceTree = "<group>";')
    w("\t\t};")

w("/* Begin PBXGroup section */")
group(ids["group:main"], [
    (ids["group:app"], APP), (ids["group:widgets"], WIDGETS),
    (ids["group:watch"], WATCH), (ids["group:watchwidgets"], WATCH_WIDGETS),
    (ids["group:tests"], TESTS), (ids["group:products"], "Products"),
], comment="")
group(ids["group:products"], [(pid, name) for pid, name, _ in products], name="Products")

app_subgroups = ["App", "Models", "Services", "ViewModels", "Views", "Resources"]
group(ids["group:app"],
      [(ids[f"group:app:{g}"], g) for g in app_subgroups]
      + [(fref(APP_ENTITLEMENTS), APP_ENTITLEMENTS[1])], path=APP)
for sub in app_subgroups:
    if sub == "Resources":
        children = [(fref(e), e[1]) for e in (ASSETS, STRINGS, INFO_STRINGS, APP_PLIST)]
    else:
        children = [(fref(e), e[1]) for e in APP_FILES[sub]]
    group(ids[f"group:app:{sub}"], children, path=sub)

group(ids["group:widgets"],
      [(fref(e), e[1]) for e in WIDGET_FILES + [WIDGET_PLIST, WIDGET_ENTITLEMENTS]], path=WIDGETS)
group(ids["group:watch"],
      [(fref(e), e[1]) for e in WATCH_FILES] + [(ids["group:watch:Resources"], "Resources")]
      + [(fref(WATCH_ENTITLEMENTS), WATCH_ENTITLEMENTS[1])], path=WATCH)
group(ids["group:watchwidgets"],
      [(fref(e), e[1]) for e in WATCH_WIDGET_FILES + [WATCH_WIDGET_PLIST, WATCH_WIDGET_ENTITLEMENTS]],
      path=WATCH_WIDGETS)
group(ids["group:watch:Resources"], [(fref(e), e[1]) for e in (WATCH_ASSETS, WATCH_PLIST)],
      path="Resources")
group(ids["group:tests"], [(fref(e), e[1]) for e in TEST_FILES], path=TESTS)
w("/* End PBXGroup section */")
w("")

# ---- PBXNativeTarget
TARGET_TYPES = {
    APP: ("com.apple.product-type.application", f"{APP}.app"),
    WIDGETS: ("com.apple.product-type.app-extension", f"{WIDGETS}.appex"),
    WATCH: ("com.apple.product-type.application", f"{WATCH}.app"),
    WATCH_WIDGETS: ("com.apple.product-type.app-extension", f"{WATCH_WIDGETS}.appex"),
    TESTS: ("com.apple.product-type.bundle.unit-test", f"{TESTS}.xctest"),
}
ids[f"dependency:{TESTS}"] = oid(f"dependency:{TESTS}")
TARGET_DEPS = {host: [] for host in ALL_TARGETS}
for host, embedded, *_ in EMBEDS:
    TARGET_DEPS[host].append((ids[f"dependency:{embedded}"], embedded))
TARGET_DEPS[TESTS].append((ids[f"dependency:{TESTS}"], APP))

w("/* Begin PBXNativeTarget section */")
for target in ALL_TARGETS:
    product_type, product_name = TARGET_TYPES[target]
    phases = [(ids[f"phase:sources:{target}"], "Sources"),
              (ids[f"phase:frameworks:{target}"], "Frameworks"),
              (ids[f"phase:resources:{target}"], "Resources")]
    for host, embedded, name, _, _ in EMBEDS:
        if host == target:
            phases.append((ids[f"embed:{host}:{embedded}"], name))

    w(f"\t\t{ids[f'target:{target}']} /* {target} */ = {{")
    w("\t\t\tisa = PBXNativeTarget;")
    w(f"\t\t\tbuildConfigurationList = {ids[f'configlist:{target}']} /* Build configuration list for PBXNativeTarget \"{target}\" */;")
    w("\t\t\tbuildPhases = (")
    for pid, pname in phases:
        w(f"\t\t\t\t{pid} /* {pname} */,")
    w("\t\t\t);")
    w("\t\t\tbuildRules = (")
    w("\t\t\t);")
    w("\t\t\tdependencies = (")
    for did, dname in TARGET_DEPS.get(target, []):
        w(f"\t\t\t\t{did} /* PBXTargetDependency */,")
    w("\t\t\t);")
    w(f"\t\t\tname = {target};")
    w(f"\t\t\tproductName = {target};")
    w(f"\t\t\tproductReference = {ids[f'product:{target}']} /* {product_name} */;")
    w(f'\t\t\tproductType = "{product_type}";')
    w("\t\t};")
w("/* End PBXNativeTarget section */")
w("")

# ---- PBXProject
w("/* Begin PBXProject section */")
w(f"\t\t{ids['project']} /* Project object */ = {{")
w("\t\t\tisa = PBXProject;")
w("\t\t\tattributes = {")
w("\t\t\t\tBuildIndependentTargetsInParallel = 1;")
w("\t\t\t\tLastSwiftUpdateCheck = 1500;")
w("\t\t\t\tLastUpgradeCheck = 1500;")
w("\t\t\t\tTargetAttributes = {")
for target in ALL_TARGETS:
    w(f"\t\t\t\t\t{ids[f'target:{target}']} = {{")
    w("\t\t\t\t\t\tCreatedOnToolsVersion = 15.0;")
    if target == TESTS:
        w(f"\t\t\t\t\t\tTestTargetID = {ids[f'target:{APP}']};")
    w("\t\t\t\t\t};")
w("\t\t\t\t};")
w("\t\t\t};")
w(f"\t\t\tbuildConfigurationList = {ids['configlist:project']} /* Build configuration list for PBXProject \"{APP}\" */;")
w('\t\t\tcompatibilityVersion = "Xcode 14.0";')
w("\t\t\tdevelopmentRegion = fr;")
w("\t\t\thasScannedForEncodings = 0;")
w("\t\t\tknownRegions = (")
for region in ("fr", "en", "es", "Base"):
    w(f"\t\t\t\t{region},")
w("\t\t\t);")
w(f"\t\t\tmainGroup = {ids['group:main']};")
w(f"\t\t\tproductRefGroup = {ids['group:products']} /* Products */;")
w('\t\t\tprojectDirPath = "";')
w('\t\t\tprojectRoot = "";')
w("\t\t\ttargets = (")
for target in ALL_TARGETS:
    w(f"\t\t\t\t{ids[f'target:{target}']} /* {target} */,")
w("\t\t\t);")
w("\t\t};")
w("/* End PBXProject section */")
w("")

# ---- PBXResourcesBuildPhase / PBXSourcesBuildPhase
for section, table, phase_key, label in (
    ("PBXResourcesBuildPhase", TARGET_RESOURCES, "phase:resources", "Resources"),
    ("PBXSourcesBuildPhase", TARGET_SOURCES, "phase:sources", "Sources"),
):
    w(f"/* Begin {section} section */")
    for target in ALL_TARGETS:
        w(f"\t\t{ids[f'{phase_key}:{target}']} /* {label} */ = {{")
        w(f"\t\t\tisa = {section};")
        w("\t\t\tbuildActionMask = 2147483647;")
        w("\t\t\tfiles = (")
        for entry in table[target]:
            w(f"\t\t\t\t{bfile(target, entry)} /* {entry[1]} in {label} */,")
        w("\t\t\t);")
        w("\t\t\trunOnlyForDeploymentPostprocessing = 0;")
        w("\t\t};")
    w(f"/* End {section} section */")
    w("")

# ---- PBXTargetDependency
w("/* Begin PBXTargetDependency section */")
dependencies = [(f"dependency:{e}", f"target:{e}", f"proxy:{e}") for _, e, *_ in EMBEDS]
dependencies.append((f"dependency:{TESTS}", f"target:{APP}", f"proxy:{TESTS}"))
for dep_key, target_key, proxy_key in dependencies:
    w(f"\t\t{ids[dep_key]} /* PBXTargetDependency */ = {{")
    w("\t\t\tisa = PBXTargetDependency;")
    w(f"\t\t\ttarget = {ids[target_key]};")
    w(f"\t\t\ttargetProxy = {ids[proxy_key]} /* PBXContainerItemProxy */;")
    w("\t\t};")
w("/* End PBXTargetDependency section */")
w("")

# ---- XCBuildConfiguration
SHARED = [
    ("ALWAYS_SEARCH_USER_PATHS", "NO"),
    ("ASSETCATALOG_COMPILER_GENERATE_ASSET_SYMBOL_FRAMEWORKS", "SwiftUI"),
    ("CLANG_ANALYZER_NONNULL", "YES"),
    ("CLANG_ENABLE_MODULES", "YES"),
    ("CLANG_ENABLE_OBJC_ARC", "YES"),
    ("CLANG_WARN_BOOL_CONVERSION", "YES"),
    ("CLANG_WARN_CONSTANT_CONVERSION", "YES"),
    ("CLANG_WARN_DOCUMENTATION_COMMENTS", "YES"),
    ("CLANG_WARN_EMPTY_BODY", "YES"),
    ("CLANG_WARN_INFINITE_RECURSION", "YES"),
    ("CLANG_WARN_INT_CONVERSION", "YES"),
    ("CLANG_WARN_SUSPICIOUS_MOVE", "YES"),
    ("CLANG_WARN_UNGUARDED_AVAILABILITY", "YES_AGGRESSIVE"),
    ("CLANG_WARN_UNREACHABLE_CODE", "YES"),
    ("COPY_PHASE_STRIP", "NO"),
    ("ENABLE_STRICT_OBJC_MSGSEND", "YES"),
    ("ENABLE_USER_SCRIPT_SANDBOXING", "YES"),
    ("GCC_C_LANGUAGE_STANDARD", "gnu17"),
    ("GCC_NO_COMMON_BLOCKS", "YES"),
    ("GCC_WARN_ABOUT_RETURN_TYPE", "YES_ERROR"),
    ("GCC_WARN_UNINITIALIZED_AUTOS", "YES_AGGRESSIVE"),
    ("GCC_WARN_UNUSED_FUNCTION", "YES"),
    ("GCC_WARN_UNUSED_VARIABLE", "YES"),
    ("IPHONEOS_DEPLOYMENT_TARGET", "17.0"),
    ("LOCALIZATION_PREFERS_STRING_CATALOGS", "YES"),
    ("MTL_FAST_MATH", "YES"),
    ("SDKROOT", "iphoneos"),
    ("WATCHOS_DEPLOYMENT_TARGET", "10.0"),
]
PROJECT_DEBUG = SHARED + [
    ("DEBUG_INFORMATION_FORMAT", "dwarf"),
    ("ENABLE_TESTABILITY", "YES"),
    ("GCC_OPTIMIZATION_LEVEL", "0"),
    ("GCC_PREPROCESSOR_DEFINITIONS", '(\n\t\t\t\t\t"DEBUG=1",\n\t\t\t\t\t"$(inherited)",\n\t\t\t\t)'),
    ("MTL_ENABLE_DEBUG_INFO", "INCLUDE_SOURCE"),
    ("ONLY_ACTIVE_ARCH", "YES"),
    ("SWIFT_ACTIVE_COMPILATION_CONDITIONS", '"DEBUG $(inherited)"'),
    ("SWIFT_OPTIMIZATION_LEVEL", '"-Onone"'),
]
PROJECT_RELEASE = SHARED + [
    ("DEBUG_INFORMATION_FORMAT", '"dwarf-with-dsym"'),
    ("ENABLE_NS_ASSERTIONS", "NO"),
    ("MTL_ENABLE_DEBUG_INFO", "NO"),
    ("SWIFT_COMPILATION_MODE", "wholemodule"),
    ("VALIDATE_PRODUCT", "YES"),
]
COMMON_TARGET = [
    ("CODE_SIGN_STYLE", "Automatic"),
    ("CURRENT_PROJECT_VERSION", "1"),
    ("MARKETING_VERSION", "1.0"),
    ("PRODUCT_NAME", '"$(TARGET_NAME)"'),
    ("SWIFT_VERSION", "5.0"),
]
RUNPATH_APP = ("LD_RUNPATH_SEARCH_PATHS",
               '(\n\t\t\t\t\t"$(inherited)",\n\t\t\t\t\t"@executable_path/Frameworks",\n\t\t\t\t)')
RUNPATH_EXT = ("LD_RUNPATH_SEARCH_PATHS",
               '(\n\t\t\t\t\t"$(inherited)",\n\t\t\t\t\t"@executable_path/Frameworks",\n\t\t\t\t\t"@executable_path/../../Frameworks",\n\t\t\t\t)')

TARGET_SETTINGS = {
    APP: COMMON_TARGET + [
        ("CODE_SIGN_ENTITLEMENTS", f"{APP}/Kliima.entitlements"),
        ("ASSETCATALOG_COMPILER_APPICON_NAME", "AppIcon"),
        ("ASSETCATALOG_COMPILER_GLOBAL_ACCENT_COLOR_NAME", "AccentColor"),
        ("ENABLE_PREVIEWS", "YES"),
        ("GENERATE_INFOPLIST_FILE", "NO"),
        ("INFOPLIST_FILE", f"{APP}/Resources/Info.plist"),
        RUNPATH_APP,
        ("PRODUCT_BUNDLE_IDENTIFIER", "com.kliima.app"),
        ("SWIFT_EMIT_LOC_STRINGS", "YES"),
        ("TARGETED_DEVICE_FAMILY", '"1,2"'),
    ],
    WIDGETS: COMMON_TARGET + [
        ("CODE_SIGN_ENTITLEMENTS", f"{WIDGETS}/KliimaWidgets.entitlements"),
        ("ENABLE_PREVIEWS", "YES"),
        ("GENERATE_INFOPLIST_FILE", "NO"),
        ("INFOPLIST_FILE", f"{WIDGETS}/Info.plist"),
        RUNPATH_EXT,
        ("PRODUCT_BUNDLE_IDENTIFIER", "com.kliima.app.widgets"),
        ("SKIP_INSTALL", "YES"),
        ("SWIFT_EMIT_LOC_STRINGS", "YES"),
        ("TARGETED_DEVICE_FAMILY", '"1,2"'),
    ],
    WATCH: COMMON_TARGET + [
        ("ASSETCATALOG_COMPILER_APPICON_NAME", "AppIcon"),
        ("CODE_SIGN_ENTITLEMENTS", f"{WATCH}/KliimaWatch.entitlements"),
        ("ASSETCATALOG_COMPILER_GLOBAL_ACCENT_COLOR_NAME", "AccentColor"),
        ("ENABLE_PREVIEWS", "YES"),
        ("GENERATE_INFOPLIST_FILE", "NO"),
        ("INFOPLIST_FILE", f"{WATCH}/Resources/Info.plist"),
        RUNPATH_APP,
        ("PRODUCT_BUNDLE_IDENTIFIER", "com.kliima.app.watchkitapp"),
        ("SDKROOT", "watchos"),
        ("SUPPORTED_PLATFORMS", '"watchos watchsimulator"'),
        ("SWIFT_EMIT_LOC_STRINGS", "YES"),
        ("TARGETED_DEVICE_FAMILY", "4"),
    ],
    WATCH_WIDGETS: COMMON_TARGET + [
        ("CODE_SIGN_ENTITLEMENTS", f"{WATCH_WIDGETS}/KliimaWatchWidgets.entitlements"),
        ("ENABLE_PREVIEWS", "YES"),
        ("GENERATE_INFOPLIST_FILE", "NO"),
        ("INFOPLIST_FILE", f"{WATCH_WIDGETS}/Info.plist"),
        RUNPATH_EXT,
        ("PRODUCT_BUNDLE_IDENTIFIER", "com.kliima.app.watchkitapp.complications"),
        ("SDKROOT", "watchos"),
        ("SKIP_INSTALL", "YES"),
        ("SUPPORTED_PLATFORMS", '"watchos watchsimulator"'),
        ("SWIFT_EMIT_LOC_STRINGS", "YES"),
        ("TARGETED_DEVICE_FAMILY", "4"),
    ],
    TESTS: COMMON_TARGET + [
        ("BUNDLE_LOADER", '"$(TEST_HOST)"'),
        ("GENERATE_INFOPLIST_FILE", "YES"),
        ("PRODUCT_BUNDLE_IDENTIFIER", "com.kliima.app.tests"),
        ("SWIFT_EMIT_LOC_STRINGS", "NO"),
        ("TARGETED_DEVICE_FAMILY", '"1,2"'),
        ("TEST_HOST", f'"$(BUILT_PRODUCTS_DIR)/{APP}.app/$(BUNDLE_EXECUTABLE_FOLDER_PATH)/{APP}"'),
    ],
}

def build_config(cid, name, settings):
    w(f"\t\t{cid} /* {name} */ = {{")
    w("\t\t\tisa = XCBuildConfiguration;")
    w("\t\t\tbuildSettings = {")
    for key, value in sorted(settings):
        w(f"\t\t\t\t{key} = {value};")
    w("\t\t\t};")
    w(f"\t\t\tname = {name};")
    w("\t\t};")

w("/* Begin XCBuildConfiguration section */")
build_config(oid("config:project:Debug"), "Debug", PROJECT_DEBUG)
build_config(oid("config:project:Release"), "Release", PROJECT_RELEASE)
for target in ALL_TARGETS:
    for cfg in ("Debug", "Release"):
        build_config(ids[f"config:{cfg}:{target}"], cfg, TARGET_SETTINGS[target])
w("/* End XCBuildConfiguration section */")
w("")

# ---- XCConfigurationList
def config_list(lid, comment, debug, release):
    w(f"\t\t{lid} /* {comment} */ = {{")
    w("\t\t\tisa = XCConfigurationList;")
    w("\t\t\tbuildConfigurations = (")
    w(f"\t\t\t\t{debug} /* Debug */,")
    w(f"\t\t\t\t{release} /* Release */,")
    w("\t\t\t);")
    w("\t\t\tdefaultConfigurationIsVisible = 0;")
    w("\t\t\tdefaultConfigurationName = Release;")
    w("\t\t};")

w("/* Begin XCConfigurationList section */")
config_list(ids["configlist:project"], f'Build configuration list for PBXProject "{APP}"',
            oid("config:project:Debug"), oid("config:project:Release"))
for target in ALL_TARGETS:
    config_list(ids[f"configlist:{target}"],
                f'Build configuration list for PBXNativeTarget "{target}"',
                ids[f"config:Debug:{target}"], ids[f"config:Release:{target}"])
w("/* End XCConfigurationList section */")
w("\t};")
w(f"\trootObject = {ids['project']} /* Project object */;")
w("}")

target_path = sys.argv[1]
os.makedirs(os.path.dirname(target_path), exist_ok=True)
with open(target_path, "w") as fh:
    fh.write("\n".join(out) + "\n")
print("écrit :", target_path)

# Schéma partagé de l'application (les autres cibles suivent par dépendance).
scheme_dir = os.path.join(os.path.dirname(target_path), "xcshareddata", "xcschemes")
os.makedirs(scheme_dir, exist_ok=True)

def buildable(target, product):
    return f'''<BuildableReference
               BuildableIdentifier = "primary"
               BlueprintIdentifier = "{ids[f'target:{target}']}"
               BuildableName = "{product}"
               BlueprintName = "{target}"
               ReferencedContainer = "container:{APP}.xcodeproj">
            </BuildableReference>'''

for target, product in ((APP, f"{APP}.app"), (WATCH, f"{WATCH}.app")):
    testables = f'''<TestableReference
            skipped = "NO">
            {buildable(TESTS, f"{TESTS}.xctest")}
         </TestableReference>''' if target == APP else ""
    scheme = f'''<?xml version="1.0" encoding="UTF-8"?>
<Scheme
   LastUpgradeVersion = "1500"
   version = "1.7">
   <BuildAction
      parallelizeBuildables = "YES"
      buildImplicitDependencies = "YES">
      <BuildActionEntries>
         <BuildActionEntry
            buildForTesting = "YES"
            buildForRunning = "YES"
            buildForProfiling = "YES"
            buildForArchiving = "YES"
            buildForAnalyzing = "YES">
            {buildable(target, product)}
         </BuildActionEntry>
      </BuildActionEntries>
   </BuildAction>
   <TestAction
      buildConfiguration = "Debug"
      selectedDebuggerIdentifier = "Xcode.DebuggerFoundation.Debugger.LLDB"
      selectedLauncherIdentifier = "Xcode.DebuggerFoundation.Launcher.LLDB"
      shouldUseLaunchSchemeArgsEnv = "YES">
      <Testables>
         {testables}
      </Testables>
   </TestAction>
   <LaunchAction
      buildConfiguration = "Debug"
      selectedDebuggerIdentifier = "Xcode.DebuggerFoundation.Debugger.LLDB"
      selectedLauncherIdentifier = "Xcode.DebuggerFoundation.Launcher.LLDB"
      launchStyle = "0"
      useCustomWorkingDirectory = "NO"
      ignoresPersistentStateOnLaunch = "NO"
      debugDocumentVersioning = "YES"
      debugServiceExtension = "internal"
      allowLocationSimulation = "YES">
      <BuildableProductRunnable
         runnableDebuggingMode = "0">
         {buildable(target, product)}
      </BuildableProductRunnable>
   </LaunchAction>
   <ProfileAction
      buildConfiguration = "Release"
      shouldUseLaunchSchemeArgsEnv = "YES"
      savedToolIdentifier = ""
      useCustomWorkingDirectory = "NO"
      debugDocumentVersioning = "YES">
      <BuildableProductRunnable
         runnableDebuggingMode = "0">
         {buildable(target, product)}
      </BuildableProductRunnable>
   </ProfileAction>
   <AnalyzeAction
      buildConfiguration = "Debug">
   </AnalyzeAction>
   <ArchiveAction
      buildConfiguration = "Release"
      revealArchiveInOrganizer = "YES">
   </ArchiveAction>
</Scheme>
'''
    with open(os.path.join(scheme_dir, f"{target}.xcscheme"), "w") as fh:
        fh.write(scheme)
print("schémas :", ", ".join(f"{t}.xcscheme" for t in (APP, WATCH)))
