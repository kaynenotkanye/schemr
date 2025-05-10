use crate::schema::TableSchema;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{collections::{BTreeMap, BTreeSet}, fs::File, fs::write, path::Path};

#[derive(Deserialize)]
struct IndexManifest { tables: Vec<String> }

struct HtmlSection {
    title: String,
    body: String,
}

impl HtmlSection {
    fn new<T: Into<String>, U: Into<String>>(title: T, body: U) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }

    fn to_html(&self) -> String {
        format!(
            r#"<details open><summary><strong>{}</strong></summary><pre>{}</pre></details>"#,
            self.title, self.body
        )
    }
}

pub fn compare(env1: &str, env2: &str) -> Result<()> {
    let dir1 = Path::new("schemr-dumps").join(env1);
    let dir2 = Path::new("schemr-dumps").join(env2);

    let m1: IndexManifest = serde_json::from_reader(File::open(dir1.join("_index.json"))?)
        .map_err(|_| anyhow!("No dump for env '{}'", env1))?;
    let m2: IndexManifest = serde_json::from_reader(File::open(dir2.join("_index.json"))?)
        .map_err(|_| anyhow!("No dump for env '{}'", env2))?;

    let t1: BTreeSet<String> = m1.tables.into_iter().collect();
    let t2: BTreeSet<String> = m2.tables.into_iter().collect();
    let all: BTreeSet<_> = t1.union(&t2).cloned().collect();

    println!("Comparing '{}' vs '{}'...", env1, env2);
    let mut html_sections: Vec<HtmlSection> = vec![];
    let mut summary = String::new();

    let only1: Vec<_> = t1.difference(&t2).cloned().collect();
    let only2: Vec<_> = t2.difference(&t1).cloned().collect();
    if !only1.is_empty() {
        println!("Only in {}: {:?}", env1, only1);
        summary += &format!("Only in {}: {:?}\n", env1, only1);
    }
    if !only2.is_empty() {
        println!("Only in {}: {:?}", env2, only2);
        summary += &format!("Only in {}: {:?}\n", env2, only2);
    }
    html_sections.push(HtmlSection::new("Summary", summary));

    for tbl in all {
        // Skip tables that are not present in both environments
        if !t1.contains(&tbl) || !t2.contains(&tbl) {
            continue;
        }

        println!("\nComparing table: {}", tbl);
        // let mut section = format!("Comparing table: {}\n", tbl);
        let mut section = format!("\n");

        let ts1: TableSchema = serde_json::from_reader(File::open(dir1.join(&format!("{}.json", tbl)))?)?;
        let ts2: TableSchema = serde_json::from_reader(File::open(dir2.join(&format!("{}.json", tbl)))?)?;

        let c1: BTreeMap<_, _> = ts1.columns.iter().map(|c| (c.name.clone(), c)).collect();
        let c2: BTreeMap<_, _> = ts2.columns.iter().map(|c| (c.name.clone(), c)).collect();
        let n1: BTreeSet<_> = c1.keys().cloned().collect();
        let n2: BTreeSet<_> = c2.keys().cloned().collect();

        let onlyc1: Vec<_> = n1.difference(&n2).cloned().collect();
        let onlyc2: Vec<_> = n2.difference(&n1).cloned().collect();
        if !onlyc1.is_empty() || !onlyc2.is_empty() {
            section += "Column differences:\n";
        }
        if !onlyc1.is_empty() {
            println!("  Only in {}: {:?}", env1, onlyc1);
            section += &format!("  Only in {}: {:?}\n", env1, onlyc1);
        }
        if !onlyc2.is_empty() {
            println!("  Only in {}: {:?}", env2, onlyc2);
            section += &format!("  Only in {}: {:?}\n", env2, onlyc2);
        }

        for col in n1.intersection(&n2) {
            let a = c1.get(col).unwrap();
            let b = c2.get(col).unwrap();
            let mut diffs = vec![];
            if a.data_type != b.data_type {
                diffs.push(format!("type {} vs {}", a.data_type, b.data_type));
            }
            if a.is_nullable != b.is_nullable {
                diffs.push(format!("nullable {} vs {}", a.is_nullable, b.is_nullable));
            }
            if a.default != b.default {
                diffs.push(format!("default {:?} vs {:?}", a.default, b.default));
            }
            if !diffs.is_empty() {
                println!("  Column '{}': {}", col, diffs.join(", "));
                section += &format!("  Column '{}': {}\n", col, diffs.join(", "));
            }
        }

        let idx1: BTreeSet<_> = ts1.indexes.iter().map(|i| (i.name.clone(), i.columns.clone(), i.is_unique)).collect();
        let idx2: BTreeSet<_> = ts2.indexes.iter().map(|i| (i.name.clone(), i.columns.clone(), i.is_unique)).collect();

        let onlyidx1: Vec<_> = idx1.difference(&idx2).cloned().collect();
        let onlyidx2: Vec<_> = idx2.difference(&idx1).cloned().collect();
        if !onlyidx1.is_empty() || !onlyidx2.is_empty() {
            section += "Index differences:\n";
        }
        if !onlyidx1.is_empty() {
            println!("  Only in {}: {:?}", env1, onlyidx1);
            section += &format!("  Only in {}: {:?}\n", env1, onlyidx1);
        }
        if !onlyidx2.is_empty() {
            println!("  Only in {}: {:?}", env2, onlyidx2);
            section += &format!("  Only in {}: {:?}\n", env2, onlyidx2);
        }

        for idx in idx1.intersection(&idx2) {
            let (name1, cols1, uniq1) = idx;
            let (_name2, cols2, uniq2) = idx;
            let mut diffs = vec![];
            if cols1 != cols2 {
                diffs.push(format!("columns {:?} vs {:?}", cols1, cols2));
            }
            if uniq1 != uniq2 {
                diffs.push(format!("uniqueness {} vs {}", uniq1, uniq2));
            }
            if !diffs.is_empty() {
                println!("  Index '{}': {}", name1, diffs.join(", "));
                section += &format!("  Index '{}': {}\n", name1, diffs.join(", "));
            }
        }

        html_sections.push(HtmlSection::new(tbl.clone(), section));
    }

    // Write HTML report
    let mut html = String::from(r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Schema Diff Report</title></head><body>"#);
    for section in html_sections {
        html += &section.to_html();
    }
    html += "</body></html>";
    write("schema_diff_report.html", html)?;

    Ok(())
}
