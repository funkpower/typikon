# Releasing

## The branching workflow

`main` is always releasable. Everything else is short-lived.

```
feat/whatever ──push──▶ ci runs tests
      │
      └──merge──▶ main ──tag v0.3.0──▶ release builds and publishes to PyPI
```

- **Work on a branch.** Pushing any branch runs `ci.yml`: clippy, the Rust tests,
  the Python bindings built and tested against the freshly built CLI, and mypy
  against the stubs.
- **Merge to `main`** when it is green.
- **Release by tagging.** `Cargo.toml` carries the version and the tag points at
  it. The first job refuses to continue if the two disagree, because PyPI will
  not let a version be reused — not even after it is yanked — so publishing
  0.2.0 under a tag that says v0.3.0 cannot be undone.

```console
$ # on main, working tree clean
$ sed -i 's/^version = .*/version = "0.3.0"/' Cargo.toml
$ cargo build            # refresh Cargo.lock
$ git commit -am 'release 0.3.0'
$ git tag v0.3.0 && git push origin main v0.3.0
```

`__version__` in Python comes from `CARGO_PKG_VERSION`, and maturin reads the
same field, so `Cargo.toml` is the only place a version is written.

## Why pull requests do not trigger CI

`ci.yml` fires on `push` and nothing else. This forge is public and takes pull
requests from strangers; a runner executes whatever the workflow checks out, so a
`pull_request` trigger would run an unread stranger's code on the runner host.
Push-only means the runner only ever executes code that someone with push rights
put there.

The cost is real: a pull request gets no automatic check. Read the diff, push the
branch yourself, and let it run then.

## What has to exist first

### 1. Actions enabled on the forge

Off by default here, deliberately — see `homedocs/services/forgejo-public.md`.
In `app.ini` on moria:

```ini
[actions]
ENABLED = true
```

```console
$ ssh moria 'cd ~/docker-services/forgejo-public && docker compose down && docker compose up -d'
```

Always restart the whole stack. `network_mode: service:forge-wg` means restarting
the forge alone silently strips its networking.

### 2. A runner

The forge has no default route and cannot address the LAN, so it cannot reach a
runner. Runners poll outward, so register one against the **public** URL and let
it come to the forge the same way any other client does:

```console
$ forgejo-runner register --no-interactive \
    --instance https://git.pumpin.online \
    --token <from Settings → Actions → Runners> \
    --name <host> --labels docker
```

It needs Docker and outbound internet — it fetches crates and wheels. It executes
repository code, so put it somewhere you would be willing to lose, not on a host
that matters. moria runs the MCP servers and the forge itself; that is the wrong
box for it.

### 3. A PyPI token

Registered under the **funkpower** account. Make it **project-scoped to
typikon**, not account-wide, and add it as the repository secret `PYPI_TOKEN`.

PyPI's trusted publishing is OIDC and supports GitHub, GitLab, Google Cloud Build
and ActiveState — not Forgejo. So this forge has to hold a long-lived credential,
and scoping it to the one project is the whole of the mitigation available: if
the forge is compromised, what is lost is the ability to publish typikon, not
every package the account owns.

Rotate it at `pypi.org/manage/account/token/` if the forge is ever suspect.

## What gets published

| | |
|---|---|
| `typikon-X.Y.Z.tar.gz` | sdist — builds from source anywhere with a Rust toolchain |
| `…-cp39-abi3-manylinux…x86_64.whl` | |
| `…-cp39-abi3-manylinux…aarch64.whl` | cross-built with `maturin --zig` |

abi3, so one wheel per platform covers every Python from 3.9 up rather than one
per interpreter.

**No macOS or Windows wheels.** Forgejo has no hosted runners, so those need a
Mac and a Windows box registered as runners. Until then both fall back to the
sdist, which needs a Rust toolchain on the installing machine — fine for
developers, a wall for everyone else.

## Checking a release without publishing

```console
$ rm -rf dist          # a wheel left in dist/ gets packed into the sdist
$ maturin sdist --out dist && maturin build --release --out dist
$ twine check dist/*
$ python -m venv /tmp/v && /tmp/v/bin/pip install --no-binary :all: dist/*.tar.gz
$ /tmp/v/bin/python -c 'import typikon, datetime; print(typikon.today())'
```

Installing from the sdist with `--no-binary` is the one that catches a missing
file, because it builds the crate from exactly what PyPI would hand someone.
