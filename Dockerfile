FROM rust:1.95.0@sha256:5b1e3484ddcd22a3738c0ec34a5e98bf19382eb295fb6db54295e62379119040 AS builder
ENV NODE_VERSION="24.14.0"
ENV DENO_VERSION="2.7.13"
ARG DOCKER_APT_PACKAGES="libgtk-3-dev=3.24.49-3 pkg-config=1.8.1-4 libjavascriptcoregtk-4.1-dev=2.52.3-2~deb13u1 libsoup-3.0-dev=3.6.5-3 libwebkit2gtk-4.1-dev=2.52.3-2~deb13u1 libpcap-dev=1.10.5-2"
RUN cargo -V
ENV PATH="/usr/local/node/bin:$PATH"

RUN arch="$(dpkg --print-architecture)" && \
    case "$arch" in \
      amd64) node_arch="x64" ;; \
      arm64) node_arch="arm64" ;; \
      *) echo "Unsupported architecture: $arch" >&2; exit 1 ;; \
    esac && \
    curl -fsSL "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-${node_arch}.tar.xz" -o /tmp/node.tar.xz && \
    mkdir -p /usr/local/node && \
    tar -xJf /tmp/node.tar.xz --strip-components=1 -C /usr/local/node && \
    rm -f /tmp/node.tar.xz
RUN npm install -g "deno@${DENO_VERSION}"

COPY config/build-versions.env /app/config/build-versions.env
COPY script/ci/use-apt-snapshot.sh /app/script/ci/use-apt-snapshot.sh
RUN /app/script/ci/use-apt-snapshot.sh
RUN apt install -y ${DOCKER_APT_PACKAGES}

WORKDIR /app
COPY . .
RUN deno install --frozen
RUN deno task tauri build


FROM scratch AS export
COPY --from=builder /app/src-tauri/target/release/bundle/deb/ /deb
COPY --from=builder /app/src-tauri/target/release/bundle/rpm/ /rpm
