//! Generates the searchbar.
use horrorshow::{box_html, Raw, RenderBox};

use crate::doc::module::ModuleInfo;

pub(crate) fn generate_searchbar(module_info: ModuleInfo) -> Box<dyn RenderBox> {
    let path_to_root = module_info.path_to_root();
    box_html! {
        script(src=format!("{}/search.js", path_to_root), type="text/javascript");
        script {
            : Raw(format!(r#"
                document.addEventListener('DOMContentLoaded', () => {{
                const searchbar = document.getElementById("search-input");
                searchbar.addEventListener("change", function() {{
                    document.getElementById("search-form").submit();
                }});
                
                function onQueryParamsChange() {{
                    const searchParams = new URLSearchParams(window.location.search);
                    const query = searchParams.get("search");
                    const searchSection = document.getElementById('search');
                    const mainSection = document.getElementById('main-content');
                    if (query) {{
                        const results = Object.values(SEARCH_INDEX).flat().filter(item => item.name.toLowerCase().includes(query.toLowerCase()))
                        console.log("Search results:", results);
                        const header = `<h1>Results for ${{query}}</h1>`;
                        if (results.length > 0) {{
                            const resultList = results.map(item => {{
                                const name = [...item.module_info, item.name].join("::");
                                const path = ["{}", ...item.module_info, item.html_filename].join("/");
                                const left = `<td style="padding-right:15px;"><code>${{name}}</code></a></td>`;
                                const right = `<td><a href="${{path}}">${{item.preview ?? ""}}</a></td>`;
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
            form(id="search-form", class="search-form") {
                div(class="search-container") {
                    span;
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
