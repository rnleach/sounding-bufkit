//! Module for reading a bufkit file and breaking it into smaller pieces for parsing later.
use crate::parse_util::check_missing_i32;
use std::collections::HashMap;

use metfor::{MetersPSec, Quantity, WindUV};
use sounding_analysis::{Sounding, StationInfo};

use super::surface::SurfaceData;
use super::upper_air::UpperAir;

#[allow(clippy::needless_pass_by_value)]
pub fn combine_data(
    ua: UpperAir,
    sd: SurfaceData,
    fname: &str,
) -> (Sounding, HashMap<&'static str, f64>) {
    let coords: Option<(f64, f64)> = ua
        .lat
        .into_option()
        .and_then(|lat| ua.lon.into_option().map(|lon| (lat, lon)));

    let station = StationInfo::new_with_values(check_missing_i32(ua.num), coords, ua.elevation);

    let snd = Sounding::new()
        .with_source_description(fname.to_owned())
        .with_station_info(station)
        .with_valid_time(ua.valid_time)
        .with_lead_time(check_missing_i32(ua.lead_time))
        // Upper air
        .with_pressure_profile(ua.pressure)
        .with_temperature_profile(ua.temperature)
        .with_wet_bulb_profile(ua.wet_bulb)
        .with_dew_point_profile(ua.dew_point)
        .with_theta_e_profile(ua.theta_e)
        .with_wind_profile(ua.wind)
        .with_pvv_profile(ua.omega)
        .with_height_profile(ua.height)
        .with_cloud_fraction_profile(ua.cloud_fraction)
        // Surface data
        .with_mslp(sd.mslp)
        .with_sfc_temperature(sd.temperature)
        .with_sfc_dew_point(sd.dewpoint)
        .with_station_pressure(sd.station_pres)
        .with_low_cloud(sd.low_cloud)
        .with_mid_cloud(sd.mid_cloud)
        .with_high_cloud(sd.hi_cloud)
        .with_sfc_wind(sd.wind);

    macro_rules! check_and_add {
        ($opt:expr, $key:expr, $hash_map:ident) => {
            if let Some(val) = $opt.into_option() {
                $hash_map.insert($key, val.unpack());
            }
        };
    }

    macro_rules! check_and_add_boolean {
        ($opt:expr, $key:expr, $hash_map:ident) => {
            if let Some(val) = $opt {
                if val {
                    $hash_map.insert($key, 1.0);
                } else {
                    $hash_map.insert($key, 0.0);
                }
            }
        };
    }

    let mut bufkit_anal: HashMap<&'static str, f64> = HashMap::new();

    // Add some profile indexes.
    check_and_add!(ua.show, "Showalter", bufkit_anal);
    check_and_add!(ua.swet, "SWeT", bufkit_anal);
    check_and_add!(ua.kinx, "K", bufkit_anal);
    check_and_add!(ua.li, "LI", bufkit_anal);
    check_and_add!(ua.lclp, "LCL", bufkit_anal);
    check_and_add!(ua.pwat, "PWAT", bufkit_anal);
    check_and_add!(ua.totl, "TotalTotals", bufkit_anal);
    check_and_add!(ua.cape, "CAPE", bufkit_anal);
    check_and_add!(ua.cins, "CIN", bufkit_anal);
    check_and_add!(ua.lclt, "LCLTemperature", bufkit_anal);
    check_and_add!(ua.eqlv, "EquilibriumLevel", bufkit_anal);
    check_and_add!(ua.lfc, "LFC", bufkit_anal);
    check_and_add!(ua.brch, "BulkRichardsonNumber", bufkit_anal);

    // Add some surface data
    check_and_add!(sd.skin_temp, "SkinTemperature", bufkit_anal);
    check_and_add!(sd.lyr_1_soil_temp, "Layer1SoilTemp", bufkit_anal);
    check_and_add!(sd.snow_1hr, "SnowFall1HourKgPerMeterSquared", bufkit_anal);
    check_and_add!(sd.p01, "Precipitation1HrMm", bufkit_anal);
    check_and_add!(sd.c01, "ConvectivePrecip1HrMm", bufkit_anal);
    check_and_add!(sd.lyr_2_soil_temp, "Layer2SoilTemp", bufkit_anal);
    check_and_add!(sd.snow_ratio, "SnowRatio", bufkit_anal);
    check_and_add!(sd.visibility, "VisibilityKm", bufkit_anal);
    check_and_add!(sd.srh, "StormRelativeHelicity", bufkit_anal);
    check_and_add!(sd.wx_sym_cod, "WxSymbolCode", bufkit_anal);
    check_and_add_boolean!(sd.snow_type, "PrecipTypeSnow", bufkit_anal);
    check_and_add_boolean!(sd.rain_type, "PrecipTypeRain", bufkit_anal);
    check_and_add_boolean!(sd.fzra_type, "PrecipTypeFreezingRain", bufkit_anal);
    check_and_add_boolean!(sd.ice_pellets_type, "PrecipTypeIcePellets", bufkit_anal);

    if let Some(WindUV {
        u: MetersPSec(u),
        v: MetersPSec(v),
    }) = sd.storm_motion.into_option()
    {
        bufkit_anal.insert("StormMotionUMps", u);
        bufkit_anal.insert("StormMotionVMps", v);
    }

    (snd, bufkit_anal)
}
