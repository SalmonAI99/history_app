# Run with Docker Compose

Build the web app image and start it with:

```sh
docker compose up --build
```

Then open <http://localhost:3000>. Stop it with `Ctrl+C`; use
`docker compose down` to stop and remove the container and network. SQLite data
and uploaded files live in Compose volumes and remain when the container is
recreated.

The app listens on port 3000 inside the container. Set `PORT` to change the
host port, for example `PORT=8080 docker compose up --build`. The
`content/slides` directory is mounted read only for gated course media. Public
assets are included in the image. Set `RUST_LOG` to adjust server logging.

To remove the persisted database and uploads as well, run
`docker compose down --volumes`.
