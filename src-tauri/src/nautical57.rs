#[derive(Clone, Copy)]
pub struct StarEntry {
    pub id: &'static str,
    pub display: &'static str,
    pub scientific: &'static str,
    pub ra_hours: f64,
    pub dec_deg: f64,
}

pub const STARS: &[StarEntry] = &[
    StarEntry { id: "NAV57_ACAMAR", display: "Acamar", scientific: "Theta Eridani", ra_hours: 2.971, dec_deg: -40.304 },
    StarEntry { id: "NAV57_ACHERNAR", display: "Achernar", scientific: "Alpha Eridani", ra_hours: 1.629, dec_deg: -57.237 },
    StarEntry { id: "NAV57_ACRUX", display: "Acrux", scientific: "Alpha Crucis", ra_hours: 12.443, dec_deg: -63.099 },
    StarEntry { id: "NAV57_ADHARA", display: "Adhara", scientific: "Epsilon Canis Majoris", ra_hours: 6.977, dec_deg: -28.972 },
    StarEntry { id: "NAV57_ALDEBARAN", display: "Aldebaran", scientific: "Alpha Tauri", ra_hours: 4.599, dec_deg: 16.509 },
    StarEntry { id: "NAV57_ALIOTH", display: "Alioth", scientific: "Epsilon Ursae Majoris", ra_hours: 12.900, dec_deg: 55.960 },
    StarEntry { id: "NAV57_ALKAID", display: "Alkaid", scientific: "Eta Ursae Majoris", ra_hours: 13.792, dec_deg: 49.313 },
    StarEntry { id: "NAV57_ALNAIR", display: "Al Na'ir", scientific: "Alpha Gruis", ra_hours: 22.137, dec_deg: -46.961 },
    StarEntry { id: "NAV57_ALNILAM", display: "Alnilam", scientific: "Epsilon Orionis", ra_hours: 5.604, dec_deg: -1.202 },
    StarEntry { id: "NAV57_ALPHARD", display: "Alphard", scientific: "Alpha Hydrae", ra_hours: 9.460, dec_deg: -8.659 },
    StarEntry { id: "NAV57_ALPHECCA", display: "Alphecca", scientific: "Alpha Coronae Borealis", ra_hours: 15.579, dec_deg: 26.714 },
    StarEntry { id: "NAV57_ALPHERATZ", display: "Alpheratz", scientific: "Alpha Andromedae", ra_hours: 0.140, dec_deg: 29.090 },
    StarEntry { id: "NAV57_ALTAIR", display: "Altair", scientific: "Alpha Aquilae", ra_hours: 19.846, dec_deg: 8.868 },
    StarEntry { id: "NAV57_ANKAA", display: "Ankaa", scientific: "Alpha Phoenicis", ra_hours: 0.438, dec_deg: -42.306 },
    StarEntry { id: "NAV57_ANTARES", display: "Antares", scientific: "Alpha Scorpii", ra_hours: 16.490, dec_deg: -26.432 },
    StarEntry { id: "NAV57_ARCTURUS", display: "Arcturus", scientific: "Alpha Boötis", ra_hours: 14.261, dec_deg: 19.182 },
    StarEntry { id: "NAV57_ATRIA", display: "Atria", scientific: "Alpha Trianguli Australis", ra_hours: 16.811, dec_deg: -69.028 },
    StarEntry { id: "NAV57_AVIOR", display: "Avior", scientific: "Epsilon Carinae", ra_hours: 8.375, dec_deg: -59.509 },
    StarEntry { id: "NAV57_BELLATRIX", display: "Bellatrix", scientific: "Gamma Orionis", ra_hours: 5.419, dec_deg: 6.350 },
    StarEntry { id: "NAV57_BETELGEUSE", display: "Betelgeuse", scientific: "Alpha Orionis", ra_hours: 5.920, dec_deg: 7.407 },
    StarEntry { id: "NAV57_CANOPUS", display: "Canopus", scientific: "Alpha Carinae", ra_hours: 6.399, dec_deg: -52.696 },
    StarEntry { id: "NAV57_CAPELLA", display: "Capella", scientific: "Alpha Aurigae", ra_hours: 5.278, dec_deg: 45.998 },
    StarEntry { id: "NAV57_DENEB", display: "Deneb", scientific: "Alpha Cygni", ra_hours: 20.691, dec_deg: 45.280 },
    StarEntry { id: "NAV57_DENEBOLA", display: "Denebola", scientific: "Beta Leonis", ra_hours: 11.817, dec_deg: 14.572 },
    StarEntry { id: "NAV57_DIPHDA", display: "Diphda", scientific: "Beta Ceti", ra_hours: 0.726, dec_deg: -17.987 },
    StarEntry { id: "NAV57_DUBHE", display: "Dubhe", scientific: "Alpha Ursae Majoris", ra_hours: 11.062, dec_deg: 61.751 },
    StarEntry { id: "NAV57_ELNATH", display: "Elnath", scientific: "Beta Tauri", ra_hours: 5.438, dec_deg: 28.607 },
    StarEntry { id: "NAV57_ELTANIN", display: "Eltanin", scientific: "Gamma Draconis", ra_hours: 17.943, dec_deg: 51.489 },
    StarEntry { id: "NAV57_ENIF", display: "Enif", scientific: "Epsilon Pegasi", ra_hours: 21.736, dec_deg: 9.875 },
    StarEntry { id: "NAV57_FOMALHAUT", display: "Fomalhaut", scientific: "Alpha Piscis Austrini", ra_hours: 22.961, dec_deg: -29.622 },
    StarEntry { id: "NAV57_GACRUX", display: "Gacrux", scientific: "Gamma Crucis", ra_hours: 12.519, dec_deg: -57.113 },
    StarEntry { id: "NAV57_GIENAH", display: "Gienah", scientific: "Gamma Corvi", ra_hours: 12.263, dec_deg: -17.542 },
    StarEntry { id: "NAV57_HADAR", display: "Hadar", scientific: "Beta Centauri", ra_hours: 14.064, dec_deg: -60.373 },
    StarEntry { id: "NAV57_HAMAL", display: "Hamal", scientific: "Alpha Arietis", ra_hours: 2.120, dec_deg: 23.462 },
    StarEntry { id: "NAV57_KAUS_AUSTRALIS", display: "Kaus Australis", scientific: "Epsilon Sagittarii", ra_hours: 18.403, dec_deg: -34.385 },
    StarEntry { id: "NAV57_KOCHAB", display: "Kochab", scientific: "Beta Ursae Minoris", ra_hours: 14.845, dec_deg: 74.155 },
    StarEntry { id: "NAV57_MARKAB", display: "Markab", scientific: "Alpha Pegasi", ra_hours: 23.079, dec_deg: 15.205 },
    StarEntry { id: "NAV57_MENKAR", display: "Menkar", scientific: "Alpha Ceti", ra_hours: 3.038, dec_deg: 4.090 },
    StarEntry { id: "NAV57_MENKENT", display: "Menkent", scientific: "Theta Centauri", ra_hours: 14.112, dec_deg: -36.370 },
    StarEntry { id: "NAV57_MIAPLACIDUS", display: "Miaplacidus", scientific: "Beta Carinae", ra_hours: 9.220, dec_deg: -69.717 },
    StarEntry { id: "NAV57_MIRFAK", display: "Mirfak", scientific: "Alpha Persei", ra_hours: 3.405, dec_deg: 49.861 },
    StarEntry { id: "NAV57_NUNKI", display: "Nunki", scientific: "Sigma Sagittarii", ra_hours: 18.921, dec_deg: -26.297 },
    StarEntry { id: "NAV57_PEACOCK", display: "Peacock", scientific: "Alpha Pavonis", ra_hours: 20.427, dec_deg: -56.735 },
    StarEntry { id: "NAV57_POLARIS", display: "Polaris", scientific: "Alpha Ursae Minoris", ra_hours: 2.530, dec_deg: 89.264 },
    StarEntry { id: "NAV57_POLLUX", display: "Pollux", scientific: "Beta Geminorum", ra_hours: 7.755, dec_deg: 28.026 },
    StarEntry { id: "NAV57_PROCYON", display: "Procyon", scientific: "Alpha Canis Minoris", ra_hours: 7.655, dec_deg: 5.225 },
    StarEntry { id: "NAV57_RASALHAGUE", display: "Rasalhague", scientific: "Alpha Ophiuchi", ra_hours: 17.582, dec_deg: 12.560 },
    StarEntry { id: "NAV57_REGULUS", display: "Regulus", scientific: "Alpha Leonis", ra_hours: 10.140, dec_deg: 11.967 },
    StarEntry { id: "NAV57_RIGEL", display: "Rigel", scientific: "Beta Orionis", ra_hours: 5.242, dec_deg: -8.202 },
    StarEntry { id: "NAV57_RIGIL_KENTAURUS", display: "Rigil Kentaurus", scientific: "Alpha Centauri", ra_hours: 14.660, dec_deg: -60.835 },
    StarEntry { id: "NAV57_SABIK", display: "Sabik", scientific: "Eta Ophiuchi", ra_hours: 17.173, dec_deg: -15.724 },
    StarEntry { id: "NAV57_SCHEDAR", display: "Schedar", scientific: "Alpha Cassiopeiae", ra_hours: 0.675, dec_deg: 56.537 },
    StarEntry { id: "NAV57_SHAULA", display: "Shaula", scientific: "Lambda Scorpii", ra_hours: 17.560, dec_deg: -37.104 },
    StarEntry { id: "NAV57_SIRIUS", display: "Sirius", scientific: "Alpha Canis Majoris", ra_hours: 6.752, dec_deg: -16.716 },
    StarEntry { id: "NAV57_SPICA", display: "Spica", scientific: "Alpha Virginis", ra_hours: 13.420, dec_deg: -11.161 },
    StarEntry { id: "NAV57_SUHAIL", display: "Suhail", scientific: "Lambda Velorum", ra_hours: 9.134, dec_deg: -43.432 },
    StarEntry { id: "NAV57_VEGA", display: "Vega", scientific: "Alpha Lyrae", ra_hours: 18.616, dec_deg: 38.784 },
    StarEntry { id: "NAV57_ZUBENELGENUBI", display: "Zubenelgenubi", scientific: "Alpha Librae", ra_hours: 14.848, dec_deg: -16.042 },
];

pub fn find_star(name: &str) -> Option<StarEntry> {
    let key = normalize_star_key(name);
    STARS.iter().copied().find(|s| normalize_star_key(s.display) == key || normalize_star_key(s.id) == key)
}

fn normalize_star_key(s: &str) -> String {
    s.to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}
