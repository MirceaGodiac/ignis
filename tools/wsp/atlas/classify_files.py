"""File catalogue of NASA's PSI flight combustion investigations: rules only, no model calls.

Reads the file listings (one JSON object per listed file: category, file_name, size_bytes,
stable_url, ...) of the 19 investigations and classifies every listed file by what it is and which
test it belongs to, from its name, its PSI category and, where the raw listing tree is on disk,
the folder it sits in. Nothing is downloaded and no file content is read.

Inputs:
  <manifests>/PSI-<id>.listing.jsonl          the flat listing (required)
  <samples>/_meta/files_PSI-<id>.json         the raw listing tree, for the folder of each file
                                              (optional: without it `folder` is empty)

Outputs (under runs/dev/data_all/work/atlas/ by default):
  files.jsonl          one row per listed file
  files_summary.json   per investigation: files and bytes by kind, tests with each measurement,
                       what one test costs to download (text against images and video)
  files_tables.md      with --tables: the per-investigation tables of docs/wsp/atlas/FILES.md

Usage (from the repository root):
  python tools/wsp/atlas/classify_files.py
  python tools/wsp/atlas/classify_files.py --tables
  python tools/wsp/atlas/classify_files.py --out some/dir

Rules of this file: names, categories and folders only. It never opens an experiment table, a
record or a data file, so it cannot read an outcome. The columns and the closed lists are
described in docs/wsp/atlas/FILES.md; tools/wsp/atlas/check_files.py checks the output.
"""
import argparse
import datetime
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

# investigation -> (listing directory, raw listing tree directory)
NEW = "runs/dev/data_all/manifests", "runs/dev/data_all/raw/samples/_meta"
OLD = "data/manifests", "data/raw/samples/_meta"
INVESTIGATIONS = {
    "PSI-10": NEW, "PSI-20": NEW, "PSI-21": NEW, "PSI-22": NEW, "PSI-23": NEW, "PSI-39": NEW,
    "PSI-47": NEW, "PSI-68": NEW, "PSI-98": NEW, "PSI-99": NEW, "PSI-100": NEW, "PSI-101": NEW,
    "PSI-102": NEW, "PSI-106": NEW, "PSI-107": NEW, "PSI-69": OLD, "PSI-159": OLD, "PSI-25": OLD,
    "PSI-26": OLD,
}
ORDER = sorted(INVESTIGATIONS, key=lambda s: int(s.split("-")[1]))

FAMILY = {
    "PSI-10": "acme", "PSI-20": "acme", "PSI-21": "acme", "PSI-22": "acme", "PSI-23": "acme",
    "PSI-159": "acme",
    "PSI-69": "droplet", "PSI-68": "droplet", "PSI-39": "droplet",
    "PSI-25": "bass2", "PSI-26": "bass",
    "PSI-98": "saffire", "PSI-99": "saffire", "PSI-100": "saffire",
    "PSI-101": "same", "PSI-102": "same",
    "PSI-106": "slice", "PSI-107": "spice", "PSI-47": "daft",
}

KINDS = (
    "experiment table", "script or settings", "sensor time series", "camera settings",
    "image archive", "video", "processed or analysed data", "calibration", "science document",
    "engineering document", "report", "presentation", "reference data", "metadata", "other",
)
NAME = "stated in the name"
SIB = "inferred from a sibling investigation"
OPEN = "checked by opening one file"
UNK = "unknown"
CONFIDENCES = (NAME, SIB, OPEN, UNK)

# Schemes that name a test (a burn, a flame, a sample). Other schemes name an auxiliary run
# (calibration point, fibre reference, radiometer check) and are not counted as tests.
TEST_SCHEMES = {
    "acme_test_point", "cir_test_point", "flex_number", "bass2_test", "saffire_sample",
    "same_test_number", "spice_test_point", "daft_instrument_test",
}

MEASUREMENTS = ("sensor time series", "image archive", "video", "processed or analysed data")


# --------------------------------------------------------------------------------------------
# helpers
# --------------------------------------------------------------------------------------------

def yyddd(year, month, day):
    d = datetime.date(int(year), int(month), int(day))
    return "%02d%03d" % (d.year % 100, d.timetuple().tm_yday)


def stem_of(inv, category, file_name):
    pre = "%s_%s_" % (inv, category)
    return file_name[len(pre):] if file_name.startswith(pre) else file_name


def ext_of(stem):
    s = stem.strip()
    m = re.search(r"\.([A-Za-z0-9]{1,8})\s*$", s)
    return m.group(1).lower() if m else ""


class Rule:
    __slots__ = ("rid", "rx", "kind", "what", "instrument", "conf", "folder_rx")

    def __init__(self, rid, pattern, kind, what, instrument=None, conf=NAME, folder=None, flags=re.I):
        assert kind in KINDS and conf in CONFIDENCES, rid
        self.rid = rid
        self.rx = re.compile(pattern, flags) if pattern else None
        self.folder_rx = re.compile(folder, re.I) if folder else None
        self.kind, self.what, self.instrument, self.conf = kind, what, instrument, conf

    def match(self, stem, folder):
        if self.folder_rx is not None and not self.folder_rx.search(folder or ""):
            return None
        if self.rx is None:
            return True
        return self.rx.search(stem)


def first(rules, stem, folder):
    for r in rules:
        m = r.match(stem, folder)
        if m:
            inst = r.instrument(m) if callable(r.instrument) else r.instrument
            return r, inst
    return None, None


# --------------------------------------------------------------------------------------------
# document rules shared by every investigation (documents filed under Raw Data, mostly "Legacy")
# --------------------------------------------------------------------------------------------

DOC_EXT = r"\.(pdf|docx?|pptx?|xlsx?|zip)\s*$"
DOC_RULES = [
    Rule("doc.folder.presentation", DOC_EXT, "presentation", "presentation (folder name says so)",
         folder=r"(External|Internal NASA) Presentation"),
    Rule("doc.folder.report", DOC_EXT, "report", "report (folder name says so)",
         folder=r"/\d{4} - [^/]*(Final Report|Post Flight Report|Verification Report)"),
    Rule("doc.folder.publication", r"\.pdf\s*$", "science document", "publication (folder is a citation)",
         folder=r"/(19|20)\d{2} - [^/]{12,}"),
    Rule("doc.report", r"(final[ _]report|FinalRpt|Post ?Flight ?Report|30-?day report|_Final Report|1YearReport|Accomplishments Report)",
         "report", "report"),
    Rule("doc.drawing", r"(^s?\d{4,5}[A-Z]{2,4}\d{3,5}|_drawings_|Drawing[_ ]|Schematic|Assy\b|Assembly|Modification|Alteration|Backshell|Bracket|Ignitor|Control Box|Top Window|Video Base|Interface Board)",
         "engineering document", "engineering drawing or drawing package"),
    Rule("doc.engineering", r"(Engineering[ _]|ERD\b|EIRD|Software[ _](Interface|Description|Requirement)|ACME-DOC|Requirements? Doc|VPRED|Wire_(Report|Test)|Offgassing|Operations\.pdf|ACME (Imaging|Instruments)\.zip|hot wire anemometry)",
         "engineering document", "engineering document"),
    Rule("doc.presentation", r"(ASGSR|CSSCI|AIAA[ _]presentation|poster|pictorial[ _]overview|CDR[ _]Presentation|IAC[ _]\d{4}|Symposium|Crew Training|Review Board|Forward Plan|Requirements Design Review|_wf(_\d)?\.mp4|_WF\.mp4|Interviews)",
         "presentation", "presentation or outreach video"),
    Rule("doc.science", r"(SRD|ISRD|SORD|\bSCR\b|\bRDR\b|\bDoD\b|Science[ _](Requirements|Operation)|fact[ _]sheet|info[ _]sheet|Researchers Guide|procedure|data[ _]interpretation|Data interpretation|file description|Readme|Test_Summary|Unlocking Mysteries|ACME on the ISS|Decisive Step|ACME Gas flow\.zip|Nomenclature|Organization Guide|Objectives, Development|Operations Manual|Concept Description|Test_Plan|benefits)",
         "science document", "science document"),
    Rule("doc.publication", r"^(19|20)\d{2} [A-Z][a-z]+ (et al|&) ", "science document", "publication"),
]

# --------------------------------------------------------------------------------------------
# ACME family: Flame Design, BRE, CLD Flame, E-FIELD Flames, s-Flame, CFI-G
# --------------------------------------------------------------------------------------------

