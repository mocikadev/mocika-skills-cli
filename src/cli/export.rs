use std::path::PathBuf;

use anyhow::Result;
use console::style;

use crate::core::{bundle, lock};
use crate::i18n;

const DEFAULT_BUNDLE_NAME: &str = "skills.bundle";

#[derive(clap::Args)]
#[command(about = "Export installed skills to a shareable bundle file")]
pub struct ExportArgs {
    #[arg(long, value_name = "FILE")]
    pub output: Option<String>,
}

pub fn run(args: ExportArgs) -> Result<()> {
    let entries = lock::list_skill_entries().unwrap_or_default();

    if entries.is_empty() {
        println!("{}", i18n::t("no installed skills"));
        return Ok(());
    }

    let mut skill_entries = Vec::new();
    let mut local_skipped = 0usize;

    for (name, entry) in &entries {
        match bundle::bundle_source_from_lock(entry) {
            Some(source) => skill_entries.push(bundle::BundleEntry {
                name: name.clone(),
                source,
            }),
            None => local_skipped += 1,
        }
    }

    let output_path = PathBuf::from(args.output.as_deref().unwrap_or(DEFAULT_BUNDLE_NAME));
    let skill_count = skill_entries.len();
    let b = bundle::Bundle {
        skills: skill_entries,
    };
    bundle::write_bundle(&output_path, &b)?;

    println!(
        "{} {}",
        style(i18n::t("export.exported")).green().bold(),
        output_path.display()
    );
    println!(
        "  {} {} {}",
        style("→").dim(),
        skill_count,
        i18n::t("export.skills_exported")
    );
    if local_skipped > 0 {
        println!(
            "  {} {} {}",
            style("·").dim(),
            local_skipped,
            i18n::t("export.local_skipped")
        );
    }

    Ok(())
}
