* default:
  - .tasks/commands/default.yml

* dev:
  - .tasks/commands/frontend-deps.yml
  - .tasks/commands/dev.yml

* dev-design:
  - .tasks/commands/frontend-deps.yml
  - .tasks/commands/dev.yml

* docker:release:
  - .tasks/commands/docker/release.yml
  - compose.yml
  - Dockerfile
  - .tasks/scripts/dist-container.sh
  - .tasks/commands/typecheck.yml
  - .tasks/commands/lint.yml
  - .tasks/commands/test.yml
  - .tasks/commands/artifacts.yml
  - .tasks/commands/build.yml
  - .tasks/scripts/copy-artifacts.sh
  - .tasks/commands/release.yml

* docker:release-install:
  - .tasks/commands/docker/release.yml