ACME_T = re.compile(r"(?<!\d)(\d{5})([A-Z]{1,2})(\d{1,2})(?!\d)")
ACME_AUX = re.compile(r"(?<!\d)(\d{5,6})(Fiber\d+[A-C]?|RAD\d+[ab]?|MFCA\d?_\d+|MFAC\d?_\d+)")
ACME_DAY = [
    re.compile(r"^(\d{5})[ _]"), re.compile(r"FusedSilica_(\d{5})"), re.compile(r"udp_+(\d{5})_Igniter"),
    re.compile(r"_(\d{5})_IgniterPosition"), re.compile(r"^(\d{5})\.(pdf|txt)$"), re.compile(r"^(\d{5})E_"),
]
ACME_DATE = [
    re.compile(r"(?<!\d)(20\d\d)-(\d\d)-(\d\d)_\d\d-\d\d"),
    re.compile(r"AcmeCameraConfigure_(20\d\d)-(\d\d)-(\d\d)_"),
    re.compile(r"moduleDataACD_\d+_(20\d\d)(\d\d)(\d\d)_"),
    re.compile(r"GC_Coverter_.*_(20\d\d)(\d\d)(\d\d)_\d{4}"),
]

CAM_ACME = "ACME colour camera"
CAM_INT = "ACME intensified camera"
CAM_HIBMS = "CIR HiBMs camera"
CAM_UV = "CIR LLL-UV camera"
VID_OPS = "operations video downlink"

ACME_RULES = [
    # numeric data
    Rule("acme.data_csv", r"_data( ?\(?\d\)?)?\.csv$", "sensor time series",
         "analog channels at 125 Hz: flow readbacks, pressures, photomultipliers, thermocouples, radiometers",
         "ACME data acquisition", OPEN),
    Rule("acme.data_zip", r"_data\.zip$", "sensor time series", "the 125 Hz data file, zipped", "ACME data acquisition", SIB),
    Rule("acme.data_errors", r"_data\.csv\.ERRORS$", "other", "error report beside a data file"),
    Rule("acme.mfc_setpoints", r"_mfcSetpoints\d?\.csv$", "script or settings",
         "command log of the three mass-flow-controller setpoints", "mass flow controllers", OPEN),
    Rule("acme.tp_analysis", r"TP_ANALYSIS_FCU_.*Readings\.txt(\.log)?$", "sensor time series",
         "rack fluids-control-unit pressure transducers and flow controllers against time",
         "CIR fluids control unit", OPEN),
    Rule("acme.cam_exposures", r"_ACME_C(A)?M\d_Exp\w*\.(txt|xlsx)$", "camera settings",
         "camera telemetry against time (exposure, gain, frame rate)", CAM_ACME, OPEN),
    Rule("acme.gc_asc", r"moduleDataACD_\d+_\d{8}_\d{4}\.asc$", "sensor time series",
         "gas chromatograph run: three channels at 100 Hz", "gas chromatograph", OPEN),
    Rule("acme.gc_zip", r"^\d{5}_GC\.zip$", "sensor time series", "gas chromatograph files of the day, zipped",
         "gas chromatograph", SIB),
    Rule("acme.gc_log", r"GC_Co(n)?verter.*\.(log|jpg)$", "other", "gas chromatograph converter log or screenshot",
         "gas chromatograph", OPEN),
    # script and settings
    Rule("acme.camera_configure", r"AcmeCameraConfigure", "camera settings",
         "camera configuration script of the day", CAM_ACME),
    Rule("acme.tp_txt", r"_TP\.txt$", "script or settings",
         "as-run stage timeline of the test point, with stamps (outcome-bearing)", None, OPEN),
    Rule("acme.script_named", r"_(VoltageSweep|BRE)\.txt$", "script or settings",
         "test point script parameters (key:value)", None, OPEN),
    Rule("acme.as_run", r"^as run\d{5}[A-Z]{1,2}\d{1,2}.*\.txt$", "script or settings",
         "test point script parameters (key:value)", None, OPEN),
    Rule("acme.script_plain", r"^\d{5}[A-Z]{1,2}\d{1,2}A?\.txt$", "script or settings",
         "test point script parameters (key:value)", None, OPEN),
    # notes
    Rule("acme.comments", r"_(Images?)?Comments?( ?on.*)?\.txt$", "metadata",
         "operator notes on the image downlink of the test", None, OPEN),
    Rule("acme.day_note", r"^\d{5}(E_Comments)?\.txt$", "metadata", "day note"),
    # image archives
    Rule("acme.zip_fiber", r"^\d{5}Fiber\d+[A-C]?(_ACME(_Converted)?|_Images)?\.zip$", "image archive",
         "thin-filament reference images", CAM_ACME),
    Rule("acme.zip_hibms", r"_HiBMs?\.zip$", "image archive", "frames of one test", CAM_HIBMS),
    Rule("acme.zip_llluv", r"_LLL-?UV\.zip$", "image archive", "frames of one test", CAM_UV),
    Rule("acme.zip_ipsu", r"[A-E]_IPSU\d\.zip$", "image archive", "frames of one test", "CIR IPSU camera"),
    Rule("acme.zip_int_ignition", r"(Intensifier|INTNS)(_B\d)?_Ignition\.zip$", "image archive",
         "frames around ignition", CAM_INT),
    Rule("acme.zip_int_all", r"_(INTNS|Inte\w+)_ALL(zip)?\.zip$", "image archive", "all frames of one test", CAM_INT),
    Rule("acme.zip_int", r"_(IPSUG_)?(INTNS|Inte\w+)\.zip$", "image archive", "frames of one test (selection)", CAM_INT),
    Rule("acme.zip_acme_converted", r"(_A(C)?ME_?Converted( images)?|Images[ _]Converted|Images_Converted ?\(?\d|_COnverted|_Converted)\.?.*\.zip$",
         "image archive", "frames of one test converted to a viewable format", CAM_ACME),
    Rule("acme.zip_acme", r"(_ACME_?\d?(zip)?|-ACME|_IPSUG_ACME)\.zi+p$", "image archive", "frames of one test as downlinked", CAM_ACME),
    Rule("acme.zip_images", r"(^Images_ ?\(?\d{5}|\d[ _]Images)\.zip$|^Images_ ?\(?\d{5}[A-Z]{1,2}\d{1,2}\.zip$", "image archive",
         "frames of one test as downlinked", CAM_ACME, SIB),
    Rule("acme.zip_selection", r"_TP_Image_Selections_Originals_\d+\.zip$", "image archive", "selected frames of one test"),
    Rule("acme.zip_igniter_position", r"IgniterPosition\d+.*_Originals\.zip$", "image archive", "igniter position check frames"),
    Rule("acme.zip_plain", r"^\d{5}[A-Z]{1,2}\d{1,2}\.zip$", "image archive", "archive named only by the test", None, UNK),
    # video
    Rule("acme.video", r"v[li]c-record.*\.(ts|mp4)$", "video", "recording of the downlinked operations video", VID_OPS),
    Rule("acme.video_zip", r"v[li]c-record.*\.zip$", "video", "the original transport stream of the recording, zipped", VID_OPS, SIB),
    Rule("acme.video_sidecar", r"\.mp4\.sfk$", "other", "editor sidecar of a video"),
    # per-frame index
    Rule("acme.frame_index_int", r"(^INTNS_.*\.mdipsug(\.ERRORS)?|_INTNS_.*\.metadata)$", "metadata",
         "per-frame index of the archive (time stamp, exposure, gain)", CAM_INT, SIB),
    Rule("acme.frame_index_acme", r"(^ACME_.*\.mdipsug(\.ERRORS)?|_ACME_.*\.metadata)$", "metadata",
         "per-frame index of the archive (time stamp, exposure, gain)", CAM_ACME, OPEN),
    Rule("acme.frame_counter", r"\.next$", "metadata", "frame counter of the camera unit", None, OPEN),
    # logs
    Rule("acme.log_commands", r"ACMECommands\.log\s*$", "other", "command log of the avionics software"),
    Rule("acme.log_acme", r"ACME\.log\s*$", "other", "message log of the avionics software"),
    Rule("acme.log_ipsug", r"IPSUG\.log\s*$", "other", "message log of the camera unit", None, OPEN),
    Rule("acme.logs_zip", r"_Logs\.zip$", "other", "logs, zipped"),
    # day paperwork
    Rule("acme.day_test_point_log", r"(daily test point log|dail;y test point log|^\d{5} test point log|Daily Test Point Log).*\.(pdf|xlsx)$",
         "experiment table", "test point log of the day"),
    Rule("acme.day_test_sheets", r"^\d{5} tests? sheets\.pdf$", "metadata", "test sheets of the day (scan)"),
    Rule("acme.day_parameter_notes", r"^\d{5} test param\w+ notes\.pdf$", "metadata", "test parameter notes of the day (scan)"),
    Rule("acme.day_traces", r"^\d{5}( flow traces etc| velocity traces|_MFCA_all).*\.pdf$", "processed or analysed data",
         "plots of the day's flow or velocity traces"),
    Rule("acme.day_pdf", r"^\d{5}\.pdf$", "other", "day document named only by the day", None, UNK),
    Rule("acme.chamber_calculator", r"(Chamber Calculator|ACMEFlowCalculator|RawDataConversion).*\.xlsx$",
         "processed or analysed data", "chamber atmosphere or flow calculator workbook", None, SIB),
    Rule("acme.screenshot", r"\.(png|jpg)$", "other", "screenshot"),
]


