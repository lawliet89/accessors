#[macro_use]
extern crate accessors;

#[derive(getters, setters)]
#[setters(into)]
struct Simple {
    normal_field: String,

    #[getter(ignore)]
    ignored_field: String,

    #[getter(return_type = "&str")]
    custom_return_type_field: String,
}

impl Simple {
    fn ignored_field(&self) -> &str {
        &self.ignored_field
    }
}

fn main() {
    let mut s = Simple {
        normal_field: "hello".to_owned(),
        ignored_field: "".to_string(),
        custom_return_type_field: "World".to_string()
    };

    println!("{}", s.normal_field());
    s.set_normal_field("there");

    let _: &str = s.custom_return_type_field();
}
