# ================================================================
# STAGE 1 — Build
# ================================================================
FROM node:20-slim AS builder

# Install Titan CLI (latest)
RUN npm install -g @ezetgalaxy/titan@latest

WORKDIR /app

# Copy project files
COPY . .

# Install JS dependencies
RUN npm install

# Build Titan App
RUN titan build

# ================================================================
# STAGE 2 — Runtime
# ================================================================
FROM node:20-slim

WORKDIR /app

# Copy Titan Build Artifacts
COPY --from=builder /app/.titan ./.titan
COPY --from=builder /app/server ./server
COPY --from=builder /app/package.json ./package.json

# Install dependencies (production only, but for now we might need all if not careful)
# Actually, the runtime depends on `express`-like behavior?
# No, Titan pure TS runs via `node .titan/app.js` which might need deps?
# The user's package.json has dependencies.
RUN npm install --omit=dev

EXPOSE 3000

CMD ["node", ".titan/app.js"]