def valid_day(yyddd_text):
    return yyddd_text.isdigit() and 1 <= int(yyddd_text[2:]) <= 366


def acme_test(stem, folder, ctx=None):
    out = {}
    aux = ACME_AUX.search(stem)
    m = ACME_T.search(stem)
    if m and not (aux and aux.start() <= m.start()):
        out.update(test_id=m.group(0), scheme="acme_test_point", source="name", day=m.group(1))
        others = [x.group(0) for x in ACME_T.finditer(stem) if x.group(0) != m.group(0)]
        if others:
            out["others"] = sorted(set(others))
        # "<name>_1": the same test point name opened a second time; kept as a variant
        v = re.match(r"^[-_](\d)(?=[-_.]|$)", stem[m.end():])
        if v:
            out["variant"] = "_" + v.group(1)
    elif aux:
        day = aux.group(1)[-5:] if len(aux.group(1)) == 5 else None
        out.update(test_id=aux.group(1) + aux.group(2), scheme="acme_auxiliary_run", source="name", day=day)
    ftest = None
    fday = None
    for comp in reversed((folder or "").split("/")):
        fm = re.match(r"^(\d{5})([A-Z]{1,2})(\d{1,2})(?!\d)", comp.strip())
        if fm and ftest is None:
            ftest = fm.group(0)
        dm = re.match(r"^(\d{5})(\D|$)", comp.strip())
        if dm and fday is None:
            fday = dm.group(1)
    if "test_id" not in out and ftest and not re.search(r"^\d{5}[ _.]|Fiber|RAD\d|MFCA|MFAC|Calculator|CameraConfigure|\.log\s*$|moduleDataACD", stem):
        out.update(test_id=ftest, scheme="acme_test_point", source="folder", day=ftest[:5])
    elif out.get("scheme") == "acme_test_point" and ftest and ftest != out["test_id"]:
        # Name and folder disagree. The name wins, unless the two differ only in the day and the
        # day in the name is impossible or hardly used elsewhere while the folder's day is common:
        # then the name is taken to be mistyped.
        name_id, counts = out["test_id"], (ctx or {}).get("day_counts", {})
        dn, df = name_id[:5], ftest[:5]
        mistyped = name_id[5:] == ftest[5:] and dn != df and valid_day(df) and (
            not valid_day(dn) or counts.get(dn, 0) * 5 < counts.get(df, 0))
        if mistyped:
            out.update(test_id=ftest, source="folder", day=df)
            out["alternative"] = name_id
        else:
            out["alternative"] = ftest
    if not out.get("day"):
        for rx in ACME_DAY:
            dm = rx.search(stem)
            if dm:
                out["day"] = dm.group(1)
                break
    if not out.get("day"):
        for rx in ACME_DATE:
            dm = rx.search(stem)
            if dm:
                try:
                    out["day"] = yyddd(dm.group(1), dm.group(2), dm.group(3))
                except ValueError:
                    pass
                break
    if not out.get("day") and fday:
        out["day"] = fday
    if out.get("day"):
        out["day_scheme"] = "YYDDD (GMT)"
    return out


# --------------------------------------------------------------------------------------------
# Droplet family in the Combustion Integrated Rack: FLEX, FLEX-2, CFI
# --------------------------------------------------------------------------------------------

TP_CORE = r"(?:[A-Z]\d{3}[A-Z]\d{2}|\d{3}[A-Z]\d{3}|[A-Z]\d{2}[A-Z]\d{3})"
DROP_TP = re.compile(r"(?<![A-Za-z0-9])(" + TP_CORE + r")(?![0-9])")
DROP_CAL = re.compile(r"(?<![A-Za-z0-9])(\d{2}CAL\d{2}|LCTF\d{3}|BKGD\d{3}|RDTST\d{2})")
DROP_FLEX = re.compile(r"(?<![A-Za-z])FLEX[-_ ]?(\d{3,4})(?!\d)")
DROP_BARE = re.compile(r"INTNS_(\d{3}[A-Z]\d{2})E_")

CAM_LETTER = {
    "A": "HiBMs camera (IPSU1)", "B": "LLL-UV camera (IPSU2)", "C": "LCTF or HFR camera (IPSU3)",
    "D": "colour camera (IPSU4)", "E": "intensified camera (IPSU-G5)",
}
CAM_SETTINGS = [
    (r"HI(BM|MB)S_CFI", "LCTF or HFR camera (IPSU3)"), (r"HI(BM|MB)S", "HiBMs camera (IPSU1)"),
    (r"LLLUV", "LLL-UV camera (IPSU2)"), (r"LLLIR", "LLL-IR camera (IPSU2)"), (r"LLL", "LLL camera (IPSU2)"),
    (r"LCTF", "LCTF or HFR camera (IPSU3)"), (r"HFR", "LCTF or HFR camera (IPSU3)"),
    (r"MDCA_Color", "MDCA colour camera (lens and exposure)"), (r"MDCA", "MDCA colour camera (lens and exposure)"),
    (r"Color", "colour camera (IPSU4)"), (r"INTNS", "intensified camera (IPSU-G5)"),
]


def cam_from_settings(m):
    tok = m.group("cam") or ""
    for rx, name in CAM_SETTINGS:
        if re.search(rx, tok, re.I):
            return name
    return None


def cam_from_letter(m):
    return CAM_LETTER.get((m.group("cam") or "").upper())


def view_instrument(m):
    s = m.group(0).lower()
    if "droplet" in s or "hibms" in s or "hims" in s:
        return "HiBMs camera (IPSU1)"
    if "mdca" in s:
        return "MDCA colour camera"
    if "flame" in s:
        return "flame-view camera"
    return None


TPX = r"(?:" + TP_CORE + r"|\d{2}CAL\d{2}|LCTF\d{3}|BKGD\d{3}|RDTST\d{2}|[A-Z]?\d{2,4}[A-Z]\d{2,3})"
DROPLET_RULES = [
    # sensor series
    Rule("droplet.chamber_readings", r"_Chamber_Readings\d?\.txt$", "sensor time series",
         "chamber pressures and temperatures against time", "CIR fluids control unit", OPEN),
    Rule("droplet.radiometer_readings", r"_Radiom(e)?ter_Readings\.(txt|xlsx)$", "sensor time series",
         "radiometers at 100 Hz with igniter state", "MDCA radiometers", OPEN),
    Rule("droplet.mixture", r"_(Mixture|Chamber_Gases|Chamber_Contents)\.txt$", "sensor time series",
         "chamber gas composition reading (one row per reading)", "gas chromatograph", OPEN, flags=re.I),
    Rule("droplet.filter_positions", r"_Filter_Positions\.txt$", "sensor time series",
         "sequence phase and radiometer filter positions against time", "MDCA", OPEN),
    # script and settings
    Rule("droplet.cool_flames", r"_Cool_Flames\.txt$", "script or settings",
         "cool-flame trigger settings of the insert", "MDCA", OPEN),
    Rule("droplet.testpoint_settings", r"_(LCTF_)?(MDCA_|MCA_)?(CFI_)?Testp\w+_settings\.(txt|log)$", "script or settings",
         "test point settings of the insert against time (needles, igniters, dispensers)", "MDCA", OPEN),
    Rule("droplet.lctf_wavelengths", r"_LCTF ?_Wavelen\w+ ?\.txt$", "camera settings",
         "tunable-filter wavelengths against time", "LCTF or HFR camera (IPSU3)", OPEN),
    Rule("droplet.camera_settings", r"_(?P<cam>[A-Za-z_]*?)_?Cam(a)?era_Set+ings\.(txt|log)$", "camera settings",
         "camera settings against time", cam_from_settings, OPEN),
    Rule("droplet.timeline", r"^" + TPX + r"\.(txt|docx)$", "script or settings",
         "test point timeline: sequence stamps and the image counts of each camera unit", None, OPEN),
    Rule("droplet.frame_index", r"^" + TPX + r"(?P<cam>[A-E])\.txt$", "metadata",
         "per-frame index of the archive (frame number, time stamp)", cam_from_letter, OPEN),
    Rule("droplet.frame_index_int", r"_INTNS_.*\.metadata$", "metadata",
         "per-frame index of the archive (time stamp, exposure, gain)", CAM_LETTER["E"], SIB),
    Rule("droplet.frame_meta", r"_INTNS_.*\.meta$", "metadata", "camera state at the first or last frame",
         CAM_LETTER["E"], OPEN),
    # notes
    Rule("droplet.comments", r"[ _]comments ?\d?\.txt$", "metadata", "operator notes on the image downlink of the test point", None, OPEN),
    Rule("droplet.readme", r"(_Readme|^Readme-|_MissingImageData|^filelist)\w*\.txt$", "metadata", "note about the files"),
    Rule("droplet.event_log", r"_Event_Log_entry.*\.(docx|jpg)$", "metadata", "event log entry of the test point", None, OPEN),
    Rule("droplet.test_sheet", r"_TestSheet\.(txt|rtf|pdf)$", "metadata",
         "test sheet: conditions, narrative and the investigators' derived values (outcome-bearing)", None, SIB),
    # image archives
    Rule("droplet.zip_camera", r"^" + TPX + r"(?P<cam>[A-E])(_IPSUG ?|_\d+)?\.zip$", "image archive",
         "frames of one test point", cam_from_letter, OPEN),
    Rule("droplet.zip_ipsu", r"^IPSU-?\w*( IMAGES)?( \(\d\))?\.zip$", "image archive", "frames by camera unit", "CIR IPSU camera"),
    Rule("droplet.zip_cal", r"^(LCTF\d{3}(_\d{4})?|BKGD\d{3}|" + TP_CORE + r")\.zip$", "image archive", "archive named only by the test point", None, UNK),
    Rule("droplet.settings_zip", r" Settings\.zip$", "script or settings", "settings files, zipped"),
    # processed
    Rule("droplet.droplet_data", r"_Droplet(Data|History)\.txt$", "processed or analysed data",
         "droplet diameter against time", "HiBMs camera (IPSU1)", SIB),
    Rule("droplet.uv_data", r"_LLUVData\.txt$", "processed or analysed data", "UV flame diameter against time",
         "LLL-UV camera (IPSU2)", SIB),
    Rule("droplet.color_data", r"_ColorCamData\.txt$", "processed or analysed data", "visible flame diameter against time",
         "colour camera", SIB),
    Rule("droplet.radiometer_data", r"_RadiometerData\.txt$", "processed or analysed data", "radiometer traces on the test clock",
         "MDCA radiometers"),
    Rule("droplet.coolflame_data", r"_CoolFlameData\.txt$", "processed or analysed data", "cool-flame diameter against time",
         "intensified camera (IPSU-G5)", OPEN),
    Rule("droplet.igor", r"\.pxp$", "processed or analysed data", "Igor experiment file with the processed series", None, SIB),
    Rule("droplet.summary", r"(_Summary|_ResultsSummary|_Results|_DropHist|_BurningHistory|_DropletProfiles|_FlameProfiles|_DLDanalysis|_DataFigure|PrelimSummary|_DualFlame|^FLEX-\d+)[-_ \w]*\.(pdf|eps|txt)$",
         "processed or analysed data", "summary sheet or figure of one test"),
    Rule("droplet.results_xls", r"Results\.xls$", "processed or analysed data", "results workbook"),
    Rule("droplet.sample_figure", r"(Sample\w*Image|CoolFlameImage|VaporCloud\w*|Soot_Image|Radial_Intensity|BacklitSample|Pressure_Temperature|_Radiometer|_Layout|^DropletData|Images_From_CFI)\.(pdf|pptx)$",
         "processed or analysed data", "sample figure"),
    # video
    Rule("droplet.video", r"(vlc-record.*|_ISS_.*|DOWNN?LINK[ -]VIDEO.*|^FLEX-?\d+[-_]\d{4,5}-.*)\.(mp4|ts|mpg)$", "video",
         "recording of the downlinked operations video", VID_OPS),
    Rule("droplet.video_sample", r"(View|Movie)[ _\w.]*\.mp4$", "video", "sample movie of one camera view", view_instrument),
    Rule("droplet.video_zip", r"(Movies|Original Videos)\.zip$", "video", "movies, zipped"),
]


