//! Seam B: the bundled Price catalog.

use nibble_adapters::JsonCatalog;
use nibble_core::{CatalogSource, PlanId, ProviderId, Usd};

const FIXTURE: &str = r#"{
  "retrieved": "2026-11-11",
  "models": [
    { "provider": "anthropic", "model_prefix": "claude-opus-5", "input": "5", "output": "25",
      "cache_read": "0.50", "cache_write_5m": "6.25", "cache_write_1h": "10", "source": "https://example.test" }
  ],
  "plans": [
    { "provider": "anthropic", "id": "pro", "price": "20", "capacity": 1, "source": "https://example.test" },
    { "provider": "anthropic", "id": "max_5x", "price": "100", "capacity": 5, "source": "https://example.test" }
  ]
}"#;

#[test]
fn parses_decimal_prices_exactly() {
    let catalog = JsonCatalog::new(FIXTURE).catalog().unwrap();

    let opus = catalog.models.first().unwrap();
    assert_eq!(opus.model_prefix, "claude-opus-5");
    assert_eq!(opus.per_million.cache_read, Usd::from_micros(500_000));
    assert_eq!(opus.per_million.cache_write_5m, Usd::from_micros(6_250_000));
    let pro = catalog.plans.first().unwrap();
    assert_eq!(
        (pro.id.clone(), pro.price, pro.capacity),
        (PlanId::new("pro"), Usd::from_cents(2_000), 1)
    );
    assert_eq!(pro.provider, ProviderId::new("anthropic"));
}

#[test]
fn a_malformed_price_is_an_error() {
    let broken = FIXTURE.replace("\"0.50\"", "\"fifty cents\"");

    assert!(JsonCatalog::new(&broken).catalog().is_err());
}

#[test]
fn the_bundled_catalog_prices_every_individual_claude_plan() {
    let catalog = JsonCatalog::bundled().catalog().unwrap();

    let capacities: Vec<_> = ["pro", "max_5x", "max_20x"]
        .iter()
        .map(|id| {
            catalog
                .plans
                .iter()
                .find(|p| p.id == PlanId::new(*id))
                .map(|p| p.capacity)
        })
        .collect();
    // Anthropic now sells max_20x as "Ultra", described as 10x Pro (claude.com/pricing, 2026-11-11).
    assert_eq!(capacities, vec![Some(1), Some(5), Some(10)]);
    let names: Vec<_> = catalog.plans.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["Pro", "Max", "Ultra"]);
    assert!(
        catalog
            .models
            .iter()
            .any(|m| "claude-opus-5-5".starts_with(&m.model_prefix))
    );
}
