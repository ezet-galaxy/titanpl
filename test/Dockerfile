# ================================================================
# STAGE 1 — Build Titan (JS → Rust)
# ================================================================
FROM rust:1.91.1 AS builder

# Install Node for Titan CLI + bundler
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install Titan CLI (latest)
RUN npm install -g @ezetgalaxy/titan@latest

WORKDIR /app

# Copy project files
COPY . .

# Install JS dependencies (needed for Titan DSL + bundler)
RUN npm install

# Build Titan metadata + bundle JS actions
RUN titan build

# Build Rust binary
RUN cd server && cargo build --release



# ================================================================
# STAGE 2 — Runtime Image (Lightweight)
# ================================================================
FROM debian:stable-slim

WORKDIR /app

# Copy Rust binary from builder stage
COPY --from=builder /app/server/target/release/server ./titan-server

# Copy Titan routing metadata
COPY --from=builder /app/server/routes.json ./routes.json
COPY --from=builder /app/server/action_map.json ./action_map.json

# Copy Titan JS bundles
RUN mkdir -p /app/actions
COPY --from=builder /app/server/actions /app/actions

# Expose Titan port
EXPOSE 3000

# Start Titan
CMD ["./titan-server"]
