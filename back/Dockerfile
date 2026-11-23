# --- Étape 1 : compilation (avec cache des dépendances) ---
FROM rust:1 AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Couche de dépendances : recompilée uniquement si Cargo.toml/Cargo.lock changent
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin sigre-back

# --- Étape 2 : image finale minimale ---
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/sigre-back /usr/local/bin/sigre-back
ENV DATABASE_URL=postgres://postgres:postgres@db:5432/revisions
EXPOSE 3000
ENTRYPOINT ["sigre-back"]