def droplet_test(stem, folder, ctx=None):
    out = {}
    up = stem
    tp = DROP_TP.search(up) or DROP_TP.search(up.upper() if "vlc-record" in up else up)
    fx = DROP_FLEX.search(stem)
    cal = DROP_CAL.search(stem)
    if tp:
        out.update(test_id=tp.group(1).upper(), scheme="cir_test_point", source="name")
        if fx:
            out["others"] = ["FLEX-%03d" % int(fx.group(1))]
    elif cal:
        out.update(test_id=cal.group(1), scheme="cir_calibration_point", source="name")
    elif fx:
        out.update(test_id="FLEX-%03d" % int(fx.group(1)), scheme="flex_number", source="name")
    else:
        bare = DROP_BARE.search(stem)
        ftp = None
        for comp in reversed((folder or "").split("/")):
            fm = re.match(r"^\s*(" + TP_CORE + r")(_\d{4})?\s*$", comp)
            if fm:
                ftp = fm.group(1)
                break
            fm = re.match(r"^\s*FLEX[- ](" + TP_CORE + r")\s*$", comp)
            if fm:
                ftp = fm.group(1)
                break
        if ftp:
            out.update(test_id=ftp, scheme="cir_test_point", source="folder")
        elif bare:
            out.update(test_id="F" + bare.group(1), scheme="cir_test_point", source="name")
    tid = out.get("test_id", "")
    if out.get("scheme") == "cir_test_point" and re.match(r"^[A-Z]\d{3}R\d{2}$", tid):
        # type letter R in the later naming: a chamber gas reading (a Mixture file, no images)
        out["scheme"] = "cir_gas_reading_point"
    if out.get("scheme") in ("cir_test_point", "cir_gas_reading_point"):
        m = re.match(r"^[A-Z](\d{3})[A-Z]\d{2}$", tid) or re.match(r"^(\d{3})[A-Z]\d{3}$", tid)
        if m:
            out["day"] = m.group(1)
            out["day_scheme"] = "DDD (GMT day of year; the year is not in the name)"
    return out


# --------------------------------------------------------------------------------------------
# BASS-II and BASS
# --------------------------------------------------------------------------------------------

BASS2_RULES = [
    Rule("bass2.proof_sheet", r"_?ProofSheet\.pdf$", "image archive", "proof sheet of the still sequence (PDF)",
         "still camera", SIB),
    Rule("bass2.stills_iss", r"^iss\d{3}e\d+\.zip$", "image archive", "still sequence of the burn, named by ISS photo number",
         "still camera", SIB),
    Rule("bass2.stills_tp", r"^GMT\d{4}_\d{3}_BASS_.*\.zip$", "image archive", "still sequence of the burn", "still camera", SIB),
    Rule("bass2.video_stills", r"_Still_Sequence(_\d)?\.mp4$", "video", "movie made from the still sequence", "still camera"),
    Rule("bass2.video_second", r"_2ND_VIEW.*\.mp4$", "video", "video of the burn, second view", "video camera, second view"),
    Rule("bass2.video_zip", r"or\w+ vid\w+\.zip$", "video", "original videos of the burn, zipped", "video cameras"),
    Rule("bass2.zip_test", r"^BASS_GMT\d+_SAMPLE_\d+_TEST_[A-Z]\d+\.zip$", "video",
         "archive named like the video of the burn; the name does not say what it holds", None, UNK),
    Rule("bass2.video", r"\.mp4$", "video", "video of the burn", "video camera"),
]
BASS_RULES = [
    Rule("bass.video", r"^\d{8} .*iss\d{3}m\d*\.mp4$", "video", "video of one burn, named by date, time and ISS video number",
         "video camera"),
    Rule("bass.stills", r"^\d{4}_\d\d_\d\d(_orig)?\.zip$", "image archive", "original stills of the day", "still camera", SIB),
    Rule("bass.xmp", r"_xmp\.zip$", "metadata", "XMP sidecars of the day's stills, zipped"),
    Rule("bass.txt", r"_txt\.zip$", "metadata", "text sidecars of the day's stills, zipped"),
]


def bass2_test(stem, folder, ctx=None):
    out = {}
    m = (re.search(r"TEST[_ -]?([A-Z])(\d{1,2})([ab])?(?!\d)", stem) or re.search(r"TP([A-Z])(\d{1,2})([ab])?(?!\d)", stem)
         or re.search(r"test[ -]([A-Z])(\d{1,2})()", stem))
    ftest = None
    comps = [c.strip() for c in (folder or "").split("/")]
    if comps and re.match(r"^[A-Z]\d{1,2}$", comps[-1]):
        ftest = comps[-1]
    if m:
        out.update(test_id=m.group(1) + str(int(m.group(2))), scheme="bass2_test", source="name")
        if m.group(3):
            out["variant"] = m.group(3)
        if ftest and ftest != out["test_id"]:
            out["alternative"] = ftest
    elif ftest:
        out.update(test_id=ftest, scheme="bass2_test", source="folder")
    d = re.search(r"GMT(\d{4})_(\d{3})", stem)
    if d:
        out.update(day="%s-%s" % (d.group(1), d.group(2)), day_scheme="YYYY-DDD (GMT)")
    else:
        d = re.search(r"BASS_GMT(\d{2,3})_", stem)
        if d:
            out.update(day="2014-%03d" % int(d.group(1)), day_scheme="YYYY-DDD (GMT)")
    return out


def bass_test(stem, folder, ctx=None):
    out = {}
    m = re.search(r"(?<!\d)(20\d\d)_(\d\d)_(\d\d)", stem) or re.search(r"^(20\d\d)(\d\d)(\d\d) ", stem) \
        or re.search(r"(20\d\d)_(\d\d)_(\d\d)", folder or "")
    if m:
        out.update(day="%s-%s-%s" % m.groups(), day_scheme="YYYY-MM-DD")
    return out


