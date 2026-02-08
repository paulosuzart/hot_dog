# Hot Dog

A simple parenting app for tracking kids' points/notes over recurring cycles. Parents can award or deduct points based on completed tasks or behavior. Points reset at the end of each cycle (e.g., monthly) and can be used as a reward system.

Built as an exploration of [Dioxus](https://dioxuslabs.com/), [Turso](https://turso.tech/), and [Fly.io](https://fly.io/).

## Local Development

### Prerequisites

- [Rust](https://rustup.rs/)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started): `cargo install dioxus-cli`
- A Turso database (or local libSQL instance)

### Environment Variables

```bash
export TURSO_DATABASE_URL="<your-turso-db-url>"
export TURSO_AUTH_TOKEN="<your-turso-auth-token>"
```

### Run

```bash
dx serve --platform web
```
