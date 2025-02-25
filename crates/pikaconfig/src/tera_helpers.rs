use crate::tera_helper;

pub fn register(tera: &mut tera::Tera) {
    tera.register_filter(
        "enquote",
        tera_helper::wrap_nil_filter(|text: &String| Ok(quote::enquote(text))),
    );
}