# --------------------------------------------------------------------------------------------
# SAFFIRE I, II, III
# --------------------------------------------------------------------------------------------

SAFFIRE_RULES = [
    Rule("saffire.distortion", r"Distortion ?Grid", "calibration", "distortion grid images of one camera"),
    Rule("saffire.overlap", r"Overlap_verification", "calibration", "camera overlap verification images"),
    Rule("saffire.illustrative", r"Illustrative_images\.zip$", "image archive", "illustrative images"),
    Rule("saffire.img_png", r"(jpe?gs?2pn[gc]|png).*\.zip$", "image archive", "frames converted to PNG"),
    Rule("saffire.img_tiff_mapped", r"RAW2TIFF[-_ ]?Map\w*.*\.zip$", "image archive", "raw frames converted to TIFF and mapped"),
    Rule("saffire.img_tiff", r"RAW2TIFF.*\.zip$", "image archive", "raw frames converted to TIFF"),
    Rule("saffire.img_raw", r"(Raw_DistroMaster|Image_Data_RAWs|icmad).*\.zip$", "image archive", "raw frames as distributed"),
    Rule("saffire.img_jpeg_mapped", r"(JPE?GS?[-_ ]?Mapped|Mapped[-_ ]?jpe?g).*\.zip$", "image archive", "JPEG frames, mapped"),
    Rule("saffire.img_jpeg", r"jpe?gs?.*\.zip$", "image archive", "JPEG frames as downlinked"),
    Rule("saffire.telemetry_plots", r"(TM [Gg]raphs|Figure for TM)\.docx$", "processed or analysed data", "plots of the telemetry"),
    Rule("saffire.telemetry", r"(Sensor Data|Sensors|Telemetry Data|telemetry file|Saffire_Data_Table).*\.(xlsx|csv)$",
         "sensor time series", "sensor telemetry", "telemetry"),
    Rule("saffire.co2_plot", r"StitchedCO2", "processed or analysed data", "stitched CO2 plot"),
    Rule("saffire.tc_placement", r"(Thermocouple Placement|nonconformance|ImageFrameSizes)", "engineering document",
         "engineering note"),
]


def saffire_test(stem, folder, ctx=None):
    out = {}
    m = re.search(r"(?<![A-Za-z0-9])([SF])(\d{1,2})C(\d)(?!\d)", stem) or re.search(r"(?<![A-Za-z0-9])([SF])(\d{1,2})C(\d)(?!\d)", folder or "")
    if m:
        out.update(test_id=m.group(1) + m.group(2), scheme="saffire_sample" if m.group(1) == "S" else "saffire_flow_visualization",
                   source="name" if m.string == stem else "folder", instrument="camera %s" % m.group(3))
        return out
    m = re.search(r"Sample (\d)", stem)
    if m:
        out.update(test_id="S" + m.group(1), scheme="saffire_sample", source="name")
        return out
    m = re.search(r"(?:SAFFIRE_II_|Telemetry/|/)S(\d)(?: Data Package|$)", folder or "")
    if m:
        out.update(test_id="S" + m.group(1), scheme="saffire_sample", source="folder")
    return out


# --------------------------------------------------------------------------------------------
# SAME and SAME-R
# --------------------------------------------------------------------------------------------

SAME_RULES = [
    Rule("same.hkp_csv", r"^HKPData_gmt\d+_\d+.*\.csv$", "sensor time series",
         "housekeeping record of the run: instrument readings against time", "SAME avionics", OPEN),
    Rule("same.video", r"vlc-record.*\.mp4$", "video", "recording of the downlinked operations video", VID_OPS),
    Rule("same.video_zip", r"vlc-record.*\.zip$", "video", "the recording, zipped", VID_OPS),
    Rule("same.workbook", r"\.xlsx?$", "processed or analysed data", "run workbook (housekeeping and instrument data)", None, SIB),
    Rule("same.sample_zip", r"\.zip$", "image archive", "images of the run's particle sample (folder: Videos, Images)", None,
         folder=r"Videos, Images"),
    Rule("same.sample_tif", r"\.tif$", "image archive", "one image of the run's particle sample"),
    Rule("same.sample_slides", r"\.(pptx|pdf)$", "image archive", "images of the run's particle sample, as slides",
         None, SIB, folder=r"/Run \d"),
    Rule("same.note", r"\.docx?$", "science document", "note", None, UNK),
]


def same_test(stem, folder, ctx=None):
    out = {}
    fm = re.search(r"/Run (\d+)([a-z])?(?:, ?(\d+))?", folder or "")
    nm = re.search(r"run ?(\d+) ?([ab])?(?![a-z0-9])", stem, re.I) or re.search(r"^#? ?(\d{2,3})()(?=[ _-]|preage)", stem)
    if fm:
        out.update(test_id=str(int(fm.group(1))), scheme="same_test_number", source="folder")
        if fm.group(3):
            out["others"] = [str(int(fm.group(3)))]
        # the letter after the run number in the file name marks the aged repeat of the run
        if nm and int(nm.group(1)) == int(fm.group(1)) and nm.group(2):
            out["variant"] = nm.group(2).lower()
    elif nm:
        out.update(test_id=str(int(nm.group(1))), scheme="same_test_number", source="name")
        if nm.group(2):
            out["variant"] = nm.group(2).lower()
    d = re.search(r"(?<!\d)(20\d\d)_(\d{3})_", stem) or None
    g = re.search(r"gmt ?(\d{3})", stem, re.I)
    if d:
        out.update(day="%s-%s" % (d.group(1), d.group(2)), day_scheme="YYYY-DDD (GMT)")
    elif g:
        out.update(day=g.group(1), day_scheme="DDD (GMT day of year; the year is not in the name)")
    return out


# --------------------------------------------------------------------------------------------
# SLICE
# --------------------------------------------------------------------------------------------

SLICE_RULES = [
    Rule("slice.cal_folder", None, "calibration", "flow or boundary-condition calibration file (one `<n>.dat` opened: a binary array)",
         folder=r"(SLICE Coflow|SLICE Fuel Flow|Flame BC|Data Before Cleanmesh|Final After Cleanmesh)"),
    Rule("slice.cal_named", r"^(CoflowBC|SPICE vel|D300s_BBcal)\.xlsx$", "calibration", "calibration workbook"),
    Rule("slice.crew_procedure", r"^SLICE \d\.\d{3} .*\.pdf$", "engineering document", "crew procedure"),
    Rule("slice.engineering_doc", r"^(SLICE ED-\d+ |SLICE Drawings).*\.pdf$", "engineering document", "engineering document"),
    Rule("slice.infosheet", r"Infosheet\.pdf$", "science document", "information sheet"),
    Rule("slice.report_doc", r"Science Verif Report\.doc$", "report", "report"),
    Rule("slice.dat_color", r"^color_\d+_\d+\.dat$", "processed or analysed data", "colour plane of one picture (binary array; 'color' read from the name)", "Nikon D300s"),
    Rule("slice.dat_temperature", r"^T_\d+_(ave|bg|br)( - Copy| \(\d\))?\.dat$", "processed or analysed data",
         "temperature map of one picture (binary array; 'T' read as temperature)", "Nikon D300s"),
    Rule("slice.dat_soot", r"^fv_\d+\.dat$", "processed or analysed data", "soot volume fraction map of one picture (binary array; 'fv' read as soot volume fraction)",
         "Nikon D300s"),
    Rule("slice.dat_ch", r"^CH[-_](sig_)?\d+(_ ?txt)?\.dat$", "processed or analysed data", "CH* signal map of one picture (binary array; 'CH' read as CH*)",
         "Nikon D300s"),
    Rule("slice.dat_picture", r"^\d+\.dat$", "image archive", "one picture as a binary array", "Nikon D300s", SIB),
    Rule("slice.plot_pdf", r"^(\d+_\d+|\d+CH4_.*)\.pdf$", "processed or analysed data",
         "PDF named by a range of picture numbers, in a folder named pdf", None, UNK),
    Rule("slice.video_dims", r"^DIMSG\d+M_\d+\.mp4$", "video", "video named by an imagery id", "video camera"),
    Rule("slice.video_zip", r"^UT_\d{3}_\d{3}\.zip$", "video", "videos of the test days, zipped", "video camera", SIB),
    Rule("slice.video_hex", r"^[0-9a-f]{16}\.(mp4|mpg|zip)$", "video", "video named by a hexadecimal id", "video camera"),
    Rule("slice.nef", r"_NEF\.zip$", "image archive", "original stills of the day (Nikon raw)", "Nikon D300s"),
    Rule("slice.jpg_zip", r"(_jpg|^OT \d{3}.*|^RoomT_\d{3}|_Setup|Focus.*|DarkFrameCal|^OT_\d{3}.*|^GMT 2012 \d{3} D300s)\.zip$", "image archive",
         "stills, zipped", "Nikon D300s"),
    Rule("slice.proof_sheet", r"(Proof[ _]Sheet|Contact Sheet|Focus.*)\.pdf$", "image archive", "proof sheet of the day's stills (PDF)", "Nikon D300s"),
    Rule("slice.num", r"_(SLICE|SPICE)_num\.txt$", "metadata", "map from ISS photo number to camera file name", "Nikon D300s", OPEN),
    Rule("slice.pic_info", r"^pic_info_.*\.(csv|xls)$", "camera settings", "per-picture exposure record (number, time, shutter)",
         "Nikon D300s", OPEN),
    Rule("slice.duct_cal", r"(Flow Duct Cal|fan calibration|Fan_cal|meter cal|SLICE_vel|^Workbook\d)", "calibration", "flow calibration"),
    Rule("slice.matlab", r"\.(mat|eps|docx|csv)$", "processed or analysed data", "analysis script, constant or figure",
         folder=r"/SLICE 1g matlab"),
    Rule("slice.listing_docx", r"\.txt\.docx$", "metadata", "text file saved as a document, named by fuel and day", None, UNK),
    Rule("slice.pic_xls", r"\.(xls|xlsx|csv)$", "processed or analysed data", "picture table of a ground-test day", None, UNK,
         folder=r"/SLICE 1g \d{3}"),
]


