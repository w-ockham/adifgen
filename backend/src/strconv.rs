use anyhow::{anyhow, Result};
use axum::middleware::from_fn_with_state;
use chrono::prelude::{DateTime, Utc};
use regex::Regex;
use std::collections::HashMap;
use tower_http::catch_panic::ResponseForPanic;

pub fn adif_time_from_hamlog(h_date: &str, h_hour: &str) -> Result<(String, String)> {
    let re_date = Regex::new(r"(\d+)/(\d+)/(\d+)")?;
    let re_hour = Regex::new(r"(\d{2}):(\d{2})(\w)")?;

    let Some(cap_date) = re_date.captures(h_date) else {
        return Err(anyhow!("Wrong date format {}", h_date));
    };

    let Some(cap_hour) = re_hour.captures(h_hour) else {
        return Err(anyhow!("Wrong time format {}", h_hour));
    };

    let mut year: u32 = cap_date[1].parse()?;
    if year < 100 {
        // year in 2 digits
        if year > 65 {
            year += 1900
        } else {
            year += 2000
        }
    };

    let mut timezone = "+0900";
    if "ZU".contains(&cap_hour[3].to_uppercase()) {
        timezone = "+0000"
    }

    let timestr = &format!(
        "{year}/{}/{} {}:{} {timezone}",
        &cap_date[2], &cap_date[3], &cap_hour[1], &cap_hour[2]
    );
    let logtime = DateTime::parse_from_str(timestr, "%Y/%m/%d %H:%M %z")?;
    let utctime = logtime.with_timezone(&Utc);

    Ok((
        utctime.format("%Y%m%d").to_string(),
        utctime.format("%H%M").to_string(),
    ))
}

pub fn adif_mode_from_hamlog(h_mode: &str) -> Result<String> {
    let adif_mode_synonm = [
        ("FREEDV", "DIGITALVOICE"),
        ("DV", "DIGITALVOICE"),
        ("D-STAR", "DSTAR"),
        ("FUSION", "DSTAR"),
    ];

    let adif_mode = [
        ("FT4", "MFSK"),
        ("JS8", "MFSK"),
        ("C4FM", "DIGITALVOICE"),
        ("DMR", "DIGITALVOICE"),
        ("DSTAR", "DIGITALVOICE"),
    ];

    let mut res = h_mode.to_string().to_uppercase();

    for (k, v) in adif_mode_synonm.iter() {
        res = res.replace(k, v);
    }

    for (k, v) in adif_mode.iter() {
        res = res.replace(k, v);
    }

    Ok(res)
}

pub fn adif_band_from_hamlog(h_freq: &str) -> Result<String> {
    let band_table = [
        (0.1357, 0.1378, "2190m"),
        (0.472, 0.479, "630m"),
        (1.8, 1.9125, "160m"),
        (3.5, 3.805, "80m"),
        (7.0, 7.2, "40m"),
        (10.000, 10.150, "30m"),
        (14.0, 14.350, "20m"),
        (18.0, 18.168, "17m"),
        (21.0, 21.450, "15m"),
        (24.0, 24.990, "12m"),
        (28.0, 29.7, "10m"),
        (50.0, 54.0, "6m"),
        (144.0, 146.0, "2m"),
        (430.0, 440.0, "70cm"),
        (1200.0, 1300.0, "23cm"),
        (2400.0, 2450.0, "13cm"),
        (5650.0, 5850.0, "6cm"),
        (10000.0, 10250.0, "3cm"),
        (10450.0, 10500.0, "3cm"),
    ];
    let freq = h_freq.parse::<f64>()?;
    for (lower, upper, band) in band_table.iter() {
        if freq <= *upper && freq >= *lower {
            return Ok(band.to_string());
        }
    }
    Err(anyhow!("Unknown band {h_freq}"))
}

#[test]
fn hamlog_test() -> Result<()> {
    let pat1 = adif_time_from_hamlog("2000/01/01", "08:00J")?;
    let pat2 = adif_time_from_hamlog("00/01/01", "08:00J")?;
    let pat3 = adif_time_from_hamlog("78/01/01", "08:00J")?;
    let pat4 = adif_time_from_hamlog("2024/03/01", "08:00J")?;
    let pat5 = adif_time_from_hamlog("2024/02/29", "23:00U")?;
    let pat6 = adif_time_from_hamlog("2024/02/29", "23:00Z")?;
    let pat7 = adif_time_from_hamlog("2024/0A/29", "23:00Z").unwrap_err();
    let pat8 = adif_time_from_hamlog("2024/09/29", "23:0AZ").unwrap_err();

    assert_eq!("19991231", pat1.0);
    assert_eq!("2300", pat1.1);
    assert_eq!("19991231", pat2.0);
    assert_eq!("2300", pat2.1);
    assert_eq!("19771231", pat3.0);
    assert_eq!("2300", pat3.1);
    assert_eq!("20240229", pat4.0);
    assert_eq!("2300", pat4.1);
    assert_eq!("20240229", pat5.0);
    assert_eq!("2300", pat5.1);
    assert_eq!("20240229", pat6.0);
    assert_eq!("2300", pat6.1);

    assert_eq!("DIGITALVOICE", adif_mode_from_hamlog("Dv")?);
    assert_eq!("DIGITALVOICE", adif_mode_from_hamlog("D-STAR")?);
    assert_eq!("DIGITALVOICE", adif_mode_from_hamlog("FUSIoN")?);
    assert_eq!("DIGITALVOICE", adif_mode_from_hamlog("DIGITALvOICE")?);
    assert_eq!("MFSK", adif_mode_from_hamlog("ft4")?);
    assert_eq!("MFSK", adif_mode_from_hamlog("js8")?);
    assert_eq!("DIGITALVOICE", adif_mode_from_hamlog("FREEDV")?);

    assert_eq!("2190m", adif_band_from_hamlog("0.136")?);
    assert_eq!("630m", adif_band_from_hamlog("0.473")?);
    assert_eq!("160m", adif_band_from_hamlog("1.9125")?);
    assert_eq!("80m", adif_band_from_hamlog("3.5")?);
    assert_eq!("40m", adif_band_from_hamlog("7.021")?);
    assert_eq!("30m", adif_band_from_hamlog("10.125")?);
    assert_eq!("20m", adif_band_from_hamlog("14.010")?);
    assert_eq!("17m", adif_band_from_hamlog("18.094")?);
    assert_eq!("15m", adif_band_from_hamlog("21.404")?);
    assert_eq!("12m", adif_band_from_hamlog("24.925")?);
    assert_eq!("10m", adif_band_from_hamlog("29.120")?);
    assert_eq!("6m", adif_band_from_hamlog("51.030")?);
    assert_eq!("2m", adif_band_from_hamlog("144.4")?);
    assert_eq!("70cm", adif_band_from_hamlog("433.2")?);
    assert_eq!("23cm", adif_band_from_hamlog("1295")?);
    assert_eq!("13cm", adif_band_from_hamlog("2449")?);
    assert_eq!("6cm", adif_band_from_hamlog("5655.1")?);
    assert_eq!("3cm", adif_band_from_hamlog("10249")?);

    Ok(())
}
