use minijinja::Environment;

pub(crate) fn register(env: &mut Environment) {
    env.add_filter("enquote", quote::enquote);
}
