# syntax=docker/dockerfile:1.25.0@sha256:0adf442eae370b6087e08edc7c50b552d80ddf261576f4ebd6421006b2461f12

FROM rust:1.97.1@sha256:1bcff4befb740599103a2c7cb51058e14479b2e35e3a34a3f0dc4ede09927488 AS build-base
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


FROM build-base AS builder
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
# Le bundler .deb de Tauri horodate ses archives (ar + tar internes) avec
# l'heure réelle de build, en ignorant SOURCE_DATE_EPOCH — seul le binaire
# lui-même est déterministe. Réemballage déterministe en place (extraction
# complète avant toute écriture, donc source == destination est sûr) via
# script/package-deb-repro.sh, restauré depuis 7e3c652b (#107).
RUN for deb in src-tauri/target/release/bundle/deb/*.deb; do \
      ./script/package-deb-repro.sh "$deb" "$deb"; \
    done


FROM scratch AS export
COPY --from=linux-builder /app/src-tauri/target/release/sonar /bin/sonar
COPY --from=linux-builder /app/src-tauri/target/release/bundle/deb/ /deb
COPY --from=linux-builder /app/src-tauri/target/release/bundle/rpm/ /rpm


# Cross-compilation Windows depuis le même environnement épinglé :
# cible MSVC via cargo-xwin. L'outillage et le sysroot Microsoft sont placés
# dans une couche indépendante des sources applicatives : elle est réutilisable
# entre builds et transportable avec l'image (`docker save`). Le lien utilise
# lld. Les import libs Npcap sont versionnées dans le dépôt
# (src-tauri/windows/npcap-sdk), build.rs les branche pour tout
# CARGO_CFG_TARGET_OS=windows. Binaire seul (--no-bundle) : le bundling
# NSIS cross reste un chantier séparé (#119).
FROM build-base AS windows-toolchain
# La copie ultérieure de rust-toolchain.toml ne doit pas pousser rustup à
# consulter static.rust-lang.org pendant le build applicatif hors réseau.
# Cette version exacte est déjà installée par l'image Rust et sa cohérence avec
# config/build-versions.env est contrôlée par check-build-versions.sh.
ENV RUSTUP_TOOLCHAIN="1.97.1"
ARG WINDOWS_CROSS_APT_PACKAGES="clang=1:19.0-63 clang-tools=1:19.0-63 lld=1:19.0-63 llvm=1:19.0-63 nsis=3.11-1"
RUN apt install -y ${WINDOWS_CROSS_APT_PACKAGES}
ARG CARGO_XWIN_VERSION=0.23.0
ARG XWIN_VERSION=17
ARG XWIN_SDK_VERSION=10.0.26100
ARG XWIN_CRT_VERSION=14.44.17.14
ARG XWIN_ARCH=x86_64
ARG XWIN_HTTP_RETRIES=5
ENV XWIN_VERSION="${XWIN_VERSION}" \
    XWIN_SDK_VERSION="${XWIN_SDK_VERSION}" \
    XWIN_CRT_VERSION="${XWIN_CRT_VERSION}" \
    XWIN_ARCH="${XWIN_ARCH}" \
    XWIN_HTTP_RETRIES="${XWIN_HTTP_RETRIES}"
ENV XWIN_CACHE_DIR="/opt/cargo-xwin/${CARGO_XWIN_VERSION}/${XWIN_VERSION}-${XWIN_SDK_VERSION}-${XWIN_CRT_VERSION}-${XWIN_ARCH}"
RUN rustup target add x86_64-pc-windows-msvc \
  && cargo install cargo-xwin --locked --version "${CARGO_XWIN_VERSION}" \
  && cargo xwin --version
COPY script/ci/cache-xwin-toolchain.sh /usr/local/bin/cache-xwin-toolchain.sh
RUN bash /usr/local/bin/cache-xwin-toolchain.sh


FROM windows-toolchain AS windows-builder
WORKDIR /app
COPY . .
RUN deno install --frozen
ARG SOURCE_DATE_EPOCH=1700000000
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}
# Hôte Linux, cible Windows : force /Brepro (timestamps PE dérivés du
# contenu) que repro-env.ts n'active par défaut que sur hôte Windows, et
# supprime toute émission de PDB (/DEBUG:NONE) — le GUID RSDS du PDB était
# la dernière source de non-déterminisme, et le profil release strippe déjà
# les symboles. repro-env.ts compose ses flags par-dessus cette RUSTFLAGS.
ENV SONAR_REPRO_WINDOWS_BREPRO=1
ENV RUSTFLAGS="-C link-arg=/DEBUG:NONE"
# Le SDK/CRT a été splatté dans windows-toolchain. Couper le réseau ici prouve
# qu'un rebuild de l'application ne dépend plus du CDN Microsoft.
RUN --network=none deno run -A ./security/repro-env.ts run deno task tauri build \
  --runner cargo-xwin --target x86_64-pc-windows-msvc --no-bundle


FROM scratch AS export-windows
COPY --from=windows-builder /app/src-tauri/target/x86_64-pc-windows-msvc/release/sonar.exe /windows/sonar.exe
