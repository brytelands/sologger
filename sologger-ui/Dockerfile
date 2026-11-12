# Sologger UI: a static Vue + WASM site served by nginx at http://localhost:8080/sologger/
#
#   docker compose up sologger-ui            (from the repository root)
#   docker build -t sologger-ui:local sologger-ui && docker run -p 8080:8080 sologger-ui:local
#
# The build uses the WASM parser committed under public/sologger-log-transformer-wasm/pkg,
# so no Rust toolchain is needed. Rebuild it first (the wasm-sync skill) to pick up parser
# changes. The app runs entirely in the browser and talks to the Solana RPC chosen in the
# UI directly; it does not connect to a sologger container.

ARG NODE_VERSION=24.12
ARG NGINX_VERSION=1.30.6

FROM node:${NODE_VERSION}-alpine AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY . .
RUN npm run build

# Unprivileged variant: runs as a non-root user and listens on 8080
FROM nginxinc/nginx-unprivileged:${NGINX_VERSION}-alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
# The app is built for the /sologger/ base path (vite base + router history)
COPY --from=build /app/dist /usr/share/nginx/html/sologger
EXPOSE 8080
