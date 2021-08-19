use super::page_type::PageType;
use maud::{html, Markup, DOCTYPE};

pub(crate) fn build_page(body: Markup, main_sidebar: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            (head())
            body {
                (header())
                main {
                    (*main_sidebar)
                    (body)
                }
            }

        }
    }
}

pub(crate) fn build_body(page_type: &PageType) -> Markup {
    let name = page_type.get_name();
    let type_name = page_type.get_type();
    let details = page_type.build_details();

    html! {
        body {
            (header())
            main {
                section class="main-section" {
                    h1 class="section-header-1" {
                        (type_name)" "(name)
                    }

                    @if let Some(details) = details {
                        (details)
                    }
                }
            }
        }
    }
}

pub(crate) fn main_sidebar(project_name: &str, main_sidebar: Vec<Markup>) -> Markup {
    html! {
        nav class="sidebar" {
            div class="logo-container" {
                "LOGO"
            }
            h2 class="location" {
                (project_name)
            }

            @for page in main_sidebar {
                (page)
            }
        }
    }
}

pub(crate) fn build_type_sidebar(type_name: &str, names: &Vec<(String, Markup)>) -> Markup {
    html! {
        div class="block" {
            h3 { (type_name) }
            ul {
                @for (name, _) in names {
                    li {
                        a href=(format!("{}.html", name)) {
                            (name)
                        }
                    }
                }

            }
        }
    }
}

fn head() -> Markup {
    html! {
        head {
            link rel="stylesheet" type="text/css" href="static/css/fonts.css";
            link rel="stylesheet" type="text/css" href="static/css/normalize.css";
            link rel="stylesheet" type="text/css" href="static/css/root.css";
            link rel="stylesheet" type="text/css" href="static/css/header.css";
            link rel="stylesheet" type="text/css" href="static/css/main.css";
        }
    }
}

fn header() -> Markup {
    html! {
        header {
            a href="index.html" class="header-button header-home-button" aria-label="Sway.rs" {
                svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" {
                    path d="M488.6 250.2L392 214V105.5c0-15-9.3-28.4-23.4-33.7l-100-37.5c-8.1-3.1-17.1-3.1-25.3 0l-100 37.5c-14.1 5.3-23.4 18.7-23.4 33.7V214l-96.6 36.2C9.3 255.5 0 268.9 0 283.9V394c0 13.6 7.7 26.1 19.9 32.2l100 50c10.1 5.1 22.1 5.1 32.2 0l103.9-52 103.9 52c10.1 5.1 22.1 5.1 32.2 0l100-50c12.2-6.1 19.9-18.6 19.9-32.2V283.9c0-15-9.3-28.4-23.4-33.7zM358 214.8l-85 31.9v-68.2l85-37v73.3zM154 104.1l102-38.2 102 38.2v.6l-102 41.4-102-41.4v-.6zm84 291.1l-85 42.5v-79.1l85-38.8v75.4zm0-112l-102 41.4-102-41.4v-.6l102-38.2 102 38.2v.6zm240 112l-85 42.5v-79.1l85-38.8v75.4zm0-112l-102 41.4-102-41.4v-.6l102-38.2 102 38.2v.6z";
                }
                span {
                    "SWAY.RS"
                }
            }
        }
    }
}
