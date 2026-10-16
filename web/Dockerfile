# Build context: the repository root.
FROM node:26.10.0-alpine3.24@sha256:0b36e8c136b94cd4fcf02188228e76c31ad5872eef3fec8cbd2eee500cfd9e80 AS web
WORKDIR /usr/local/src/cinelist/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

FROM joseluisq/static-web-server:2.44.1@sha256:94618e3d17802265a21926217e6f9e7832b311b6fbf5933684001739fafed06a AS server

FROM scratch
COPY --from=server /static-web-server /usr/local/bin/static-web-server
COPY --from=web /usr/local/src/cinelist/web/dist /usr/share/cinelist/public
ENV SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    SERVER_ROOT=/usr/share/cinelist/public \
    SERVER_FALLBACK_PAGE=/usr/share/cinelist/public/index.html \
    SERVER_HEALTH=true \
    SERVER_LOG_LEVEL=info
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/static-web-server"]
