FROM rust:1.96.0@sha256:f4d1e78866618fe7155aa6eaea26f9f6270d105e4918ee2c2f2dd5b2c11cc815 AS builder
ENV NODE_VERSION="24.15.0"
ENV DENO_VERSION="2.8.3"
ARG DOCKER_APT_PACKAGES="libgtk-3-dev=3.24.49-3 pkg-config=1.8.1-4 libjavascriptcoregtk-4.1-dev=2.52.5-1~deb13u1 libsoup-3.0-dev=3.6.5-3 libwebkit2gtk-4.1-dev=2.52.5-1~deb13u1 libpcap-dev=1.10.5-2 unzip"
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
# Le contexte docker n'a pas de .git (.dockerignore) : l'epoch ne peut pas
# être dérivée de l'historique, elle est injectée en build-arg — même valeur
# ⇒ même binaire, quel que soit le démon Docker qui construit (voir
# script/release/repro-build-container.sh).
ARG SOURCE_DATE_EPOCH=1700000000
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}


FROM builder AS linux-builder
RUN deno run -A ./security/repro-env.ts run deno task tauri build


FROM scratch AS export
COPY --from=linux-builder /app/src-tauri/target/release/sonar /bin/sonar
COPY --from=linux-builder /app/src-tauri/target/release/bundle/deb/ /deb
COPY --from=linux-builder /app/src-tauri/target/release/bundle/rpm/ /rpm


# Cross-compilation Windows depuis le même environnement épinglé :
# cible MSVC via cargo-xwin (SDK Windows téléchargé et vérifié par xwin),
# lien par lld. Les import libs Npcap sont versionnées dans le dépôt
# (src-tauri/windows/npcap-sdk), build.rs les branche pour tout
# CARGO_CFG_TARGET_OS=windows. Binaire seul (--no-bundle) : le bundling
# NSIS cross reste un chantier séparé (#119).
FROM builder AS windows-builder
ARG WINDOWS_CROSS_APT_PACKAGES="clang=1:19.0-63 clang-tools=1:19.0-63 lld=1:19.0-63 llvm=1:19.0-63 nsis=3.11-1"
RUN apt install -y ${WINDOWS_CROSS_APT_PACKAGES}
ARG CARGO_XWIN_VERSION=0.23.0
RUN rustup target add x86_64-pc-windows-msvc \
  && cargo install cargo-xwin --locked --version "${CARGO_XWIN_VERSION}"
RUN deno run -A ./security/repro-env.ts run deno task tauri build \
  --runner cargo-xwin --target x86_64-pc-windows-msvc --no-bundle


FROM scratch AS export-windows
COPY --from=windows-builder /app/src-tauri/target/x86_64-pc-windows-msvc/release/sonar.exe /windows/sonar.exe
