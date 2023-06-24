use anyhow::Result;

use crate::quote;
use crate::tera_helper;

pub fn register(tera: &mut tera::Tera) -> Result<()> {
    tera.register_filter(
        "enquote",
        tera_helper::wrap_nil_filter(|text: &String| Ok(quote::enquote(text))),
    );
    Ok(())
}
