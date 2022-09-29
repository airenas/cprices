#########################################################################################
## docker will invoke this file from ../.. dir in order to access code
#########################################################################################
FROM rust:1.64-alpine3.15 as builder

RUN apk --update add openssl-dev musl-dev

ARG BUILD_VERSION=0.1

WORKDIR /src/

COPY cprices /src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
      CARGO_PKG_VERSION=$BUILD_VERSION cargo build --release
#########################################################################################
FROM alpine:3.15 as runner

ARG BUILD_VERSION=0.1

LABEL org.opencontainers.image.version=$BUILD_VERSION \
      org.opencontainers.image.authors="airenass@gmail.com" \
      name="cprices-importer" \
      maintainer="airenass@gmail.com" \
      vendor="airenass@gmail.com" \
      version=$BUILD_VERSION \
      release=$BUILD_VERSION \
      summary="This image is used to host the cprice import service" \
      description="This image is used to host the cprice import service" 

WORKDIR /app
ENV UID=999

RUN addgroup app && adduser -S -G app app --uid $UID app
RUN mkdir -p /app && chown -R app:app /app   

COPY --from=builder /src/target/release/importer /app/

RUN chown app:app /app/* /app
 
USER app

ENTRYPOINT ["./importer"]
