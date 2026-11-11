//! The Price catalog from JSON: API rates and Plan prices, each with its source.

use nibble_core::{
    CatalogSource, ModelRates, PlanOffer, PriceCatalog, ProviderId, Rates, SourceError, Usd,
};
use serde::Deserialize;

/// A Price catalog read from JSON text.
#[derive(Debug, Clone)]
pub struct JsonCatalog {
    json: String,
}

impl JsonCatalog {
    /// A catalog over `json`, parsed on each read.
    #[must_use]
    pub fn new(json: &str) -> Self {
        Self {
            json: json.to_owned(),
        }
    }
}

#[derive(Deserialize)]
struct File {
    models: Vec<ModelEntry>,
    plans: Vec<PlanEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    provider: String,
    model_prefix: String,
    input: String,
    output: String,
    cache_read: String,
    cache_write_5m: String,
    cache_write_1h: String,
}

#[derive(Deserialize)]
struct PlanEntry {
    provider: String,
    id: String,
    price: String,
    capacity: u32,
}

impl CatalogSource for JsonCatalog {
    fn catalog(&self) -> Result<PriceCatalog, SourceError> {
        let file: File = serde_json::from_str(&self.json)
            .map_err(|e| SourceError::new(format!("price catalog is not valid: {e}")))?;
        let models = file
            .models
            .into_iter()
            .map(|m| {
                Ok(ModelRates {
                    provider: ProviderId::new(m.provider),
                    per_million: Rates {
                        input: dollars(&m.input)?,
                        output: dollars(&m.output)?,
                        cache_read: dollars(&m.cache_read)?,
                        cache_write_5m: dollars(&m.cache_write_5m)?,
                        cache_write_1h: dollars(&m.cache_write_1h)?,
                    },
                    model_prefix: m.model_prefix,
                })
            })
            .collect::<Result<_, SourceError>>()?;
        let plans = file
            .plans
            .into_iter()
            .map(|p| {
                let price = dollars(&p.price)?;
                Ok(PlanOffer::new(
                    ProviderId::new(p.provider),
                    &p.id,
                    price,
                    p.capacity,
                ))
            })
            .collect::<Result<_, SourceError>>()?;
        Ok(PriceCatalog { models, plans })
    }
}

/// Parses `"18.75"` as dollars, exactly, to micro-dollar precision.
fn dollars(text: &str) -> Result<Usd, SourceError> {
    let invalid = || SourceError::new(format!("price catalog has an invalid amount: {text:?}"));
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || (!fraction.is_empty() && !digits(fraction)) || fraction.len() > 6 {
        return Err(invalid());
    }
    let whole: u64 = whole.parse().map_err(|_| invalid())?;
    let fraction: u64 = format!("{fraction:0<6}").parse().map_err(|_| invalid())?;
    whole
        .checked_mul(1_000_000)
        .and_then(|micros| micros.checked_add(fraction))
        .map(Usd::from_micros)
        .ok_or_else(invalid)
}
