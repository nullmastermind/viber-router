set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]
set shell := ["bash", "-cu"]

dev-ui:
    cd {{justfile_directory()}} && bun run dev

dev-api:
    cd {{justfile_directory()}}/viber-router-api && cargo run

check:
    cd {{justfile_directory()}} && bun run vue-tsc --noEmit
    cd {{justfile_directory()}} && bun run biome lint ./src
    cd {{justfile_directory()}}/viber-router-api && cargo check
    cd {{justfile_directory()}}/viber-router-api && cargo clippy -- -D warnings

docker-build:
    cd {{justfile_directory()}} && docker build -t nullmastermind/viber-router:latest .

docker-push:
    cd {{justfile_directory()}} && docker push nullmastermind/viber-router:latest
