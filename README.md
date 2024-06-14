# Administration Center API
A RESTful API for a proxmox-based network administration center built with Rust. This repo is the backend used by the [Administration Center Frontend](https://github.com/0Killian/AdminCenter) project.

## Goals
- Authentication & User Management
- VM & Container Management using Proxmox
- Manage different applications though plugins
- More to come...

## Usage
Though this API can be used as is, it is recommended to use it together with the [Administration Center Frontend](https://github.com/0Killian/AdminCenter) project.

### Prerequisites
- A SQL database (SQLite, MySQL or Postgres)

### Configuration
The backend is configured through the `.env` file. A sample is available at [.env.sample](.env.sample).
These variables must be present:
- `DATABASE_URI`: The URI of the database to use. See [.env.sample](.env.sample) and [src/config.rs](src/config.rs) for examples and available options.

These variables are optional:
- `HOST`: The host to listen on. Defaults to `0.0.0.0`
- `PORT`: The port to listen on. Defaults to `3000`