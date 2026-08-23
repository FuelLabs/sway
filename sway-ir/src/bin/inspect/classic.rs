//! Classic (non-TUI) printer — the original inspect output path.

use std::io::{IsTerminal, Write};

use anyhow::{bail, Result};

use crate::parse::{
    diff_stats, has_metadata_defs, parse_source_map, prepare_ir_text, print_diff, print_diff_stats,
    print_final_ir, FuncStats, MdMode, ParsedIr, PrintMode, VersionMode,
};

pub(crate) fn run(
    irs: &[ParsedIr],
    cleaned: &str,
    filter_fn: &[String],
    mode: PrintMode,
    metadata: MdMode,
    version: VersionMode,
    source: bool,
) -> Result<()> {
    if source && !has_metadata_defs(cleaned) {
        eprintln!(
            "warning: `--print source` was requested but the input contains no metadata \
             definitions (`!N = ...`).\n  \
             Regenerate the IR dump with the `print-md` flag, e.g. \
             `forc build --ir all print-md`, so the span/file metadata is emitted."
        );
    }

    let stdout = std::io::stdout();
    let is_terminal = stdout.is_terminal();
    let mut out = stdout.lock();

    let mut previous_ir: Option<String> = None;

    for ir in irs {
        if ir.is_initial() {
            previous_ir = None;
        }

        let Some(final_ir) = prepare_ir_text(filter_fn, metadata, version, ir) else {
            continue;
        };

        let mut source_map = if source {
            Some(parse_source_map(&ir.body))
        } else {
            None
        };

        writeln!(out, "// IR: {}", ir.pass_name)?;

        if mode == PrintMode::Diff {
            if let Some(prev_text) = previous_ir.as_ref() {
                let changeset = prettydiff::diff_lines(prev_text, &final_ir);
                let ops = changeset.diff();
                let (adds, removes) = diff_stats(&ops);
                if adds == 0 && removes == 0 {
                    print_diff_stats(&mut out, adds, removes)?;
                } else {
                    let cur_stats = FuncStats::compute_stats(&final_ir);
                    let prev_stats = previous_ir.as_ref().map(|p| FuncStats::compute_stats(p));
                    cur_stats.print_fn_stats(&mut out, prev_stats.as_ref())?;
                    print_diff_stats(&mut out, adds, removes)?;
                    print_diff(&mut out, &ops, is_terminal)?;
                }
            } else {
                let cur_stats = FuncStats::compute_stats(&final_ir);
                cur_stats.print_fn_stats(&mut out, None)?;
                print_final_ir(&mut out, &final_ir, source_map.as_mut())?;
            }
        } else {
            let cur_stats = FuncStats::compute_stats(&final_ir);
            cur_stats.print_fn_stats(&mut out, None)?;
            print_final_ir(&mut out, &final_ir, source_map.as_mut())?;
        }

        previous_ir = Some(final_ir);
    }

    if irs.is_empty() {
        bail!("no `// IR:` dumps found in input");
    }

    Ok(())
}
