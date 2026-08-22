# mgldvd/tauri-vue (https://hub.docker.com/r/mgldvd/tauri-vue) — this
# project's own prebuilt base image, replacing ivangabriele/tauri:debian-
# bookworm-22. The apt/rustup steps below stay in place since they're
# idempotent no-ops if the tooling is already present in the image.
FROM mgldvd/tauri-vue:latest

# No PATH override here (there used to be one prepending
# /usr/local/cargo/bin:/root/.cargo/bin as a defensive fallback): neither
# directory actually exists in this base image — cargo lives at
# /opt/cargo/bin, already on the base image's own PATH, confirmed directly
# against a running container. Worse, /root/.cargo/bin isn't just a no-op
# for a non-root process (see `USER root` below and dist-container.sh):
# `/root` itself exists but is 0700, so linuxdeploy's own PATH-scanning
# during `task build`'s AppImage bundling step segfaulted on an unhandled
# `boost::filesystem::filesystem_error: Permission denied: "/root/.cargo/bin"`
# the moment this container started running that step as a non-root user
# instead of root.
ENV DEBIAN_FRONTEND=noninteractive \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    npm_config_fetch_retries=5 \
    npm_config_fetch_retry_mintimeout=10000 \
    npm_config_fetch_retry_maxtimeout=120000 \
    npm_config_fetch_timeout=300000

# mgldvd/tauri-vue defaults to a non-root USER, unlike the old
# ivangabriele/tauri image — apt/rustup need root to install/modify
# system-wide state, so switch back explicitly for this step. Root stays
# the effective user all the way through CMD (no `USER master` below):
# `HOST_UID`/`HOST_GID` (see compose.yml) are only known at `docker compose
# run` time, not at image-build time, so `scripts/dist-container.sh` itself
# remaps the base image's non-root user to match them and drops from root
# via `gosu` before doing anything else — see that script.
USER root

# patchelf: required by Tauri's Linux bundler for AppImage RPATH patching,
# not part of the base image. pkg-config: almost certainly already pulled
# in transitively by the *-dev packages, but installing it explicitly costs
# nothing and removes the doubt. clippy/rustfmt: `task lint` needs both;
# the base image's Rust install doesn't document including them, so this
# adds them if missing (a fast no-op if they're already present). gosu:
# lets `scripts/dist-container.sh` drop from root to a uid/gid-matched user
# before touching the bind-mounted repo — see that script for why.
RUN apt-get update \
    && apt-get install -y --no-install-recommends patchelf pkg-config gosu \
    && rm -rf /var/lib/apt/lists/* \
    && rustup component add clippy rustfmt

# go-task/task: not packaged for Debian/Ubuntu, so installed via the
# project's own official install script rather than apt. This container's
# entrypoint (scripts/dist-container.sh) now runs `task release` — the repo
# no longer has a Makefile.
RUN sh -c "$(curl --location https://taskfile.dev/install.sh)" -- -d -b /usr/local/bin

WORKDIR /workspace

# Plain (non-login) shell: `bash -lc` sources /etc/profile and friends,
# which on some Debian-derived images reset PATH from scratch and drop
# cargo's directory again even though ENV set it correctly above.
CMD ["bash", "scripts/dist-container.sh"]
