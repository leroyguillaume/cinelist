# Build context: the repository root.
FROM node:26.11.1-alpine3.24@sha256:143494b1da2945f061539253adc65e4f1569ddf07da2d384c022c791a9d90a4a AS web
WORKDIR /usr/local/src/cinelist/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

FROM joseluisq/static-web-server:2.44.0@sha256:2c1a7c3e0feaea5859307403b74e1c575f3ec1499094fc077344173d11abaae2 AS server

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
