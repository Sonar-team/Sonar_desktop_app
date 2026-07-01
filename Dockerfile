FROM rust:1.96.0@sha256:f4d1e78866618fe7155aa6eaea26f9f6270d105e4918ee2c2f2dd5b2c11cc815 AS builder
ENV NODE_VERSION="24.15.0"
ENV DENO_VERSION="2.8.3"
ARG DOCKER_APT_PACKAGES="libgtk-3-dev=3.24.49-3 pkg-config=1.8.1-4 libjavascriptcoregtk-4.1-dev=2.52.3-2~deb13u1 libsoup-3.0-dev=3.6.5-3 libwebkit2gtk-4.1-dev=2.52.3-2~deb13u1 libpcap-dev=1.10.5-2 unzip"
RUN cargo -V

COPY config/build-versions.env /app/config/build-versions.env
COPY script/ci/use-apt-snapshot.sh /app/script/ci/use-apt-snapshot.sh
RUN /app/script/ci/use-apt-snapshot.sh
RUN apt install -y ${DOCKER_APT_PACKAGES}

ENV PATH="/usr/local/node/bin:$PATH"

RUN arch="$(dpkg --print-architecture)" && \
    case "$arch" in \
      amd64) node_arch="x64" ;; \
      arm64) node_arch="arm64" ;; \
      *) echo "Unsupported architecture: $arch" >&2; exit 1 ;; \
    esac && \
    node_archive="node-v${NODE_VERSION}-linux-${node_arch}.tar.xz" && \
    curl -fsSL "https://nodejs.org/dist/v${NODE_VERSION}/${node_archive}" -o "/tmp/${node_archive}" && \
    curl -fsSL "https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt" -o /tmp/SHASUMS256.txt && \
    grep -F "  ${node_archive}" /tmp/SHASUMS256.txt > /tmp/node.sha256sum && \
    (cd /tmp && sha256sum --check --status node.sha256sum) && \
    mkdir -p /usr/local/node && \
    tar -xJf "/tmp/${node_archive}" --strip-components=1 -C /usr/local/node && \
    rm -f "/tmp/${node_archive}" /tmp/SHASUMS256.txt /tmp/node.sha256sum

RUN arch="$(dpkg --print-architecture)" && \
    case "$arch" in \
      amd64) deno_target="x86_64-unknown-linux-gnu" ;; \
      arm64) deno_target="aarch64-unknown-linux-gnu" ;; \
      *) echo "Unsupported architecture: $arch" >&2; exit 1 ;; \
    esac && \
    deno_archive="deno-${deno_target}.zip" && \
    curl -fsSL "https://github.com/denoland/deno/releases/download/v${DENO_VERSION}/${deno_archive}" -o "/tmp/${deno_archive}" && \
    curl -fsSL "https://github.com/denoland/deno/releases/download/v${DENO_VERSION}/${deno_archive}.sha256sum" -o "/tmp/${deno_archive}.sha256sum" && \
    (cd /tmp && sha256sum --check --status "${deno_archive}.sha256sum") && \
    unzip -q "/tmp/${deno_archive}" -d /usr/local/bin && \
    rm -f "/tmp/${deno_archive}" "/tmp/${deno_archive}.sha256sum"

WORKDIR /app
COPY . .
RUN deno install --frozen
RUN deno task tauri build


FROM scratch AS export
COPY --from=builder /app/src-tauri/target/release/bundle/deb/ /deb
COPY --from=builder /app/src-tauri/target/release/bundle/rpm/ /rpm
