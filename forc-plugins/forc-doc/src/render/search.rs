//! Generates the searchbar.
use crate::doc::module::ModuleInfo;
use horrorshow::{box_html, Raw, RenderBox};
use minifier::js::minify;

pub(crate) fn generate_searchbar(module_info: &ModuleInfo) -> Box<dyn RenderBox> {
    let path_to_root = module_info.path_to_root();
    // Since this searchbar is rendered on all pages, we need to inject the path the root into the script.
    // Therefore, we can't simply import this script from a javascript file.
    let minified_script = minify(&format!(r#"
        function onSearchFormSubmit(event) {{
            event.preventDefault();
            const searchQuery = document.getElementById("search-input").value;
            const url = new URL(window.location.href);
            if (searchQuery) {{
                url.searchParams.set('search', searchQuery);
            }} else {{
                url.searchParams.delete('search');
            }}
            history.pushState({{ search: searchQuery }}, "", url);
            window.dispatchEvent(new HashChangeEvent("hashchange"));
        }}
        
        document.addEventListener('DOMContentLoaded', () => {{                
            const searchbar = document.getElementById("search-input");
            const searchForm = document.getElementById("search-form");
            searchbar.addEventListener("keyup", function(event) {{
                onSearchFormSubmit(event);
            }});
            searchbar.addEventListener("search", function(event) {{
                onSearchFormSubmit(event);
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
                        return item.name.toLowerCase().includes(lowerQuery);
                    }});
                    const header = `<h1>Results for ${{query}}</h1>`;
                    if (results.length > 0) {{
                        const resultList = results.map(item => {{
                            const formattedName = `<span class="type ${{item.type_name}}">${{item.name}}</span>`;
                            const name = item.type_name === "module"
                                ? [...item.module_info.slice(0, -1), formattedName].join("::")
                                : [...item.module_info, formattedName].join("::");
                            const path = ["{path_to_root}", ...item.module_info, item.html_filename].join("/");
                            const left = `<td><span>${{name}}</span></td>`;
                            const right = `<td><p>${{item.preview}}</p></td>`;
                            return `<tr onclick="window.location='${{path}}';">${{left}}${{right}}</tr>`;
                        }}).join('');
                        searchSection.innerHTML = `${{header}}<table>${{resultList}}</table>`;
                    }} else {{
                        searchSection.innerHTML = `${{header}}<p>No results found.</p>`;
                    }}
                    searchSection.setAttribute("class", "search-results");
                    mainSection.setAttribute("class", "content hidden");
                }} else {{
                    searchSection.setAttribute("class", "search-results hidden");
                    mainSection.setAttribute("class", "content");
                }}
            }}
            window.addEventListener('hashchange', onQueryParamsChange);
            
            // Check for any query parameters initially
            onQueryParamsChange();
        }}
    );"#)).to_string();
    box_html! {
        script(src=format!("{}/search.js", path_to_root), type="text/javascript");
        script {
            : Raw(minified_script)
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
