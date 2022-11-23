use std::collections::BTreeSet;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use pica_path::{Path, PathExt};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::{Lint, Status};

#[derive(Debug, Deserialize)]
pub struct Iso639 {
    path: Path,
}

macro_rules! codes {
    ($($x:expr),+ $(,)?) => (
        vec![$($x.as_bytes().to_vec()),*]
    );
}

static ISO639_CODES: Lazy<Mutex<BTreeSet<Vec<u8>>>> = Lazy::new(|| {
    let codes = BTreeSet::from_iter(codes![
        "aar", "abk", "ace", "ach", "ada", "ady", "afa", "afh", "afr",
        "ain", "aka", "akk", "alb", "ale", "alg", "alt", "amh", "ang",
        "anp", "apa", "ara", "arc", "arg", "arm", "arn", "arp", "art",
        "arw", "asm", "ast", "ath", "aus", "ava", "ave", "awa", "aym",
        "aze", "bad", "bai", "bak", "bal", "bam", "ban", "baq", "bas",
        "bat", "bej", "bel", "bem", "ben", "ber", "bho", "bih", "bik",
        "bin", "bis", "bla", "bnt", "tib", "bos", "bra", "bre", "btk",
        "bua", "bug", "bul", "bur", "byn", "cad", "cai", "car", "cat",
        "cau", "ceb", "cel", "cze", "cha", "chb", "che", "chg", "chi",
        "chk", "chm", "chn", "cho", "chp", "chr", "chu", "chv", "chy",
        "cmc", "cnr", "cop", "cor", "cos", "cpe", "cpf", "cpp", "cre",
        "crh", "crp", "csb", "cus", "wel", "cze", "dak", "dan", "dar",
        "day", "del", "den", "ger", "dgr", "din", "div", "doi", "dra",
        "dsb", "dua", "dum", "dut", "dyu", "dzo", "efi", "egy", "eka",
        "gre", "elx", "eng", "enm", "epo", "est", "baq", "ewe", "ewo",
        "fan", "fao", "per", "fat", "fij", "fil", "fin", "fiu", "fon",
        "fre", "fre", "frm", "fro", "frr", "frs", "fry", "ful", "fur",
        "gaa", "gay", "gba", "gem", "geo", "ger", "gez", "gil", "gla",
        "gle", "glg", "glv", "gmh", "goh", "gon", "gor", "got", "grb",
        "grc", "gre", "grn", "gsw", "guj", "gwi", "hai", "hat", "hau",
        "haw", "heb", "her", "hil", "him", "hin", "hit", "hmn", "hmo",
        "hrv", "hsb", "hun", "hup", "arm", "iba", "ibo", "ice", "ido",
        "iii", "ijo", "iku", "ile", "ilo", "ina", "inc", "ind", "ine",
        "inh", "ipk", "ira", "iro", "ice", "ita", "jav", "jbo", "jpn",
        "jpr", "jrb", "kaa", "kab", "kac", "kal", "kam", "kan", "kar",
        "kas", "geo", "kau", "kaw", "kaz", "kbd", "kha", "khi", "khm",
        "kho", "kik", "kin", "kir", "kmb", "kok", "kom", "kon", "kor",
        "kos", "kpe", "krc", "krl", "kro", "kru", "kua", "kum", "kur",
        "kut", "lad", "lah", "lam", "lao", "lat", "lav", "lez", "lim",
        "lin", "lit", "lol", "loz", "ltz", "lua", "lub", "lug", "lui",
        "lun", "luo", "lus", "mac", "mad", "mag", "mah", "mai", "mak",
        "mal", "man", "mao", "map", "mar", "mas", "may", "mdf", "mdr",
        "men", "mga", "mic", "min", "mis", "mac", "mkh", "mlg", "mlt",
        "mnc", "mni", "mno", "moh", "mon", "mos", "mao", "may", "mul",
        "mun", "mus", "mwl", "mwr", "bur", "myn", "myv", "nah", "nai",
        "nap", "nau", "nav", "nbl", "nde", "ndo", "nds", "nep", "new",
        "nia", "nic", "niu", "dut", "nno", "nob", "nog", "non", "nor",
        "nqo", "nso", "nub", "nwc", "nya", "nym", "nyn", "nyo", "nzi",
        "oci", "oji", "ori", "orm", "osa", "oss", "ota", "oto", "paa",
        "pag", "pal", "pam", "pan", "pap", "pau", "peo", "per", "phi",
        "phn", "pli", "pol", "pon", "por", "pra", "pro", "pus", "que",
        "raj", "rap", "rar", "roa", "roh", "rom", "rum", "rum", "run",
        "rup", "rus", "sad", "sag", "sah", "sai", "sal", "sam", "san",
        "sas", "sat", "scn", "sco", "sel", "sem", "sga", "sgn", "shn",
        "sid", "sin", "sio", "sit", "sla", "slo", "slo", "slv", "sma",
        "sme", "smi", "smj", "smn", "smo", "sms", "sna", "snd", "snk",
        "sog", "som", "son", "sot", "spa", "alb", "srd", "srn", "srp",
        "srr", "ssa", "ssw", "suk", "sun", "sus", "sux", "swa", "swe",
        "syc", "syr", "tah", "tai", "tam", "tat", "tel", "tem", "ter",
        "tet", "tgk", "tgl", "tha", "tib", "tig", "tir", "tiv", "tkl",
        "tlh", "tli", "tmh", "tog", "ton", "tpi", "tsi", "tsn", "tso",
        "tuk", "tum", "tup", "tur", "tut", "tvl", "twi", "tyv", "udm",
        "uga", "uig", "ukr", "umb", "und", "urd", "uzb", "vai", "ven",
        "vie", "vol", "vot", "wak", "wal", "war", "was", "wel", "wen",
        "wln", "wol", "xal", "xho", "yao", "yap", "yid", "yor", "ypk",
        "zap", "zbl", "zen", "zgh", "zha", "chi", "znd", "zul", "zun",
        "zxx", "zza",
    ]);

    Mutex::new(codes)
});

impl Lint for Iso639 {
    fn check(&mut self, record: &ByteRecord) -> Status {
        let codes = ISO639_CODES.lock().unwrap();
        record
            .path(&self.path)
            .iter()
            .any(|value| !codes.contains(&value.to_vec()))
            .into()
    }
}
