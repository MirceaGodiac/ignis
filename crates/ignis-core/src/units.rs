//! Deterministic unit conversion to SI. Unknown units are an error: a unit we cannot interpret is a
//! curation problem, not something to guess. Affine conversions (temperature) are explicit.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unit {
    pub symbol_si: &'static str,
    pub dimension: &'static str,
    pub factor: f64,
    pub offset: f64,
}

const fn u(symbol_si: &'static str, dimension: &'static str, factor: f64) -> Unit {
    Unit {
        symbol_si,
        dimension,
        factor,
        offset: 0.0,
    }
}

pub fn lookup(unit: &str) -> Option<Unit> {
    Some(match unit.trim() {
        "m" => u("m", "length", 1.0),
        "cm" => u("m", "length", 1e-2),
        "mm" => u("m", "length", 1e-3),
        "um" | "µm" | "micron" => u("m", "length", 1e-6),
        "in" | "inch" => u("m", "length", 0.0254),
        "m2" | "m^2" => u("m2", "area", 1.0),
        "mm2" | "mm^2" => u("m2", "area", 1e-6),
        "cm2" | "cm^2" => u("m2", "area", 1e-4),
        "m2/s" | "m^2/s" => u("m2/s", "area_rate", 1.0),
        "mm2/s" | "mm^2/s" => u("m2/s", "area_rate", 1e-6),
        "m/s" => u("m/s", "velocity", 1.0),
        "cm/s" => u("m/s", "velocity", 1e-2),
        "mm/s" => u("m/s", "velocity", 1e-3),
        "s" | "sec" => u("s", "time", 1.0),
        "ms" => u("s", "time", 1e-3),
        "min" => u("s", "time", 60.0),
        "Pa" => u("Pa", "pressure", 1.0),
        "kPa" => u("Pa", "pressure", 1e3),
        "MPa" => u("Pa", "pressure", 1e6),
        "bar" => u("Pa", "pressure", 1e5),
        "atm" => u("Pa", "pressure", 101325.0),
        "mmHg" => u("Pa", "pressure", 133.322387415),
        "torr" | "Torr" => u("Pa", "pressure", 101325.0 / 760.0),
        "psi" | "psia" => u("Pa", "pressure", 6894.757293168361),
        "K" => u("K", "temperature", 1.0),
        "C" | "degC" | "°C" => Unit {
            symbol_si: "K",
            dimension: "temperature",
            factor: 1.0,
            offset: 273.15,
        },
        "1" | "fraction" | "mol/mol" => u("1", "fraction", 1.0),
        "%" | "pct" | "mol%" => u("1", "fraction", 1e-2),
        "ppm" => u("1", "fraction", 1e-6),
        "kg/s" => u("kg/s", "mass_flow", 1.0),
        "g/s" => u("kg/s", "mass_flow", 1e-3),
        "mg/s" => u("kg/s", "mass_flow", 1e-6),
        "m3/s" => u("m3/s", "volume_flow", 1.0),
        "L/min" | "lpm" => u("m3/s", "volume_flow", 1e-3 / 60.0),
        // Standard-condition volume flow keeps its own dimension: the reference state belongs to the source.
        "slpm" | "SLPM" => u("Sm3/s", "std_volume_flow", 1e-3 / 60.0),
        "sccm" | "SCCM" => u("Sm3/s", "std_volume_flow", 1e-6 / 60.0),
        "g0" | "g" => u("1", "gravity_ratio", 1.0),
        "px" => u("px", "pixel", 1.0),
        _ => return None,
    })
}

#[derive(Debug, thiserror::Error)]
#[error("unit {0:?} is not registered; add it with its source convention")]
pub struct UnknownUnit(pub String);

pub fn to_si(value: f64, unit: &str) -> Result<(f64, &'static str), UnknownUnit> {
    let x = lookup(unit).ok_or_else(|| UnknownUnit(unit.to_string()))?;
    Ok((value * x.factor + x.offset, x.symbol_si))
}

pub fn from_si(value_si: f64, unit: &str) -> Result<f64, UnknownUnit> {
    let x = lookup(unit).ok_or_else(|| UnknownUnit(unit.to_string()))?;
    Ok((value_si - x.offset) / x.factor)
}
