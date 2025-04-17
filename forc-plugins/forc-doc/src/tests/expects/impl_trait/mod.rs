#![cfg(test)]
use crate::{
    cli::Command,
    compile_html,
    tests::expects::{check_file, get_doc_dir},
};
use dir_indexer::get_relative_file_paths_set;
use expect_test::{expect, Expect};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// The path to the generated HTML of the type the traits are implemented on.
const IMPL_FOR: &str = "bar/struct.Bar.html";
const DATA_DIR: &str = "src/tests/data";
const JS_SEARCH_FILE_PATH: &str = "search.js";

#[test]
fn test_impl_traits_default() {
    let doc_dir_name: &str = "impl_traits_default";
    let project_name = "impl_traits";
    let command = Command {
        path: Some(format!("{}/{}", DATA_DIR, project_name)),
        doc_path: Some(doc_dir_name.into()),
        ..Default::default()
    };
    let (doc_path, _) = compile_html(&command, &get_doc_dir).unwrap();
    assert_index_html(
        &doc_path,
        project_name,
        &expect![[r##"
            <!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="generator" content="swaydoc"><meta name="description" content="API documentation for the Sway `Bar` struct in `bar`."><meta name="keywords" content="sway, swaylang, sway-lang, Bar"><link rel="icon" href="../../static.files/sway-logo.svg"><title>Bar in bar - Sway</title><link rel="stylesheet" type="text/css" href="../../static.files/normalize.css"><link rel="stylesheet" type="text/css" href="../../static.files/swaydoc.css" id="mainThemeStyle"><link rel="stylesheet" type="text/css" href="../../static.files/ayu.css"><link rel="stylesheet" href="../../static.files/ayu.min.css"></head><body class="swaydoc struct"><nav class="sidebar"><a class="sidebar-logo" href="../../impl_traits/index.html"><div class="logo-container"><img class="sway-logo" src="../../static.files/sway-logo.svg" alt="logo"></div></a><h2 class="location">Struct Bar</h2><div class="sidebar-elems"><section><h3><a href="#methods">Methods</a></h3><ul class="block method"><li><a href="#method.foo_bar">foo_bar</a></li></ul></section><section><h3><a href="#trait-implementations">Trait Implementations</a></h3><ul class="block method"><li><a href="#impl-Foo">Foo</a></li><li><a href="#impl-Baz">Baz</a></li><li><a href="#impl-Add">Add</a></li><li><a href="#impl-Subtract">Subtract</a></li></ul></section></div></nav><main><div class="width-limiter"><script src="../../search.js" type="text/javascript"></script><script>function onSearchFormSubmit(event){event.preventDefault();const searchQuery=document.getElementById("search-input").value;const url=new URL(window.location.href);if(searchQuery){url.searchParams.set('search',searchQuery)}else{url.searchParams.delete('search')}history.pushState({search:searchQuery},"",url);window.dispatchEvent(new HashChangeEvent("hashchange"))}document.addEventListener('DOMContentLoaded',()=>{const searchbar=document.getElementById("search-input");const searchForm=document.getElementById("search-form");searchbar.addEventListener("keyup",function(event){onSearchFormSubmit(event)});searchbar.addEventListener("search",function(event){onSearchFormSubmit(event)});function onQueryParamsChange(){const searchParams=new URLSearchParams(window.location.search);const query=searchParams.get("search");const searchSection=document.getElementById('search');const mainSection=document.getElementById('main-content');const searchInput=document.getElementById('search-input');if(query){searchInput.value=query;const results=Object.values(SEARCH_INDEX).flat().filter(item=>{const lowerQuery=query.toLowerCase();return item.name.toLowerCase().includes(lowerQuery)});const header=`<h1>Results for ${query}</h1>`;if(results.length>0){const resultList=results.map(item=>{const formattedName=`<span class="type ${item.type_name}">${item.name}</span>`;const name=item.type_name==="module"?[...item.module_info.slice(0,-1),formattedName].join("::"):[...item.module_info,formattedName].join("::");const path=["../..",...item.module_info,item.html_filename].join("/");const left=`<td><span>${name}</span></td>`;const right=`<td><p>${item.preview}</p></td>`;return`<tr onclick="window.location='${path}';">${left}${right}</tr>`}).join('');searchSection.innerHTML=`${header}<table>${resultList}</table>`}else{searchSection.innerHTML=`${header}<p>No results found.</p>`}searchSection.setAttribute("class","search-results");mainSection.setAttribute("class","content hidden")}else{searchSection.setAttribute("class","search-results hidden");mainSection.setAttribute("class","content")}}window.addEventListener('hashchange',onQueryParamsChange);onQueryParamsChange()})</script><nav class="sub"><form id="search-form" class="search-form" onsubmit="onSearchFormSubmit(event)"><div class="search-container"><input id="search-input" class="search-input" name="search" autocomplete="off" spellcheck="false" placeholder="Search the docs..." type="search"></div></form></nav><section id="main-content" class="content"><div class="main-heading"><h1 class="fqn"><span class="in-band">Struct <a class="mod" href="../index.html">impl_traits</a><span>::</span><a class="mod" href="index.html">bar</a><span>::</span><a class="struct" href="#">Bar</a></span></h1></div><div class="docblock item-decl"><pre class="sway struct"><code>pub struct Bar {}</code></pre></div><h2 id="methods" class="small-section-header">Implementations<a href="#methods" class="anchor"></a></h2><div id="methods-list"><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Bar" class="impl has-srclink"><a href="#impl-Bar" class="anchor"></a><h3 class="code-header in-band">impl Bar</h3></div></summary><div class="impl-items"><div id="method.foo_bar" class="method trait-impl"><a href="#method.foo_bar" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.foo_bar">foo_bar</a>()</h4></div></div></details></div><h2 id="trait-implementations" class="small-section-header">Trait Implementations<a href="#trait-implementations" class="anchor"></a></h2><div id="trait-implementations-list"><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Foo" class="impl has-srclink"><a href="#impl-Foo" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Foo.html">Foo</a> for Bar</h3></div></summary><div class="impl-items"><details class="swaydoc-toggle method-toggle" open><summary><div id="method.foo" class="method trait-impl"><a href="#method.foo" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.foo">foo</a>()</h4></div></summary><div class="docblock"><p>something more about foo();</p>
            </div></details></div></details><div id="impl-Baz" class="impl has-srclink"><a href="#impl-Baz" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Baz.html">Baz</a> for Bar</h3></div><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Add" class="impl has-srclink"><a href="#impl-Add" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="..//trait.Add.html">Add</a> for Bar</h3></div></summary><div class="impl-items"><div id="method.add" class="method trait-impl"><a href="#method.add" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.add">add</a>(self, other: Self) -&gt; Self</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Subtract" class="impl has-srclink"><a href="#impl-Subtract" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../../ops/trait.Subtract.html">Subtract</a> for Bar</h3></div></summary><div class="impl-items"><div id="method.subtract" class="method trait-impl"><a href="#method.subtract" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.subtract">subtract</a>(self, other: Self) -&gt; Self</h4></div></div></details></div></section><section id="search" class="search-results"></section></div></main><script src="../../static.files/highlight.js"></script><script>hljs.highlightAll();</script></body></html>"##]],
    );
    assert_search_js(
        &doc_path,
        &expect![[
            r#"var SEARCH_INDEX={"impl_traits":[{"html_filename":"trait.Foo.html","module_info":["impl_traits","foo"],"name":"Foo","preview":"","type_name":"trait"},{"html_filename":"trait.Baz.html","module_info":["impl_traits","foo"],"name":"Baz","preview":"","type_name":"trait"},{"html_filename":"struct.Bar.html","module_info":["impl_traits","bar"],"name":"Bar","preview":"","type_name":"struct"},{"html_filename":"index.html","module_info":["impl_traits","bar"],"name":"bar","preview":"","type_name":"module"},{"html_filename":"index.html","module_info":["impl_traits","foo"],"name":"foo","preview":"","type_name":"module"}],"ops":[{"html_filename":"trait.Add.html","module_info":["ops"],"name":"Add","preview":"","type_name":"trait"},{"html_filename":"trait.Subtract.html","module_info":["ops"],"name":"Subtract","preview":"","type_name":"trait"},{"html_filename":"index.html","module_info":["ops"],"name":"ops","preview":"","type_name":"module"}]};
"object"==typeof exports&&"undefined"!=typeof module&&(module.exports=SEARCH_INDEX);"#
        ]],
    );
    assert_file_tree(
        doc_dir_name,
        project_name,
        vec![
            "impl_traits/foo/trait.Foo.html",
            "impl_traits/foo/index.html",
            "impl_traits/all.html",
            "ops/trait.Subtract.html",
            "ops/all.html",
            "impl_traits/bar/struct.Bar.html",
            "impl_traits/bar/index.html",
            "ops/trait.Add.html",
            "search.js",
            "impl_traits/index.html",
            "ops/index.html",
            "impl_traits/foo/trait.Baz.html",
        ],
    );
}

#[test]
fn test_impl_traits_no_deps() {
    let doc_dir_name: &str = "impl_traits_no_deps";
    let project_name: &str = "impl_traits_generic";
    let command = Command {
        path: Some(format!("{}/{}", DATA_DIR, project_name)),
        doc_path: Some(doc_dir_name.into()),
        no_deps: true,
        ..Default::default()
    };
    let (doc_path, _) = compile_html(&command, &get_doc_dir).unwrap();
    assert_index_html(
        &doc_path,
        project_name,
        &expect![[r##"
            <!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="generator" content="swaydoc"><meta name="description" content="API documentation for the Sway `Bar` struct in `bar`."><meta name="keywords" content="sway, swaylang, sway-lang, Bar"><link rel="icon" href="../../static.files/sway-logo.svg"><title>Bar in bar - Sway</title><link rel="stylesheet" type="text/css" href="../../static.files/normalize.css"><link rel="stylesheet" type="text/css" href="../../static.files/swaydoc.css" id="mainThemeStyle"><link rel="stylesheet" type="text/css" href="../../static.files/ayu.css"><link rel="stylesheet" href="../../static.files/ayu.min.css"></head><body class="swaydoc struct"><nav class="sidebar"><a class="sidebar-logo" href="../../impl_traits_generic/index.html"><div class="logo-container"><img class="sway-logo" src="../../static.files/sway-logo.svg" alt="logo"></div></a><h2 class="location">Struct Bar</h2><div class="sidebar-elems"><section><h3><a href="#trait-implementations">Trait Implementations</a></h3><ul class="block method"><li><a href="#impl-Foo">Foo</a></li><li><a href="#impl-Baz">Baz</a></li><li><a href="#impl-Bar">Bar</a></li><li><a href="#impl-Add">Add</a></li><li><a href="#impl-Subtract">Subtract</a></li></ul></section></div></nav><main><div class="width-limiter"><script src="../../search.js" type="text/javascript"></script><script>function onSearchFormSubmit(event){event.preventDefault();const searchQuery=document.getElementById("search-input").value;const url=new URL(window.location.href);if(searchQuery){url.searchParams.set('search',searchQuery)}else{url.searchParams.delete('search')}history.pushState({search:searchQuery},"",url);window.dispatchEvent(new HashChangeEvent("hashchange"))}document.addEventListener('DOMContentLoaded',()=>{const searchbar=document.getElementById("search-input");const searchForm=document.getElementById("search-form");searchbar.addEventListener("keyup",function(event){onSearchFormSubmit(event)});searchbar.addEventListener("search",function(event){onSearchFormSubmit(event)});function onQueryParamsChange(){const searchParams=new URLSearchParams(window.location.search);const query=searchParams.get("search");const searchSection=document.getElementById('search');const mainSection=document.getElementById('main-content');const searchInput=document.getElementById('search-input');if(query){searchInput.value=query;const results=Object.values(SEARCH_INDEX).flat().filter(item=>{const lowerQuery=query.toLowerCase();return item.name.toLowerCase().includes(lowerQuery)});const header=`<h1>Results for ${query}</h1>`;if(results.length>0){const resultList=results.map(item=>{const formattedName=`<span class="type ${item.type_name}">${item.name}</span>`;const name=item.type_name==="module"?[...item.module_info.slice(0,-1),formattedName].join("::"):[...item.module_info,formattedName].join("::");const path=["../..",...item.module_info,item.html_filename].join("/");const left=`<td><span>${name}</span></td>`;const right=`<td><p>${item.preview}</p></td>`;return`<tr onclick="window.location='${path}';">${left}${right}</tr>`}).join('');searchSection.innerHTML=`${header}<table>${resultList}</table>`}else{searchSection.innerHTML=`${header}<p>No results found.</p>`}searchSection.setAttribute("class","search-results");mainSection.setAttribute("class","content hidden")}else{searchSection.setAttribute("class","search-results hidden");mainSection.setAttribute("class","content")}}window.addEventListener('hashchange',onQueryParamsChange);onQueryParamsChange()})</script><nav class="sub"><form id="search-form" class="search-form" onsubmit="onSearchFormSubmit(event)"><div class="search-container"><input id="search-input" class="search-input" name="search" autocomplete="off" spellcheck="false" placeholder="Search the docs..." type="search"></div></form></nav><section id="main-content" class="content"><div class="main-heading"><h1 class="fqn"><span class="in-band">Struct <a class="mod" href="../index.html">impl_traits_generic</a><span>::</span><a class="mod" href="index.html">bar</a><span>::</span><a class="struct" href="#">Bar</a></span></h1></div><div class="docblock item-decl"><pre class="sway struct"><code>pub struct Bar&lt;T&gt; {}</code></pre></div><h2 id="trait-implementations" class="small-section-header">Trait Implementations<a href="#trait-implementations" class="anchor"></a></h2><div id="trait-implementations-list"><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Foo" class="impl has-srclink"><a href="#impl-Foo" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Foo.html">Foo</a> for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><details class="swaydoc-toggle method-toggle" open><summary><div id="method.foo" class="method trait-impl"><a href="#method.foo" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.foo">foo</a>()</h4></div></summary><div class="docblock"><p>something more about foo();</p>
            </div></details></div></details><div id="impl-Baz" class="impl has-srclink"><a href="#impl-Baz" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Baz.html">Baz</a> for Bar&lt;T&gt;</h3></div><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Bar" class="impl has-srclink"><a href="#impl-Bar" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../bar/trait.Bar.html">Bar</a> for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><div id="method.foo_bar" class="method trait-impl"><a href="#method.foo_bar" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.foo_bar">foo_bar</a>()</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Add" class="impl has-srclink"><a href="#impl-Add" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="..//trait.Add.html">Add</a> for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><div id="method.add" class="method trait-impl"><a href="#method.add" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.add">add</a>(self, other: Self) -&gt; Self</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Subtract" class="impl has-srclink"><a href="#impl-Subtract" class="anchor"></a><h3 class="code-header in-band">impl Subtract for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><div id="method.subtract" class="method trait-impl"><a href="#method.subtract" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.subtract">subtract</a>(self, other: Self) -&gt; Self</h4></div></div></details></div></section><section id="search" class="search-results"></section></div></main><script src="../../static.files/highlight.js"></script><script>hljs.highlightAll();</script></body></html>"##]],
    );
    assert_search_js(
        &doc_path,
        &expect![[
            r#"var SEARCH_INDEX={"impl_traits_generic":[{"html_filename":"trait.Foo.html","module_info":["impl_traits_generic","foo"],"name":"Foo","preview":"","type_name":"trait"},{"html_filename":"trait.Baz.html","module_info":["impl_traits_generic","foo"],"name":"Baz","preview":"","type_name":"trait"},{"html_filename":"struct.Bar.html","module_info":["impl_traits_generic","bar"],"name":"Bar","preview":"","type_name":"struct"},{"html_filename":"index.html","module_info":["impl_traits_generic","bar"],"name":"bar","preview":"","type_name":"module"},{"html_filename":"index.html","module_info":["impl_traits_generic","foo"],"name":"foo","preview":"","type_name":"module"}]};
"object"==typeof exports&&"undefined"!=typeof module&&(module.exports=SEARCH_INDEX);"#
        ]],
    );
    assert_file_tree(
        doc_dir_name,
        project_name,
        vec![
            "impl_traits_generic/index.html",
            "impl_traits_generic/all.html",
            "impl_traits_generic/foo/trait.Foo.html",
            "impl_traits_generic/bar/index.html",
            "impl_traits_generic/foo/index.html",
            "impl_traits_generic/foo/trait.Baz.html",
            "search.js",
            "impl_traits_generic/bar/struct.Bar.html",
        ],
    );
}

fn assert_index_html(doc_path: &Path, project_name: &str, expect: &Expect) {
    let path_to_file = PathBuf::from(format!("{}/{}", project_name, IMPL_FOR));
    check_file(doc_path, &path_to_file, expect);
}

fn assert_search_js(doc_path: &Path, expect: &Expect) {
    let path_to_file = PathBuf::from(JS_SEARCH_FILE_PATH);
    check_file(doc_path, &path_to_file, expect);
}

fn assert_file_tree(doc_dir_name: &str, project_name: &str, expected_files: Vec<&str>) {
    let doc_root: PathBuf = format!("{}/{}/out/{}", DATA_DIR, project_name, doc_dir_name).into();
    let expected = expected_files
        .iter()
        .map(PathBuf::from)
        .collect::<HashSet<PathBuf>>();
    let files = get_relative_file_paths_set(doc_root.clone());
    if files != expected {
        let diffs = files.symmetric_difference(&expected);
        assert_eq!(
            files, expected,
            "Symmetric Difference: {diffs:?} at {doc_root:?}"
        );
    }
}
