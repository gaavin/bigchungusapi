# OGC API

[OGC API](https://ogcapi.ogc.org/) building blocks implemented in [Rust](https://www.rust-lang.org/)

## Quick Start

This will take a while and use quite some disk space

```bash
# Setup the database
docker-compose up

# Import administrative bounaries
docker exec -ti ogcapi \
    cargo run -- \
    import ogcapi/data/ne_110m_admin_0_countries.geojson \
    --collection countries
```

Open <http://localhost:8484/> were you will find the `Landing Page`.

## Developing

### Prerequisites

- Rust
- Docker & Docker Compose
- GDAL

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres,rustls
```

### Setup

```bash
# Setup the database
docker-compose up db db-migrations

# Run tests
cargo test --workspace

# Import administrative bounaries
cargo run -- import ogcapi/data/ne_110m_admin_0_countries.geojson --collection countries

# Serve 
cargo run -- serve

# Documentation
cargo doc --workspace --all-features --no-deps --open
```

## Teamengine

```bash
docker run --network host ogccite/ets-ogcapi-features10
# docker run --network host ogccite/ets-ogcapi-edr10
```
