
# schemr 🧠📊

`schemr` is a schema management CLI written in Rust. It allows you to:
- Dump MySQL schemas to files
- Compare schemas between environments (e.g., dev vs qa, or qa vs prod)
- Track schema drift over time
- Manage connection configuration via a `.toml` file or interactive setup

## 🔧 Installation through Crates.io

```bash
cargo install schemr
```

## 🚀 Usage

### 1. Configure
Set up `schemr.toml` in the current directory, or run:

```bash
schemr configure
```

You can also set environment variables for sensitive values like DB passwords.

### 2. Dump Schema

```bash
schemr dump-schema --env qa
schemr dump-schema --env prod
```

### 3. Compare Schemas

```bash
schemr compare --env1 qa --env2 prod
```

The terminal output will show the schema diffs for the configured environments (within the schemr.toml file)
If the terminal output is difficult to read, `schemr compare` will also generate a friendly HTML report as well.


## 📁 Configuration File

`schemr.toml` should look like:

```toml
[qa]
host = "someqa.example.com"
port = 3306
username = "root"
password_env = "ENV_VAR_FOR_SCHEMR_DB_PASSWORD_QA"
database = "yourdbname"

[prod]
host = "someprod.example.com"
port = 3306
username = "root"
password_env = "ENV_VAR_FOR_SCHEMR_DB_PASSWORD_PROD"
database = "yourdbname"
```

## 🔐 Security Note

Please note that I do not allow actual passwords within the schemr.toml file.
The password_env is the ENVIRONMENT_VARIABLE_NAME of what the password will be.
You will then need to either set the ENVIRONMENT_VARIABLE either within .bashrc, .zshrc, or just use export

Example `export` commands:
export ENV_VAR_FOR_SCHEMR_DB_PASSWORD_QA=yourqapassword
export ENV_VAR_FOR_SCHEMR_DB_PASSWORD_PROD=yourprodpassword

## 📦 Directory Structure

```
schemr/
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── compare.rs
│   └── schema.rs
├── schema-dumps/
│   ├── env1/
│   └── env2/
├── schemr.toml
├── README.md
└── Cargo.toml
```

The command `schemr configure` will auto-generate the `schemr.toml` file, or advanced users may create/modify this manually.

## ✅ License

MIT License.
