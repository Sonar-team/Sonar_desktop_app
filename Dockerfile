FROM rust:latest AS builder
ENV NODE_VERSION="24.14.0"
RUN apt update
RUN cargo -V
ENV PATH="/usr/local/node/bin:$PATH"

RUN curl -sL https://github.com/nodenv/node-build/archive/master.tar.gz | tar xz -C /tmp/
RUN /tmp/node-build-master/bin/node-build "${NODE_VERSION}" /usr/local/node
RUN npm install -g deno

RUN apt install -y libgtk-3-dev pkg-config libjavascriptcoregtk-4.1-dev libsoup-3.0-dev

WORKDIR /app
COPY . .
RUN deno install
RUN deno task tauri build


FROM scratch AS export
COPY --from=builder /app/src-tauri/target/release/bundle/deb/ /deb
COPY --from=builder /app/src-tauri/target/release/bundle/rpm/ /rpm