def slice_test(stem, folder, ctx=None):
    out = {}
    s = (folder or "") + " :: " + stem
    for rx in (r"GMT ?2[-0]12_(\d{3})", r"(?:OT|RoomT)[ _](\d{3})", r"SLICE 1g (\d{3})", r"OMA Format/(?:00_)?(?:UT)?(\d{3})_",
               r"UT_?(\d{3})", r"CH4_(\d{3})", r"pic_info_(\d{3})"):
        m = re.search(rx, s, re.I)
        if m:
            out.update(day="2012-%s" % m.group(1), day_scheme="YYYY-DDD (GMT)")
            break
    return out


# --------------------------------------------------------------------------------------------
# SPICE
# --------------------------------------------------------------------------------------------

SPICE_RULES = [
    Rule("spice.calibration", None, "calibration", "flow calibration file", folder=r"Calibration Data"),
    Rule("spice.frame_grabs", r"(Frame Grab|vlcsnap).*\.zip$", "image archive", "frames grabbed from the downlinked video", "video camera"),
    Rule("spice.nef", r"(_NEF\.zip|\.nef)$", "image archive", "original stills (Nikon raw)", "Nikon still camera"),
    Rule("spice.jpegs", r"(jpe?gs?|Still Images|Test Point Images)\.zip$", "image archive", "stills, zipped", "Nikon still camera"),
    Rule("spice.contact_sheet", r"Contact ?_?Sheets?\.(pdf|zip)$", "image archive", "contact sheet of the day's stills", "Nikon still camera"),
    Rule("spice.picture_book", r"PictureBook\.ppt$", "image archive", "picture book of the day's stills (slides)", "Nikon still camera"),
    Rule("spice.video", r"\.(mp4|avi)$", "video", "video of the test session", "video camera"),
    Rule("spice.video_zip", r"Movie Files\.zip$", "video", "movies, zipped", "video camera"),
    Rule("spice.num", r"_num\.txt$", "metadata", "note on the day's still downlink", "Nikon still camera", OPEN),
    Rule("spice.nikon_xls", r"nikon\w*\.xls$", "processed or analysed data", "workbook named after the still camera", "Nikon still camera", UNK),
    Rule("spice.results", r"(Flight Test Data|Flamelength|SmokePointReview|smoke hie?ght)", "processed or analysed data",
         "results workbook or slide"),
]


def spice_test(stem, folder, ctx=None):
    out = {}
    m = re.search(r"SPICE Day (\d{3})", folder or "")
    if m:
        out.update(day="2009-%s" % m.group(1), day_scheme="YYYY-DDD (GMT)")
    t = re.search(r"TP ?-? ?0*(\d{1,3})([a-zA-Z])?", stem)
    if t:
        out.update(test_id="TP %s" % t.group(1), scheme="spice_test_point", source="name")
        if t.group(2):
            out["variant"] = t.group(2)
    return out


# --------------------------------------------------------------------------------------------
# DAFT
# --------------------------------------------------------------------------------------------

DAFT_RULES = [
    Rule("daft.dusttrak", r"^DTrk\d{4}\.txt$", "sensor time series", "instrument log of one test", "DustTrak", OPEN),
    Rule("daft.ptrak", r"^PTrk\d{4}\.txt$", "sensor time series", "instrument log of one test", "P-Trak", OPEN),
]


def daft_test(stem, folder, ctx=None):
    out = {}
    m = re.search(r"([DP]Trk)(\d{4})", stem)
    if m:
        out.update(test_id=m.group(1) + m.group(2), scheme="daft_instrument_test", source="name")
    d = re.search(r"(?:flight data |GMT)(\d{3})", stem)
    if d:
        out.update(day="2005-%s" % d.group(1), day_scheme="YYYY-DDD (GMT)")
    return out


FAMILY_RULES = {
    "acme": (ACME_RULES, acme_test), "droplet": (DROPLET_RULES, droplet_test), "bass2": (BASS2_RULES, bass2_test),
    "bass": (BASS_RULES, bass_test), "saffire": (SAFFIRE_RULES, saffire_test), "same": (SAME_RULES, same_test),
    "slice": (SLICE_RULES, slice_test), "spice": (SPICE_RULES, spice_test), "daft": (DAFT_RULES, daft_test),
}

RULE_BY_ID = {}
for _rules, _fn in FAMILY_RULES.values():
    for _r in _rules:
        assert _r.rid not in RULE_BY_ID, _r.rid
        RULE_BY_ID[_r.rid] = _r

# Where a file of the kind was opened, by this catalogue or by an earlier analysis of this lane
# (docs/wsp/atlas/FILES.md lists the file or the document for each entry). In those investigations
# the confidence is "checked by opening one file"; in the others that share the rule it is
# "inferred from a sibling investigation".
OPENED = {
    "acme.data_csv": ("PSI-10", "PSI-159"),
    "acme.mfc_setpoints": ("PSI-10", "PSI-159"),
    "acme.tp_txt": ("PSI-10", "PSI-159"),
    "acme.as_run": ("PSI-10", "PSI-159"),
    "acme.script_plain": ("PSI-10",),
    "acme.script_named": ("PSI-20", "PSI-22"),
    "acme.comments": ("PSI-10",),
    "acme.tp_analysis": ("PSI-21",),
    "acme.cam_exposures": ("PSI-21",),
    "acme.gc_asc": ("PSI-23",),
    "acme.gc_log": ("PSI-10",),
    "acme.frame_index_acme": ("PSI-22", "PSI-23"),
    "acme.frame_counter": ("PSI-23",),
    "acme.log_ipsug": ("PSI-23",),
    "acme.chamber_calculator": ("PSI-159",),
    "droplet.chamber_readings": ("PSI-39", "PSI-68"),
    "droplet.radiometer_readings": ("PSI-68",),
    "droplet.mixture": ("PSI-68",),
    "droplet.filter_positions": ("PSI-39",),
    "droplet.cool_flames": ("PSI-68",),
    "droplet.testpoint_settings": ("PSI-68",),
    "droplet.lctf_wavelengths": ("PSI-68",),
    "droplet.camera_settings": ("PSI-39", "PSI-68"),
    "droplet.timeline": ("PSI-39", "PSI-68"),
    "droplet.frame_index": ("PSI-39",),
    "droplet.frame_meta": ("PSI-39",),
    "droplet.comments": ("PSI-68",),
    "droplet.event_log": ("PSI-68",),
    "droplet.zip_camera": ("PSI-68",),
    "droplet.coolflame_data": ("PSI-39",),
    "droplet.droplet_data": ("PSI-69",),
    "droplet.uv_data": ("PSI-69",),
    "droplet.color_data": ("PSI-69",),
    "droplet.test_sheet": ("PSI-39", "PSI-69"),
    "droplet.igor": ("PSI-39",),
    "bass2.proof_sheet": ("PSI-25",),
    "bass2.stills_iss": ("PSI-25",),
    "bass2.stills_tp": ("PSI-25",),
    "bass2.video": ("PSI-25",),
    "bass.stills": ("PSI-26",),
    "same.hkp_csv": ("PSI-101",),
    "daft.dusttrak": ("PSI-47",),
    "daft.ptrak": ("PSI-47",),
    "slice.num": ("PSI-106",),
    "slice.pic_info": ("PSI-106",),
    "spice.num": ("PSI-107",),
    "table.campaign_spreadsheet": ("PSI-39",),
}


for _r in list(RULE_BY_ID.values()):
    assert _r.conf != OPEN or OPENED.get(_r.rid), "rule %s is marked opened but OPENED does not say where" % _r.rid
for _rid in OPENED:
    assert _rid in RULE_BY_ID or _rid.startswith("table."), _rid


