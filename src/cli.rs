use std::io::Cursor;

use anyhow::{anyhow, Result};
use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    Skim,
};

pub fn pick_option<V: Into<String>>(options: V, title: &str) -> Result<String> {
    let skim_options = SkimOptionsBuilder::default()
        .header(Some(title))
        .no_height(true)
        .no_hscroll(true)
        .no_clear(true)
        .multi(false)
        .build()
        .unwrap();

    let input: String = options.into();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let pick_result =
        Skim::run_with(&skim_options, Some(items)).ok_or(anyhow!("Failed to display options"))?;

    if pick_result.is_abort {
        return Err(anyhow!("You have to pick an option"));
    }

    let picked_option = pick_result
        .selected_items
        .get(0)
        .ok_or(anyhow!("First option not found"))?
        .output()
        .to_string();

    return Ok(picked_option);
}
