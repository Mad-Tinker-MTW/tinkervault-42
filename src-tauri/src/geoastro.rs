use crate::errors::{Result, VaultError};
use crate::nautical57::StarEntry;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct PlaceLock {
    pub label: String,
    pub lat: f64,
    pub lon: f64,
}

pub fn resolve_place_lock(input: &str) -> Result<PlaceLock> {
    if let Some((lat, lon)) = parse_coords(input) {
        return Ok(PlaceLock {
            label: "ManualCoordinates".to_string(),
            lat,
            lon,
        });
    }

    // Final-core fallback: deterministic place vector.
    // This keeps the Rust core offline and repeatable even without geocoding.
    // For highest confidence, users should use exact coordinates.
    let normalized = normalize_place(input);
    if normalized.is_empty() {
        return Err(VaultError::Message("Place or coordinates cannot be empty.".into()));
    }
    let digest = Sha256::digest(normalized.as_bytes());
    let lat_raw = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);
    let lon_raw = u32::from_be_bytes([digest[4], digest[5], digest[6], digest[7]]);
    let lat = (lat_raw as f64 / u32::MAX as f64) * 180.0 - 90.0;
    let lon = (lon_raw as f64 / u32::MAX as f64) * 360.0 - 180.0;
    Ok(PlaceLock {
        label: normalized,
        lat: round6(lat),
        lon: round6(lon),
    })
}

fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

fn normalize_place(input: &str) -> String {
    input.trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_coords(text: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = text
        .split(|c| c == ',' || c == ';' || c == ' ')
        .filter(|s| !s.trim().is_empty())
        .collect();
    if parts.len() < 2 {
        return None;
    }
    let lat = parts[0].trim().parse::<f64>().ok()?;
    let lon = parts[1].trim().parse::<f64>().ok()?;
    if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) {
        Some((round6(lat), round6(lon)))
    } else {
        None
    }
}

pub fn normalize_date(date: &str) -> Result<String> {
    let v = date.trim().replace('.', "/").replace('-', "/");
    let parts: Vec<&str> = v.split('/').collect();
    if parts.len() != 3 {
        return Err(VaultError::Message("Date must be YYYY-MM-DD, YYYY/MM/DD, or MM/DD/YYYY.".into()));
    }

    if parts[0].len() == 4 {
        let y = parts[0].parse::<i32>().map_err(|_| VaultError::Message("Invalid year.".into()))?;
        let m = parts[1].parse::<u32>().map_err(|_| VaultError::Message("Invalid month.".into()))?;
        let d = parts[2].parse::<u32>().map_err(|_| VaultError::Message("Invalid day.".into()))?;
        validate_date(y, m, d)?;
        Ok(format!("{:04}-{:02}-{:02}", y, m, d))
    } else {
        let m = parts[0].parse::<u32>().map_err(|_| VaultError::Message("Invalid month.".into()))?;
        let d = parts[1].parse::<u32>().map_err(|_| VaultError::Message("Invalid day.".into()))?;
        let y = parts[2].parse::<i32>().map_err(|_| VaultError::Message("Invalid year.".into()))?;
        validate_date(y, m, d)?;
        Ok(format!("{:04}-{:02}-{:02}", y, m, d))
    }
}

fn validate_date(y: i32, m: u32, d: u32) -> Result<()> {
    if !(1..=12).contains(&m) {
        return Err(VaultError::Message(format!("Invalid date {}-{:02}-{:02}: month must be 01-12.", y, m, d)));
    }
    let max_day = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
            if leap { 29 } else { 28 }
        }
        _ => unreachable!(),
    };
    if d < 1 || d > max_day {
        return Err(VaultError::Message(format!(
            "Invalid date {}-{:02}-{:02}: month {} has at most {} days.",
            y, m, d, m, max_day
        )));
    }
    Ok(())
}

pub fn normalize_sky_time(t: &str) -> Result<String> {
    let mut v = t.trim().to_string();
    if v.chars().all(|c| c.is_ascii_digit()) && (v.len() == 3 || v.len() == 4) {
        if v.len() == 3 {
            v = format!("0{}", v);
        }
        v = format!("{}:{}", &v[0..2], &v[2..4]);
    }
    let parts: Vec<&str> = v.split(':').collect();
    if parts.len() != 2 {
        return Err(VaultError::Message("Sky Time must be HH:MM or HHMM.".into()));
    }
    let h = parts[0].parse::<u32>().map_err(|_| VaultError::Message("Invalid hour.".into()))?;
    let m = parts[1].parse::<u32>().map_err(|_| VaultError::Message("Invalid minute.".into()))?;
    if h > 23 || m > 59 {
        return Err(VaultError::Message("Sky Time must be 00:00 through 23:59.".into()));
    }
    Ok(format!("{:02}:{:02}", h, m))
}

fn julian_day_utc(date: &str, sky_time: &str) -> Result<f64> {
    let err = |s: &str| VaultError::Message(format!("Internal date/time parse error: '{}'", s));
    let dp: Vec<i32> = date
        .split('-')
        .map(|x| x.parse::<i32>().map_err(|_| err(x)))
        .collect::<Result<Vec<_>>>()?;
    let tp: Vec<i32> = sky_time
        .split(':')
        .map(|x| x.parse::<i32>().map_err(|_| err(x)))
        .collect::<Result<Vec<_>>>()?;

    if dp.len() < 3 || tp.len() < 2 {
        return Err(VaultError::Message("Internal: malformed normalized date/time.".into()));
    }

    let mut y = dp[0];
    let mut m = dp[1];
    let d = dp[2];
    let hh = tp[0];
    let mm = tp[1];

    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let a = y / 100;
    let b = 2 - a + a / 4;
    let day_fraction = (hh as f64 + mm as f64 / 60.0) / 24.0;

    Ok((365.25 * (y + 4716) as f64).floor()
        + (30.6001 * (m + 1) as f64).floor()
        + d as f64
        + b as f64
        - 1524.5
        + day_fraction)
}

pub fn alt_az(place: &PlaceLock, date: &str, sky_time: &str, star: StarEntry) -> Result<(String, String)> {
    let lat = place.lat.to_radians();
    let lon_deg = place.lon;
    let ra_hours = star.ra_hours;
    let dec = star.dec_deg.to_radians();

    let jd = julian_day_utc(date, sky_time)?;
    let d = jd - 2451545.0;
    let gmst_hours = (18.697374558 + 24.06570982441908 * d).rem_euclid(24.0);
    let lst_hours = (gmst_hours + lon_deg / 15.0).rem_euclid(24.0);
    let ha = (((lst_hours - ra_hours) * 15.0 + 180.0).rem_euclid(360.0) - 180.0).to_radians();

    let sin_alt = dec.sin() * lat.sin() + dec.cos() * lat.cos() * ha.cos();
    let alt = sin_alt.clamp(-1.0, 1.0).asin();

    let y = -ha.sin();
    let x = dec.tan() * lat.cos() - lat.sin() * ha.cos();
    let az = y.atan2(x);

    Ok((
        format!("{:.3}", alt.to_degrees()),
        format!("{:.3}", az.to_degrees().rem_euclid(360.0)),
    ))
}