def conf_for(rule, inv):
    if inv in OPENED.get(rule.rid, ()):
        return OPEN
    return SIB if rule.conf == OPEN else rule.conf


# category -> (kind, what)
CATEGORY_KIND = {
    "Investigation Metadata Files": ("metadata", "PSI investigation metadata archive"),
    "Experimental table": ("experiment table", "PSI experiment table"),
    "Engineering Documents": ("engineering document", "engineering document"),
    "Science Documents": ("science document", "science document"),
    "Reports": ("report", "report"),
    "Presentations": ("presentation", "presentation"),
    "Reference Data": ("reference data", "reference data"),
    "Representative Image(s)": ("other", "representative image of the investigation"),
    "Analyzed Data": ("processed or analysed data", "analysed data"),
    "Calibration": ("calibration", "calibration file"),
}
# named exceptions inside document categories
CATEGORY_EXCEPTIONS = [
    Rule("table.campaign_spreadsheet", r"DataSummary_Spreadsheet.*\.xlsx$", "experiment table",
         "campaign table of every test, wider than the PSI experiment table", None, OPEN),
    Rule("table.merge_result", r"MergeResult.*\.xlsx$", "metadata", "merge table of the image files", None, UNK),
    Rule("table.results_workbook", r"SAME, SAME-R_Results\.xlsx$", "processed or analysed data", "results workbook of both campaigns"),
]


IMAGE_EXT = {"jpg", "jpeg", "png", "tif", "tiff", "nef"}
VIDEO_EXT = {"mp4", "ts", "mpg", "avi"}
MEDIA = ("images", "video", "text, tables and documents")


def medium_of(kind, rule_id, ext):
    """What one downloads: images, video, or text, tables and documents."""
    r = RULE_BY_ID.get(rule_id.split("+")[-1])
    base_kind = r.kind if r else kind
    if base_kind == "video" or ext in VIDEO_EXT:
        return "video"
    if base_kind == "image archive" or ext in IMAGE_EXT:
        return "images"
    return "text, tables and documents"


def norm_stem(stem):
    return re.sub(r"[^a-z0-9]+", "", stem.lower())


def classify(inv, category, stem, folder, sibling_kinds):
    """Return (kind, what, instrument, confidence, rule id) for one listed file."""
    family = FAMILY[inv]
    rules, _ = FAMILY_RULES[family]
    r, inst = first(CATEGORY_EXCEPTIONS, stem, folder)
    if r:
        return r.kind, r.what, inst, conf_for(r, inv), r.rid
    if category in CATEGORY_KIND and category not in ("Calibration", "Analyzed Data", "Reference Data"):
        kind, what = CATEGORY_KIND[category]
        return kind, what, None, NAME, "category." + re.sub(r"[^a-z]+", "_", category.lower()).strip("_")
    fr, finst = first(rules, stem, folder)
    in_cal_folder = bool(re.search(r"Calibration", folder or "", re.I))
    if category == "Calibration" or (in_cal_folder and category == "Raw Data" and family == "droplet"):
        if fr:
            return "calibration", "calibration run: " + fr.what, finst, conf_for(fr, inv), "calibration+" + fr.rid
        return "calibration", "calibration file", None, NAME, "category.calibration"
    if category == "Analyzed Data":
        if fr and fr.kind == "processed or analysed data":
            return fr.kind, fr.what, finst, conf_for(fr, inv), fr.rid
        return "processed or analysed data", "analysed data", None, NAME, "category.analyzed_data"
    if category == "Reference Data":
        if fr and fr.kind in ("image archive", "video"):
            return "reference data", "reference: " + fr.what, finst, conf_for(fr, inv), "reference+" + fr.rid
        return "reference data", "reference data", None, NAME, "category.reference_data"
    # Raw Data (and any category this file does not know)
    if fr:
        return fr.kind, fr.what, finst, conf_for(fr, inv), fr.rid
    dr, dinst = first(DOC_RULES, stem, folder)
    if dr:
        return dr.kind, dr.what, dinst, dr.conf, dr.rid
    sk = sibling_kinds.get(norm_stem(stem))
    if sk:
        return sk, "document filed under another category in a sibling investigation", None, SIB, "doc.sibling_stem"
    return "other", "not classified", None, UNK, "unclassified"


# --------------------------------------------------------------------------------------------
# reading
# --------------------------------------------------------------------------------------------

def read_listing(inv):
    p = ROOT / INVESTIGATIONS[inv][0] / ("%s.listing.jsonl" % inv)
    rows = []
    with p.open(encoding="utf-8") as f:
        for n, line in enumerate(f, 1):
            if line.strip():
                r = json.loads(line)
                r["_row"] = n
                rows.append(r)
    return rows


def read_folders(inv):
    """psi_file_id -> folder path under the category, from the raw listing tree; {} when absent."""
    p = ROOT / INVESTIGATIONS[inv][1] / ("files_%s.json" % inv)
    if not p.exists():
        return {}
    tree = json.loads(p.read_text(encoding="utf-8"))
    out = {}

    def walk(node, path):
        for f in node.get("files", []):
            if f.get("directory") or f.get("files"):
                walk(f, path + [str(f.get("displayName") or "?")])
            else:
                out.setdefault(f.get("id"), "/".join(path))

    for top in tree.get("files", []):
        walk(top, [])
    return out


def group_of(folder, test_id):
    comps = [c.strip() for c in (folder or "").split("/") if c.strip()]
    if len(comps) < 2:
        return None
    g = comps[1]
    if test_id and g.startswith(test_id):
        return None
    if re.match(r"^\d{5}[A-Z]{1,2}\d", g) or re.match(r"^" + TP_CORE + r"(_\d{4})?$", g):
        return None
    return g


def build_rows():
    listings = {inv: read_listing(inv) for inv in ORDER}
    # kinds of document stems filed under a document category anywhere, for documents under Raw Data
    seen = defaultdict(set)
    for inv in ORDER:
        for r in listings[inv]:
            c = r["category"]
            if c in ("Engineering Documents", "Science Documents", "Reports", "Presentations"):
                seen[norm_stem(stem_of(inv, c, r["file_name"]))].add(CATEGORY_KIND[c][0])
    sibling_kinds = {k: sorted(v)[0] for k, v in seen.items() if len(v) == 1 and len(k) >= 8}

    rows = []
    for inv in ORDER:
        folders = read_folders(inv)
        _, test_fn = FAMILY_RULES[FAMILY[inv]]
        # how often each day appears in name-derived test ids, to judge mistyped names
        ctx = {"day_counts": Counter()}
        for r in listings[inv]:
            t0 = test_fn(stem_of(inv, r["category"], r["file_name"]), None, None)
            if t0.get("source") == "name" and t0.get("day"):
                ctx["day_counts"][t0["day"]] += 1
        first_row = {}
        latest = {}
        for r in listings[inv]:
            latest[r["file_name"]] = max(latest.get(r["file_name"], 0), int(r.get("file_version") or 0))
        for r in listings[inv]:
            cat = r["category"]
            stem = stem_of(inv, cat, r["file_name"])
            folder = folders.get(r["psi_file_id"])
            kind, what, inst, conf, rid = classify(inv, cat, stem, folder, sibling_kinds)
            t = test_fn(stem, folder, ctx) if cat in ("Raw Data", "Analyzed Data", "Calibration", "Reference Data") else {}
            if inst is None and t.get("instrument"):
                inst = t["instrument"]
            fid = r["psi_file_id"]
            rep = first_row.get(fid)
            if rep is None:
                first_row[fid] = r["_row"]
            rows.append({
                "investigation": inv,
                "listing_row": r["_row"],
                "file_name": r["file_name"],
                "category": cat,
                "folder": folder,
                "size_bytes": int(r["size_bytes"] or 0),
                "url": r["stable_url"],
                "psi_file_id": fid,
                "file_version": r.get("file_version"),
                "repeat_of_row": rep,
                "latest_version": int(r.get("file_version") or 0) == latest[r["file_name"]],
                "extension": ext_of(stem),
                "medium": medium_of(kind, rid, ext_of(stem)),
                "kind": kind,
                "what": what,
                "instrument": inst,
                "confidence": conf,
                "rule": rid,
                "test_id": t.get("test_id"),
                "test_id_scheme": t.get("scheme"),
                "test_id_source": t.get("source"),
                "test_id_variant": t.get("variant"),
                "test_ids_other": t.get("others"),
                "test_id_alternative": t.get("alternative"),
                "test_day": t.get("day"),
                "test_day_scheme": t.get("day_scheme"),
                "group": group_of(folder, t.get("test_id")),
            })
    return rows


# --------------------------------------------------------------------------------------------
# summary
# --------------------------------------------------------------------------------------------

def quantiles(values):
    v = sorted(values)
    if not v:
        return None

    def q(p):
        return v[min(len(v) - 1, int(p * (len(v) - 1) + 0.5))]

    return {"min": v[0], "p10": q(0.10), "median": q(0.50), "p90": q(0.90), "max": v[-1], "sum": sum(v)}


