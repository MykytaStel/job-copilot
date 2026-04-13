# ── Stage 1: build ───────────────────────────────────────────────────────────
# Build context must be the monorepo root so packages/ workspace is accessible.
FROM node:22-slim AS builder

RUN npm install -g pnpm@10

WORKDIR /workspace

# Copy workspace manifests + lockfile first for layer caching
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml tsconfig.base.json ./
COPY packages/ ./packages/
COPY apps/web/ ./apps/web/

RUN pnpm install --frozen-lockfile

# VITE_ENGINE_API_URL is baked into the static bundle at build time.
# It must be the URL the user's browser can reach — not the internal Docker hostname.
ARG VITE_ENGINE_API_URL=http://localhost:8080
ENV VITE_ENGINE_API_URL=$VITE_ENGINE_API_URL

RUN pnpm --filter @job-copilot/web build

# ── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM nginx:alpine

COPY --from=builder /workspace/apps/web/dist /usr/share/nginx/html
COPY apps/web/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
