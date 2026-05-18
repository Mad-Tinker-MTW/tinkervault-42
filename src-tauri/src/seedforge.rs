use crate::errors::{Result, VaultError};
use crate::geoastro::{alt_az, normalize_date, normalize_sky_time, resolve_place_lock};
use crate::nautical57::find_star;
use sha2::{Digest, Sha256};

const PRINTABLE95: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

pub struct ForgeInput {
    pub seed: String,
    pub place: String,
    pub date: String,
    pub sky_time: String,
    pub star: String,
}

pub struct ForgedSeed {
    pub material: String,
}

pub fn forge_seed(input: ForgeInput) -> Result<ForgedSeed> {
    let seed = normalize_seed(&input.seed);
    if seed.chars().count() < 3 {
        return Err(VaultError::Message("Seed phrase must be at least 3 characters.".into()));
    }

    let place = resolve_place_lock(&input.place)?;
    let date = normalize_date(&input.date)?;
    let sky_time = normalize_sky_time(&input.sky_time)?;
    let star = find_star(&input.star)
        .ok_or_else(|| VaultError::Message(format!("Unknown Nautical-57 star: {}", input.star)))?;

    let poly = polyglyph95(&seed, &place.label, place.lat, place.lon, &date, &sky_time, star.id)?;
    let (a, b, c) = split_three(&poly);
    let ga = glyph5(&a, &place.label, place.lat, place.lon, &date, &sky_time, star.id, "A");
    let gb = glyph5(&b, &place.label, place.lat, place.lon, &date, &sky_time, star.id, "B");
    let gc = glyph5(&c, &place.label, place.lat, place.lon, &date, &sky_time, star.id, "C");
    let (alt, az) = alt_az(&place, &date, &sky_time, star)?;

    let material = format!(
        "TVLT42-FORGE::{}@{:.6},{:.6}::{}::{}::{}::{}::{}::{}::GeoAstroLock42-Nautical57::ALT{}::AZ{}",
        place.label,
        place.lat,
        place.lon,
        date,
        ga,
        sky_time,
        gb,
        star.id,
        gc,
        alt,
        az
    );

    Ok(ForgedSeed { material })
}

fn normalize_seed(seed: &str) -> String {
    seed.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn polyglyph95(
    seed: &str,
    place_label: &str,
    lat: f64,
    lon: f64,
    date: &str,
    sky_time: &str,
    star_id: &str,
) -> Result<String> {
    let control = sha256_hex(&format!(
        "PolyGlyph95-42|{}|{}|{:.6},{:.6}|{}|{}|{}|GeoAstroLock42-Nautical57",
        seed, place_label, lat, lon, date, sky_time, star_id
    ));

    let chars: Vec<char> = PRINTABLE95.chars().collect();
    let mut out = String::new();

    for (pos, ch) in seed.chars().enumerate() {
        let idx = chars
            .iter()
            .position(|c| *c == ch)
            .ok_or_else(|| VaultError::Message(format!("Unsupported seed character: {:?}", ch)))?;

        let shuffled = deterministic_shuffle(&chars, &format!("{}|pos={}", control, pos + 1));
        out.push(shuffled[idx]);
    }

    Ok(out)
}

fn deterministic_shuffle(chars: &[char], control: &str) -> Vec<char> {
    let mut arr = chars.to_vec();

    for i in (1..arr.len()).rev() {
        let digest = Sha256::digest(format!("{}|{}", control, i).as_bytes());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[0..8]);
        let j = (u64::from_be_bytes(bytes) as usize) % (i + 1);
        arr.swap(i, j);
    }

    arr
}

fn glyph5(
    text: &str,
    place_label: &str,
    lat: f64,
    lon: f64,
    date: &str,
    sky_time: &str,
    star_id: &str,
    section: &str,
) -> String {
    let control = format!(
        "PolyGlyph95-42|Glyph5|{}|{:.6},{:.6}|{}|{}|{}|{}",
        place_label, lat, lon, date, sky_time, star_id, section
    );

    let mut out = String::new();
    for (pos, ch) in text.chars().enumerate() {
        let h = sha256_hex(&format!("{}|{}|{}|{}", control, pos + 1, ch as u32, ch));
        out.push_str(&h[..5]);
    }
    out
}

fn split_three(text: &str) -> (String, String, String) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let a = std::cmp::max(1, n / 3);
    let b = std::cmp::max(a + 1, (2 * n) / 3);
    (
        chars[0..a].iter().collect(),
        chars[a..b].iter().collect(),
        chars[b..].iter().collect(),
    )
}

fn sha256_hex(s: &str) -> String {
    let d = Sha256::digest(s.as_bytes());
    d.iter().map(|b| format!("{:02x}", b)).collect()
}
