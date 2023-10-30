//! Generates the searchbar.
use horrorshow::{box_html, RenderBox};

// TODO: Implement Searchbar
// - Add search functionality to search bar
// - Add help.html support
// - Add settings.html support
pub(crate) fn generate_searchbar() -> Box<dyn RenderBox> {
    box_html! {
        nav(class="sub") {
            form(class="search-form") {
                div(class="search-container") {
                    span;
                    input(
                        class="search-input",
                        name="search",
                        autocomplete="off",
                        spellcheck="false",
                        placeholder="Click or press ‘S’ to search, ‘?’ for more options…",
                        type="search"
                    );
                    // div(id="help-button", title="help", tabindex="-1") {
                    //     a(href="../help.html") { : "?" }
                    // }
                    // div(id="settings-menu", tabindex="-1") {
                    //     a(href="../settings.html", title="settings") {
                    //         img(
                    //             width="22",
                    //             height="22",
                    //             alt="change settings",
                    //             src="../static.files/wheel.svg"
                    //         )
                    //     }
                    // }
                }
            }
        }
    }
}
