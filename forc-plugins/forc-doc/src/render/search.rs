//! Generates the searchbar.
use horrorshow::{box_html, Raw, RenderBox};

use crate::doc::module::ModuleInfo;

pub(crate) fn generate_searchbar(module_info: ModuleInfo) -> Box<dyn RenderBox> {
    let path_to_root = module_info.path_to_root();
    box_html! {
        script(src=format!("{}/search.js", path_to_root), type="text/javascript");
        script {
            : Raw(format!(r#"
function onSearchFormSubmit(event) {{
    event.preventDefault();
    const searchQuery = document.getElementById("search-input").value;
    const url = new URL(window.location.href);
    url.searchParams.set('search', searchQuery);
    history.pushState({{ search: searchQuery }}, "", url);
    window.dispatchEvent(new HashChangeEvent("hashchange"));
}}

document.addEventListener('DOMContentLoaded', () => {{                
    const searchbar = document.getElementById("search-input");
    const searchForm = document.getElementById("search-form");
    searchbar.addEventListener("keyup", function(event) {{
        searchForm.dispatchEvent(new Event('submit'));
    }});

    function onQueryParamsChange() {{
        const searchParams = new URLSearchParams(window.location.search);
        const query = searchParams.get("search");
        const searchSection = document.getElementById('search');
        const mainSection = document.getElementById('main-content');
        const searchInput = document.getElementById('search-input');
        if (query) {{
            searchInput.value = query;
            const results = Object.values(SEARCH_INDEX).flat().filter(item => {{
                const lowerQuery = query.toLowerCase();
                return item.name.toLowerCase().includes(lowerQuery) || item.preview?.toLowerCase().includes(lowerQuery);
            }});
            const header = `<h1>Results for ${{query}}</h1>`;
            if (results.length > 0) {{
                const resultList = results.map(item => {{
                    const name = [...item.module_info, item.name].join("::");
                    const path = ["{}", ...item.module_info, item.html_filename].join("/");
                    const left = `<td style="padding-right:15px;"><a href="${{path}}"><code>${{name}}</code></a></td>`;
                    const right = `<td><a href="${{path}}">${{item.preview?.replace('<p>', '<p style="margin:0">') ?? ""}}</a></td>`;
                    return `<tr style="height:30px; white-space:nowrap; overflow:hidden;" onmouseover="this.style.background='darkgreen';" onmouseout="this.style.background='';">${{left}}${{right}}</tr>`;
                }}).join('');
                searchSection.innerHTML = `${{header}}<table>${{resultList}}</table>`;
            }} else {{
                searchSection.innerHTML = `${{header}}<p>No results found.</p>`;
            }}
            searchSection.removeAttribute("class", "hidden");
            mainSection.setAttribute("class", "content hidden");
        }} else {{
            searchSection.setAttribute("class", "content hidden");
            mainSection.removeAttribute("class", "hidden");
        }}
    }}
    window.addEventListener('hashchange', onQueryParamsChange);
    
    // Check for any query parameters initially
    onQueryParamsChange();
}});"#, path_to_root))
        }
        nav(class="sub") {
            form(id="search-form", class="search-form", onsubmit="onSearchFormSubmit(event)") {
                div(class="search-container") {
                    input(
                        id="search-input",
                        class="search-input",
                        name="search",
                        autocomplete="off",
                        spellcheck="false",
                        placeholder="Search the docs...",
                        type="search"
                    );
                }
            }
        }
    }
}
