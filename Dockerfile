FROM rust:1.95.0 AS builder
ENV NODE_VERSION="24.14.0"
ENV DENO_VERSION="2.7.13"
RUN apt update
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

RUN apt install -y libgtk-3-dev pkg-config libjavascriptcoregtk-4.1-dev libsoup-3.0-dev

WORKDIR /app
COPY . .
RUN deno install --frozen
RUN deno task tauri build


FROM scratch AS export
COPY --from=builder /app/src-tauri/target/release/bundle/deb/ /deb
COPY --from=builder /app/src-tauri/target/release/bundle/rpm/ /rpm
