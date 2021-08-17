use lazy_static::lazy_static;
use std::{collections::HashMap, env, fs::File, path::PathBuf};

use crate::map;
use crate::utils::cli_error::CliError;

// CSS files
static FONTS_CSS: &[u8] = include_bytes!("static/css/fonts.css");
static HEADER_CSS: &[u8] = include_bytes!("static/css/header.css");
static MAIN_CSS: &[u8] = include_bytes!("static/css/main.css");
static NORMALIZE_CSS: &[u8] = include_bytes!("static/css/normalize.css");
static ROOT_CSS: &[u8] = include_bytes!("static/css/root.css");

// Fonts
static FIRA_SANS_LICENSE: &[u8] = include_bytes!("static/fonts/FiraSans-LICENSE.txt");
static FIRA_SANS_MEDIUM: &[u8] = include_bytes!("static/fonts/FiraSans-Medium.woff");
static FIRA_SANS_MEDIUM_2: &[u8] = include_bytes!("static/fonts/FiraSans-Medium.woff2");
static FIRA_SANS_REGULAR: &[u8] = include_bytes!("static/fonts/FiraSans-Regular.woff");
static FIRA_SANS_REGULAR_2: &[u8] = include_bytes!("static/fonts/FiraSans-Regular.woff2");
static SOURCE_CODE_PRO_LICENSE: &[u8] = include_bytes!("static/fonts/SourceCodePro-LICENSE.txt");
static SOURCE_CODE_PRO_ITALIC: &[u8] = include_bytes!("static/fonts/SourceCodePro-It.ttf.woff");
static SOURCE_CODE_PRO_ITALIC_2: &[u8] = include_bytes!("static/fonts/SourceCodePro-It.ttf.woff2");
static SOURCE_CODE_PRO_REGULAR: &[u8] =
    include_bytes!("static/fonts/SourceCodePro-Regular.ttf.woff");
static SOURCE_CODE_PRO_REGULAR_2: &[u8] =
    include_bytes!("static/fonts/SourceCodePro-Regular.ttf.woff2");
static SOURCE_CODE_PRO_SEMIBOLD: &[u8] =
    include_bytes!("static/fonts/SourceCodePro-Semibold.ttf.woff");
static SOURCE_CODE_PRO_SEMIBOLD_2: &[u8] =
    include_bytes!("static/fonts/SourceCodePro-Semibold.ttf.woff2");
static SOURCE_SERIF_4_LICENSE: &[u8] = include_bytes!("static/fonts/SourceSerif4-License.md");
static SOURCE_SERIF_4_BOLD: &[u8] = include_bytes!("static/fonts/SourceSerif4-Bold.ttf.woff");
static SOURCE_SERIF_4_BOLD_2: &[u8] = include_bytes!("static/fonts/SourceSerif4-Bold.ttf.woff2");
static SOURCE_SERIF_4_ITALIC: &[u8] = include_bytes!("static/fonts/SourceSerif4-It.ttf.woff");
static SOURCE_SERIF_4_ITALIC_2: &[u8] = include_bytes!("static/fonts/SourceSerif4-It.ttf.woff2");
static SOURCE_SERIF_4_REGULAR: &[u8] = include_bytes!("static/fonts/SourceSerif4-Regular.ttf.woff");
static SOURCE_SERIF_4_REGULAR_2: &[u8] =
    include_bytes!("static/fonts/SourceSerif4-Regular.ttf.woff2");

// todo: this can be achieved with stdlib, once std::lazy is available on 'stable' - https://doc.rust-lang.org/std/lazy/index.html
lazy_static! {
    static ref CSS_FILES: HashMap<&'static str, &'static [u8]> = map! {
        "fonts.css" => FONTS_CSS,
        "header.css" => HEADER_CSS,
        "main.css" => MAIN_CSS,
        "normalize.css" => NORMALIZE_CSS,
        "root.css" => ROOT_CSS
    };
    static ref FONT_FILES: HashMap<&'static str, &'static [u8]> = map! {
        "FiraSans-LICENSE.txt" => FIRA_SANS_LICENSE,
        "FiraSans-Medium.woff" => FIRA_SANS_MEDIUM,
        "FiraSans-Medium.woff2" => FIRA_SANS_MEDIUM_2,
        "FiraSans-Regular.woff" => FIRA_SANS_REGULAR,
        "FiraSans-Regular.woff2" => FIRA_SANS_REGULAR_2,

        "SourceCodePro-LICENSE.txt" => SOURCE_CODE_PRO_LICENSE,
        "SourceCodePro-It.ttf.woff" => SOURCE_CODE_PRO_ITALIC,
        "SourceCodePro-It.ttf.woff2" => SOURCE_CODE_PRO_ITALIC_2,
        "SourceCodePro-Regular.ttf.woff" => SOURCE_CODE_PRO_REGULAR,
        "SourceCodePro-Regular.ttf.woff2" => SOURCE_CODE_PRO_REGULAR_2,
        "SourceCodePro-Semibold.ttf.woff" => SOURCE_CODE_PRO_SEMIBOLD,
        "SourceCodePro-Semibold.ttf.woff2" => SOURCE_CODE_PRO_SEMIBOLD_2,

        "SourceSerif4-License.md" => SOURCE_SERIF_4_LICENSE,
        "SourceSerif4-Bold.ttf.woff" => SOURCE_SERIF_4_BOLD,
        "SourceSerif4-Bold.ttf.woff2" => SOURCE_SERIF_4_BOLD_2,
        "SourceSerif4-It.ttf.woff" => SOURCE_SERIF_4_ITALIC,
        "SourceSerif4-It.ttf.woff2" => SOURCE_SERIF_4_ITALIC_2,
        "SourceSerif4-Regular.ttf.woff" => SOURCE_SERIF_4_REGULAR,
        "SourceSerif4-Regular.ttf.woff2" => SOURCE_SERIF_4_REGULAR_2,
    };
}

pub fn build_css_files(project_path: &PathBuf) -> Result<(), CliError> {
    let prev_dir = env::current_dir()?;
    env::set_current_dir(&format!("{}/static/css", &project_path.to_str().unwrap()))?;

    for (file_name, data) in CSS_FILES.iter() {
        let _ = File::create(file_name)?;
        std::fs::write(&file_name, data)?;
    }

    // reset to previous directory
    env::set_current_dir(prev_dir)?;

    Ok(())
}

pub fn build_font_files(project_path: &PathBuf) -> Result<(), CliError> {
    let prev_dir = env::current_dir()?;
    env::set_current_dir(&format!("{}/static/fonts", &project_path.to_str().unwrap()))?;

    for (file_name, data) in FONT_FILES.iter() {
        let _ = File::create(&file_name)?;
        std::fs::write(&file_name, data)?;
    }

    // reset to previous directory
    env::set_current_dir(prev_dir)?;

    Ok(())
}
