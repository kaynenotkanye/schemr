use crate::config::Config;
use anyhow::{Context, Result};
use mysql::prelude::*;
use mysql::{Conn, OptsBuilder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Vec<String>,
    pub indexes: Vec<Index>,
}

#[derive(Serialize)]
struct IndexManifest {
    pub tables: Vec<String>,
}

pub fn dump_schema(env_name: &str, output_dir: &str) -> Result<()> {
    // Load config and environment
    let cfg = Config::load()?;
    let env = cfg.get_env(env_name)?;
    let pwd = std::env::var(&env.password_env)
        .context(format!("Env var '{}' not set", env.password_env))?;

    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(env.host.clone()))
        .tcp_port(env.port)
        .user(Some(env.username.clone()))
        .pass(Some(pwd))
        .db_name(Some(env.database.clone()));
    let mut conn = Conn::new(opts).context("MySQL connection failed")?;

    // let out = Path::new(output_dir);
    let out = Path::new(output_dir).join(env_name);
    fs::create_dir_all(&out).context("Failed to create output dir")?;

    let names: Vec<String> = conn.exec_map(
        "SELECT table_name FROM information_schema.tables \
        WHERE table_schema = ? AND table_type='BASE TABLE'",
        (env.database.clone(),),
        |t: String| t,
    )?;

    let tables: BTreeSet<String> = names.into_iter().collect();

    // _index.json is used by schemr as a metadata file about the database
    let manifest = IndexManifest { tables: tables.iter().cloned().collect() };
    let idx_file = File::create(&out.join("_index.json"))?;
    serde_json::to_writer_pretty(idx_file, &manifest)?;

    for tbl in &tables {
        // Columns: fetch raw tuples then map
        let cols_query = r#"
            SELECT column_name, column_type, is_nullable, column_default
            FROM information_schema.columns
            WHERE table_schema = ? AND table_name = ?
            ORDER BY ordinal_position
        "#;

        let cols: Vec<Column> = conn.exec_map(
            cols_query,
            (env.database.clone(), tbl.clone()),
            |(name, dt, null, def): (String, String, String, Option<String>)| Column {
                name,
                data_type: dt,
                is_nullable: null == "YES",
                default: def,
            },
        )?;

        // Primary key
        let pk_query = r#"
            SELECT column_name
            FROM information_schema.key_column_usage
            WHERE table_schema = ? AND table_name = ? AND constraint_name = 'PRIMARY'
            ORDER BY ordinal_position
        "#;
        let pk: Vec<String> = conn.exec(
            pk_query,
            (env.database.clone(), tbl.clone()),
        )?;

        // Indexes
        let idx_query = r#"
            SELECT index_name, column_name, non_unique
            FROM information_schema.statistics
            WHERE table_schema = ? AND table_name = ? AND index_name != 'PRIMARY'
            ORDER BY index_name, seq_in_index
        "#;
        let mut idx_map = std::collections::BTreeMap::new();
        for row in conn.exec_iter(idx_query, (env.database.clone(), tbl.clone()))? {
            let (name, col, non_unique): (String, String, u8) = mysql::from_row(row?);
            let entry = idx_map.entry(name.clone()).or_insert((Vec::new(), non_unique == 0));
            entry.0.push(col);
        }
        let indexes: Vec<Index> = idx_map
            .into_iter()
            .map(|(n, (c, u))| Index { name: n, columns: c, is_unique: u })
            .collect();

        // Serialize table schema
        let ts = TableSchema {
            name: tbl.clone(),
            columns: cols,
            primary_key: pk,
            indexes,
        };
        let file = File::create(&out.join(format!("{}.json", tbl)))?;
        serde_json::to_writer_pretty(file, &ts)?;
    }

    println!("Schema for '{}' dumped to {}", env_name, output_dir);
    Ok(())
}
