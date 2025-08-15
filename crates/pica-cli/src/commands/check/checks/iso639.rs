use std::sync::LazyLock;

use bstr::ByteSlice;
use hashbrown::HashSet;
use pica_record::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Iso639 {
    path: Path,

    #[serde(default = "super::strsim_threshold")]
    strsim_threshold: f64,

    #[serde(default)]
    case_ignore: bool,
}

macro_rules! codes {
    ($($x:expr),+ $(,)?) => (
        vec![$($x.as_bytes().to_vec()),*]
    );
}

static CODES: LazyLock<HashSet<Vec<u8>>> = LazyLock::new(|| {
    HashSet::from_iter(codes![
        "aar", "abk", "ace", "ach", "ada", "ady", "afa", "afh", "afr",
        "ain", "aka", "akk", "alb", "ale", "alg", "alt", "amh", "ang",
        "anp", "apa", "ara", "arc", "arg", "arm", "arn", "arp", "art",
        "arw", "asm", "ast", "ath", "aus", "ava", "ave", "awa", "aym",
        "aze", "bad", "bai", "bak", "bal", "bam", "ban", "baq", "bas",
        "bat", "bej", "bel", "bem", "ben", "ber", "bho", "bih", "bik",
        "bin", "bis", "bla", "bnt", "bos", "bra", "bre", "btk", "bua",
        "bug", "bul", "bur", "byn", "cad", "cai", "car", "cat", "cau",
        "ceb", "cel", "cha", "chb", "che", "chg", "chi", "chk", "chm",
        "chn", "cho", "chp", "chr", "chu", "chv", "chy", "cmc", "cnr",
        "cop", "cor", "cos", "cpe", "cpf", "cpp", "cre", "crh", "crp",
        "csb", "cus", "cze", "dak", "dan", "dar", "day", "del", "den",
        "dgr", "din", "div", "doi", "dra", "dsb", "dua", "dum", "dut",
        "dyu", "dzo", "efi", "egy", "eka", "elx", "eng", "enm", "epo",
        "est", "ewe", "ewo", "fan", "fao", "fat", "fij", "fil", "fin",
        "fiu", "fon", "fre", "frm", "fro", "frr", "frs", "fry", "ful",
        "fur", "gaa", "gay", "gba", "gem", "geo", "ger", "gez", "gil",
        "gla", "gle", "glg", "glv", "gmh", "goh", "gon", "gor", "got",
        "grb", "grc", "gre", "grn", "gsw", "guj", "gwi", "hai", "hat",
        "hau", "haw", "heb", "her", "hil", "him", "hin", "hit", "hmn",
        "hmo", "hrv", "hsb", "hun", "hup", "iba", "ibo", "ice", "ido",
        "iii", "ijo", "iku", "ile", "ilo", "ina", "inc", "ind", "ine",
        "inh", "ipk", "ira", "iro", "ita", "jav", "jbo", "jpn", "jpr",
        "jrb", "kaa", "kab", "kac", "kal", "kam", "kan", "kar", "kas",
        "kau", "kaw", "kaz", "kbd", "kha", "khi", "khm", "kho", "kik",
        "kin", "kir", "kmb", "kok", "kom", "kon", "kor", "kos", "kpe",
        "krc", "krl", "kro", "kru", "kua", "kum", "kur", "kut", "lad",
        "lah", "lam", "lao", "lat", "lav", "lez", "lim", "lin", "lit",
        "lol", "loz", "ltz", "lua", "lub", "lug", "lui", "lun", "luo",
        "lus", "mac", "mad", "mag", "mah", "mai", "mak", "mal", "man",
        "mao", "map", "mar", "mas", "may", "mdf", "mdr", "men", "mga",
        "mic", "min", "mis", "mkh", "mlg", "mlt", "mnc", "mni", "mno",
        "moh", "mon", "mos", "mul", "mun", "mus", "mwl", "mwr", "myn",
        "myv", "nah", "nai", "nap", "nau", "nav", "nbl", "nde", "ndo",
        "nds", "nep", "new", "nia", "nic", "niu", "nno", "nob", "nog",
        "non", "nor", "nqo", "nso", "nub", "nwc", "nya", "nym", "nyn",
        "nyo", "nzi", "oci", "oji", "ori", "orm", "osa", "oss", "ota",
        "oto", "paa", "pag", "pal", "pam", "pan", "pap", "pau", "peo",
        "per", "phi", "phn", "pli", "pol", "pon", "por", "pra", "pro",
        "pus", "que", "raj", "rap", "rar", "roa", "roh", "rom", "rum",
        "run", "rup", "rus", "sad", "sag", "sah", "sai", "sal", "sam",
        "san", "sas", "sat", "scn", "sco", "sel", "sem", "sga", "sgn",
        "shn", "sid", "sin", "sio", "sit", "sla", "slo", "slv", "sma",
        "sme", "smi", "smj", "smn", "smo", "sms", "sna", "snd", "snk",
        "sog", "som", "son", "sot", "spa", "srd", "srn", "srp", "srr",
        "ssa", "ssw", "suk", "sun", "sus", "sux", "swa", "swe", "syc",
        "syr", "tah", "tai", "tam", "tat", "tel", "tem", "ter", "tet",
        "tgk", "tgl", "tha", "tib", "tig", "tir", "tiv", "tkl", "tlh",
        "tli", "tmh", "tog", "ton", "tpi", "tsi", "tsn", "tso", "tuk",
        "tum", "tup", "tur", "tut", "tvl", "twi", "tyv", "udm", "uga",
        "uig", "ukr", "umb", "und", "urd", "uzb", "vai", "ven", "vie",
        "vol", "vot", "wak", "wal", "war", "was", "wel", "wen", "wln",
        "wol", "xal", "xho", "yao", "yap", "yid", "yor", "ypk", "zap",
        "zbl", "zen", "zgh", "zha", "znd", "zul", "zun", "zxx", "zza",
    ])
});

impl Iso639 {
    pub(crate) fn check(
        &self,
        record: &ByteRecord,
    ) -> (bool, Option<String>) {
        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold)
            .case_ignore(self.case_ignore);

        let message = record
            .path(&self.path, &options)
            .filter(|value| !CODES.contains(value.as_bstr()))
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        if !message.is_empty() {
            (true, Some(message))
        } else {
            (false, None)
        }
    }
}
