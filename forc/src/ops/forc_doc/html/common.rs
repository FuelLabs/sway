use maud::{html, Markup, DOCTYPE};

pub fn header(page_title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title { (page_title) }
    }
}

pub fn footer() -> Markup {
    html! {
        footer {
            "example footer"
        }
    }
}