def summarize(rows):
    out = {"schema": "wsp-atlas-files-summary/1", "kinds": list(KINDS), "confidences": list(CONFIDENCES),
           "investigations": {}}
    tot = Counter()
    for inv in ORDER:
        rs = [r for r in rows if r["investigation"] == inv]
        distinct = [r for r in rs if r["repeat_of_row"] is None]
        by_kind = {}
        for k in KINDS:
            a = [r for r in rs if r["kind"] == k]
            if a:
                d = [r for r in a if r["repeat_of_row"] is None]
                by_kind[k] = {"files": len(a), "bytes": sum(r["size_bytes"] for r in a),
                              "distinct_files": len(d), "distinct_bytes": sum(r["size_bytes"] for r in d)}
        by_conf = {c: {"files": sum(1 for r in rs if r["confidence"] == c),
                       "bytes": sum(r["size_bytes"] for r in rs if r["confidence"] == c)} for c in CONFIDENCES}
        unclassified = [r for r in rs if r["rule"] == "unclassified"]
        # tests: files that carry a test id, one download per file name (latest version, no repeats)
        per_test = defaultdict(lambda: defaultdict(int))
        kinds_of_test = defaultdict(set)
        scheme_of = {}
        for r in distinct:
            if not r["test_id"] or not r["latest_version"]:
                continue
            key = (r["test_id_scheme"], r["test_id"])
            scheme_of[key] = r["test_id_scheme"]
            kinds_of_test[key].add(r["kind"])
            bucket = {"images": "image_bytes", "video": "video_bytes"}.get(r["medium"], "text_bytes")
            per_test[key][bucket] += r["size_bytes"]
            per_test[key]["files"] += 1
        schemes = {}
        for sch in sorted(set(scheme_of.values())):
            keys = [k for k in per_test if k[0] == sch]
            ent = {"is_test_scheme": sch in TEST_SCHEMES, "ids": len(keys)}
            for m in MEASUREMENTS:
                ent["with " + m] = sum(1 for k in keys if m in kinds_of_test[k])
            ent["with script or settings"] = sum(1 for k in keys if "script or settings" in kinds_of_test[k])
            ent["with sensor time series, image archive and video"] = sum(
                1 for k in keys if all(m in kinds_of_test[k] for m in MEASUREMENTS[:3]))
            for b in ("text_bytes", "image_bytes", "video_bytes"):
                ent["per_id_" + b] = quantiles([per_test[k][b] for k in keys])
            schemes[sch] = ent
        days = sorted({r["test_day"] for r in rs if r["test_day"]})
        kinds_of_day = defaultdict(set)
        for r in rs:
            if r["test_day"]:
                kinds_of_day[r["test_day"]].add(r["kind"])
        days_with = {"with " + m: sum(1 for d in days if m in kinds_of_day[d]) for m in MEASUREMENTS}
        with_id = [r for r in rs if r["test_id"]]
        # names that carry two ids at once link the two naming schemes of one test
        pairs = sorted({(r["test_id"], o) for r in rs if r["test_id"] for o in (r["test_ids_other"] or [])})
        out["investigations"][inv] = {
            "files": len(rs),
            "bytes": sum(r["size_bytes"] for r in rs),
            "distinct_files": len(distinct),
            "distinct_bytes": sum(r["size_bytes"] for r in distinct),
            "repeated_listing_rows": len(rs) - len(distinct),
            "superseded_versions": sum(1 for r in distinct if not r["latest_version"]),
            "folders_known": sum(1 for r in rs if r["folder"] is not None),
            "by_kind": by_kind,
            "by_medium": {md: {"files": sum(1 for r in rs if r["medium"] == md),
                               "bytes": sum(r["size_bytes"] for r in rs if r["medium"] == md)} for md in MEDIA},
            "by_confidence": by_conf,
            "unclassified": {"files": len(unclassified), "bytes": sum(r["size_bytes"] for r in unclassified)},
            "classified_share_of_files": round(1 - len(unclassified) / len(rs), 6) if rs else None,
            "classified_share_of_bytes": round(1 - sum(r["size_bytes"] for r in unclassified) / max(1, sum(r["size_bytes"] for r in rs)), 9),
            "files_with_test_id": len(with_id),
            "bytes_with_test_id": sum(r["size_bytes"] for r in with_id),
            "files_with_test_day": sum(1 for r in rs if r["test_day"]),
            "test_days": len(days),
            "test_days_with": days_with,
            "test_id_from_folder_only": sum(1 for r in with_id if r["test_id_source"] == "folder"),
            "name_and_folder_disagree": sum(1 for r in rs if r["test_id_alternative"]),
            "ids_linked_by_a_file_name": len(pairs),
            "id_schemes": schemes,
        }
        tot["files"] += len(rs)
        tot["bytes"] += sum(r["size_bytes"] for r in rs)
        tot["distinct_files"] += len(distinct)
        tot["distinct_bytes"] += sum(r["size_bytes"] for r in distinct)
        tot["unclassified_files"] += len(unclassified)
        tot["unclassified_bytes"] += sum(r["size_bytes"] for r in unclassified)
    out["total"] = dict(sorted(tot.items()))
    out["total"]["by_medium"] = {}
    for md in MEDIA:
        a = [r for r in rows if r["medium"] == md]
        out["total"]["by_medium"][md] = {"files": len(a), "bytes": sum(r["size_bytes"] for r in a)}
    out["total"]["by_kind"] = {}
    for k in KINDS:
        a = [r for r in rows if r["kind"] == k]
        out["total"]["by_kind"][k] = {"files": len(a), "bytes": sum(r["size_bytes"] for r in a)}
    return out


def write(rows, summary, out_dir):
    out_dir.mkdir(parents=True, exist_ok=True)
    with (out_dir / "files.jsonl").open("w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False, sort_keys=True) + "\n")
    with (out_dir / "files_summary.json").open("w", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(summary, ensure_ascii=False, indent=1) + "\n")


# --------------------------------------------------------------------------------------------
# tables for docs/wsp/atlas/FILES.md
# --------------------------------------------------------------------------------------------

def human(n):
    for unit, size in (("TB", 1e12), ("GB", 1e9), ("MB", 1e6), ("KB", 1e3)):
        if n >= size:
            return "%.1f %s" % (n / size, unit)
    return "%d B" % n


SHORT = {NAME: "name", SIB: "sibling", OPEN: "opened", UNK: "unknown"}


def name_shape(row):
    st = stem_of(row["investigation"], row["category"], row["file_name"])
    for tok in [row["test_id"]] + (row["test_ids_other"] or []) + [row["test_id_alternative"]]:
        if tok and tok in st:
            st = st.replace(tok, "<test>")
    st = re.sub(r"FLEX[-_ ]?\d{3,4}", "<test>", st)
    return re.sub(r"\d+", "#", st)


def tables_markdown(rows):
    """One table per investigation: file kinds with patterns, counts, bytes and confidence."""
    out = []
    for inv in ORDER:
        rs = [r for r in rows if r["investigation"] == inv]
        groups = defaultdict(list)
        for r in rs:
            groups[(KINDS.index(r["kind"]), r["what"], r["instrument"] or "", r["confidence"])].append(r)
        out.append("<!-- table %s -->" % inv)
        out.append("| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |")
        out.append("|---|---|---|---|---:|---:|---|")
        for key in sorted(groups, key=lambda k: (k[0], -sum(r["size_bytes"] for r in groups[k]), k[1], k[2], k[3])):
            g = groups[key]
            if g[0]["rule"].startswith("category."):
                pat = "every file of the PSI category \"%s\"" % g[0]["category"]
            else:
                shapes = Counter(name_shape(r) for r in g)
                top = sorted(shapes, key=lambda k: (-shapes[k], k))[:2]
                pat = " ; ".join("`%s`" % t.replace("|", "/").replace("`", "'") for t in top)
                if len(shapes) > 2:
                    pat += " and %d more spellings" % (len(shapes) - 2)
            out.append("| %s | %s | %s | %s | %d | %s | %s |" % (
                pat, KINDS[key[0]], key[1], key[2] or "-", len(g), human(sum(r["size_bytes"] for r in g)), SHORT[key[3]]))
        out.append("")
    return "\n".join(out) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--out", default=str(ROOT / "runs/dev/data_all/work/atlas"))
    ap.add_argument("--tables", action="store_true",
                    help="also write files_tables.md, the per-investigation tables of docs/wsp/atlas/FILES.md")
    a = ap.parse_args(argv)
    rows = build_rows()
    summary = summarize(rows)
    write(rows, summary, Path(a.out))
    if a.tables:
        with (Path(a.out) / "files_tables.md").open("w", encoding="utf-8", newline="\n") as f:
            f.write(tables_markdown(rows))
    t = summary["total"]
    print("files %d  bytes %d  distinct files %d  unclassified %d files, %d bytes -> %s" % (
        t["files"], t["bytes"], t["distinct_files"], t["unclassified_files"], t["unclassified_bytes"], a.out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
