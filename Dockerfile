# mgldvd/tauri-vue (https://hub.docker.com/r/mgldvd/tauri-vue) — this
# project's own prebuilt base image, replacing ivangabriele/tauri:debian-
# bookworm-22. The apt/rustup steps below stay in place since they're
# idempotent no-ops if the tooling is already present in the image.
FROM mgldvd/tauri-vue:latest

# PATH is re-asserted explicitly (covering both the official rust image's
# /usr/local/cargo and a plain rustup-as-root's /root/.cargo) rather than
# trusted from the base image alone: `cargo: command not found` showed up
# in practice even though the base image sets it too.
ENV DEBIAN_FRONTEND=noninteractive \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    npm_config_fetch_retries=5 \
    npm_config_fetch_retry_mintimeout=10000 \
    npm_config_fetch_retry_maxtimeout=120000 \
    npm_config_fetch_timeout=300000 \
    PATH="/usr/local/cargo/bin:/root/.cargo/bin:${PATH}"

# mgldvd/tauri-vue defaults to a non-root USER, unlike the old
# ivangabriele/tauri image — apt/rustup need root to install/modify
# system-wide state, so switch back explicitly for this step.
USER root

# patchelf: required by Tauri's Linux bundler for AppImage RPATH patching,
# not part of the base image. pkg-config: almost certainly already pulled
# in transitively by the *-dev packages, but installing it explicitly costs
# nothing and removes the doubt. clippy/rustfmt: `make lint` needs both;
# the base image's Rust install doesn't document including them, so this
# adds them if missing (a fast no-op if they're already present).
RUN apt-get update \
    && apt-get install -y --no-install-recommends patchelf pkg-config \
    && rm -rf /var/lib/apt/lists/* \
    && rustup component add clippy rustfmt

WORKDIR /workspace

# Plain (non-login) shell: `bash -lc` sources /etc/profile and friends,
# which on some Debian-derived images reset PATH from scratch and drop
# cargo's directory again even though ENV set it correctly above.
CMD ["bash", "scripts/dist-container.sh"]
