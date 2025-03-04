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
    <!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="generator" content="swaydoc"><meta name="description" content="API documentation for the Sway `Bar` struct in `bar`."><meta name="keywords" content="sway, swaylang, sway-lang, Bar"><link rel="icon" href="../../static.files/sway-logo.svg"><title>Bar in bar - Sway</title><link rel="stylesheet" type="text/css" href="../../static.files/normalize.css"><link rel="stylesheet" type="text/css" href="../../static.files/swaydoc.css" id="mainThemeStyle"><link rel="stylesheet" type="text/css" href="../../static.files/ayu.css"><link rel="stylesheet" href="../../static.files/ayu.min.css"></head><body class="swaydoc struct"><nav class="sidebar"><a class="sidebar-logo" href="../../impl_traits/index.html"><div class="logo-container"><img class="sway-logo" src="../../static.files/sway-logo.svg" alt="logo"></div></a><h2 class="location">Struct Bar</h2><div class="sidebar-elems"><section><h3><a href="#methods">Methods</a></h3><ul class="block method"><li><a href="#method.foo_bar">foo_bar</a></li></ul></section><section><h3><a href="#trait-implementations">Trait Implementations</a></h3><ul class="block method"><li><a href="#impl-AbiEncode">AbiEncode</a></li><li><a href="#impl-AbiDecode">AbiDecode</a></li><li><a href="#impl-Foo">Foo</a></li><li><a href="#impl-Baz">Baz</a></li><li><a href="#impl-Add">Add</a></li><li><a href="#impl-Subtract">Subtract</a></li></ul></section></div></nav><main><div class="width-limiter"><script src="../../search.js" type="text/javascript"></script><script>function onSearchFormSubmit(event){event.preventDefault();const searchQuery=document.getElementById("search-input").value;const url=new URL(window.location.href);if(searchQuery){url.searchParams.set('search',searchQuery)}else{url.searchParams.delete('search')}history.pushState({search:searchQuery},"",url);window.dispatchEvent(new HashChangeEvent("hashchange"))}document.addEventListener('DOMContentLoaded',()=>{const searchbar=document.getElementById("search-input");const searchForm=document.getElementById("search-form");searchbar.addEventListener("keyup",function(event){onSearchFormSubmit(event)});searchbar.addEventListener("search",function(event){onSearchFormSubmit(event)});function onQueryParamsChange(){const searchParams=new URLSearchParams(window.location.search);const query=searchParams.get("search");const searchSection=document.getElementById('search');const mainSection=document.getElementById('main-content');const searchInput=document.getElementById('search-input');if(query){searchInput.value=query;const results=Object.values(SEARCH_INDEX).flat().filter(item=>{const lowerQuery=query.toLowerCase();return item.name.toLowerCase().includes(lowerQuery)});const header=`<h1>Results for ${query}</h1>`;if(results.length>0){const resultList=results.map(item=>{const formattedName=`<span class="type ${item.type_name}">${item.name}</span>`;const name=item.type_name==="module"?[...item.module_info.slice(0,-1),formattedName].join("::"):[...item.module_info,formattedName].join("::");const path=["../..",...item.module_info,item.html_filename].join("/");const left=`<td><span>${name}</span></td>`;const right=`<td><p>${item.preview}</p></td>`;return`<tr onclick="window.location='${path}';">${left}${right}</tr>`}).join('');searchSection.innerHTML=`${header}<table>${resultList}</table>`}else{searchSection.innerHTML=`${header}<p>No results found.</p>`}searchSection.setAttribute("class","search-results");mainSection.setAttribute("class","content hidden")}else{searchSection.setAttribute("class","search-results hidden");mainSection.setAttribute("class","content")}}window.addEventListener('hashchange',onQueryParamsChange);onQueryParamsChange()})</script><nav class="sub"><form id="search-form" class="search-form" onsubmit="onSearchFormSubmit(event)"><div class="search-container"><input id="search-input" class="search-input" name="search" autocomplete="off" spellcheck="false" placeholder="Search the docs..." type="search"></div></form></nav><section id="main-content" class="content"><div class="main-heading"><h1 class="fqn"><span class="in-band">Struct <a class="mod" href="../index.html">impl_traits</a><span>::</span><a class="mod" href="index.html">bar</a><span>::</span><a class="struct" href="#">Bar</a></span></h1></div><div class="docblock item-decl"><pre class="sway struct"><code>pub struct Bar {}</code></pre></div><h2 id="methods" class="small-section-header">Implementations<a href="#methods" class="anchor"></a></h2><div id="methods-list"><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Bar" class="impl has-srclink"><a href="#impl-Bar" class="anchor"></a><h3 class="code-header in-band">impl Bar</h3></div></summary><div class="impl-items"><div id="method.foo_bar" class="method trait-impl"><a href="#method.foo_bar" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.foo_bar">foo_bar</a>()</h4></div></div></details></div><h2 id="trait-implementations" class="small-section-header">Trait Implementations<a href="#trait-implementations" class="anchor"></a></h2><div id="trait-implementations-list"><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-AbiEncode" class="impl has-srclink"><a href="#impl-AbiEncode" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../../std/codec/trait.AbiEncode.html">AbiEncode</a> for Bar</h3></div></summary><div class="impl-items"><div id="method.abi_encode" class="method trait-impl"><a href="#method.abi_encode" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.abi_encode">abi_encode</a>(self, buffer: Buffer) -&gt; Buffer</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-AbiDecode" class="impl has-srclink"><a href="#impl-AbiDecode" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../../std/codec/trait.AbiDecode.html">AbiDecode</a> for Bar</h3></div></summary><div class="impl-items"><div id="method.abi_decode" class="method trait-impl"><a href="#method.abi_decode" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.abi_decode">abi_decode</a>(refmut _buffer: BufferReader) -&gt; Self</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Foo" class="impl has-srclink"><a href="#impl-Foo" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Foo.html">Foo</a> for Bar</h3></div></summary><div class="impl-items"><details class="swaydoc-toggle method-toggle" open><summary><div id="method.foo" class="method trait-impl"><a href="#method.foo" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.foo">foo</a>()</h4></div></summary><div class="docblock"><p>something more about foo();</p>
    </div></details></div></details><div id="impl-Baz" class="impl has-srclink"><a href="#impl-Baz" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Baz.html">Baz</a> for Bar</h3></div><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Add" class="impl has-srclink"><a href="#impl-Add" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../../std/ops/trait.Add.html">Add</a> for Bar</h3></div></summary><div class="impl-items"><div id="method.add" class="method trait-impl"><a href="#method.add" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.add">add</a>(self, other: Self) -&gt; Self</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Subtract" class="impl has-srclink"><a href="#impl-Subtract" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../../std/ops/trait.Subtract.html">Subtract</a> for Bar</h3></div></summary><div class="impl-items"><div id="method.subtract" class="method trait-impl"><a href="#method.subtract" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.subtract">subtract</a>(self, other: Self) -&gt; Self</h4></div></div></details></div></section><section id="search" class="search-results"></section></div></main><script src="../../static.files/highlight.js"></script><script>hljs.highlightAll();</script></body></html>"##]],
    );
    assert_search_js(
        &doc_path,
        &expect![[r#"
            var SEARCH_INDEX={"std":[{"html_filename":"trait.AsRawSlice.html","module_info":["std","raw_slice"],"name":"AsRawSlice","preview":"Trait to return a type as a <code>raw_slice</code>.\n","type_name":"trait"},{"html_filename":"fn.from_str_array.html","module_info":["std","str"],"name":"from_str_array","preview":"","type_name":"function"},{"html_filename":"trait.Add.html","module_info":["std","ops"],"name":"Add","preview":"Trait for the addition of two values.\n","type_name":"trait"},{"html_filename":"trait.Subtract.html","module_info":["std","ops"],"name":"Subtract","preview":"Trait for the subtraction of two values.\n","type_name":"trait"},{"html_filename":"trait.Multiply.html","module_info":["std","ops"],"name":"Multiply","preview":"Trait for the multiplication of two values.\n","type_name":"trait"},{"html_filename":"trait.Divide.html","module_info":["std","ops"],"name":"Divide","preview":"Trait for the division of two values.\n","type_name":"trait"},{"html_filename":"trait.Mod.html","module_info":["std","ops"],"name":"Mod","preview":"Trait for the modulo of two values.\n","type_name":"trait"},{"html_filename":"trait.Not.html","module_info":["std","ops"],"name":"Not","preview":"Trait to invert a type.\n","type_name":"trait"},{"html_filename":"trait.Eq.html","module_info":["std","ops"],"name":"Eq","preview":"Trait to evaluate if two types are equal.\n","type_name":"trait"},{"html_filename":"trait.Ord.html","module_info":["std","ops"],"name":"Ord","preview":"Trait to evaluate if one value is greater or less than another of the same type.\n","type_name":"trait"},{"html_filename":"trait.BitwiseAnd.html","module_info":["std","ops"],"name":"BitwiseAnd","preview":"Trait to bitwise AND two values of the same type.\n","type_name":"trait"},{"html_filename":"trait.BitwiseOr.html","module_info":["std","ops"],"name":"BitwiseOr","preview":"Trait to bitwise OR two values of the same type.\n","type_name":"trait"},{"html_filename":"trait.BitwiseXor.html","module_info":["std","ops"],"name":"BitwiseXor","preview":"Trait to bitwise XOR two values of the same type.\n","type_name":"trait"},{"html_filename":"trait.OrdEq.html","module_info":["std","ops"],"name":"OrdEq","preview":"Trait to evaluate if one value is greater than or equal, or less than or equal to another of the same type.","type_name":"trait"},{"html_filename":"trait.Shift.html","module_info":["std","ops"],"name":"Shift","preview":"Trait to bit shift a value.\n","type_name":"trait"},{"html_filename":"trait.TotalOrd.html","module_info":["std","ops"],"name":"TotalOrd","preview":"Trait to compare values of the same type.\n","type_name":"trait"},{"html_filename":"fn.ok_str_eq.html","module_info":["std","ops"],"name":"ok_str_eq","preview":"","type_name":"function"},{"html_filename":"struct.StorageKey.html","module_info":["std","storage"],"name":"StorageKey","preview":"Describes a location in storage.\n","type_name":"struct"},{"html_filename":"struct.Buffer.html","module_info":["std","codec"],"name":"Buffer","preview":"","type_name":"struct"},{"html_filename":"struct.BufferReader.html","module_info":["std","codec"],"name":"BufferReader","preview":"","type_name":"struct"},{"html_filename":"trait.AbiDecode.html","module_info":["std","codec"],"name":"AbiDecode","preview":"","type_name":"trait"},{"html_filename":"trait.AbiEncode.html","module_info":["std","codec"],"name":"AbiEncode","preview":"","type_name":"trait"},{"html_filename":"fn.encode.html","module_info":["std","codec"],"name":"encode","preview":"","type_name":"function"},{"html_filename":"fn.abi_decode.html","module_info":["std","codec"],"name":"abi_decode","preview":"","type_name":"function"},{"html_filename":"fn.abi_decode_in_place.html","module_info":["std","codec"],"name":"abi_decode_in_place","preview":"","type_name":"function"},{"html_filename":"fn.contract_call.html","module_info":["std","codec"],"name":"contract_call","preview":"","type_name":"function"},{"html_filename":"fn.decode_script_data.html","module_info":["std","codec"],"name":"decode_script_data","preview":"","type_name":"function"},{"html_filename":"fn.decode_predicate_data.html","module_info":["std","codec"],"name":"decode_predicate_data","preview":"","type_name":"function"},{"html_filename":"fn.decode_predicate_data_by_index.html","module_info":["std","codec"],"name":"decode_predicate_data_by_index","preview":"","type_name":"function"},{"html_filename":"fn.decode_first_param.html","module_info":["std","codec"],"name":"decode_first_param","preview":"","type_name":"function"},{"html_filename":"fn.decode_second_param.html","module_info":["std","codec"],"name":"decode_second_param","preview":"","type_name":"function"},{"html_filename":"primitive.u256.html","module_info":["std"],"name":"u256","preview":"256-bit unsigned integer","type_name":"primitive"},{"html_filename":"primitive.u64.html","module_info":["std"],"name":"u64","preview":"64-bit unsigned integer","type_name":"primitive"},{"html_filename":"primitive.u32.html","module_info":["std"],"name":"u32","preview":"32-bit unsigned integer","type_name":"primitive"},{"html_filename":"primitive.u16.html","module_info":["std"],"name":"u16","preview":"16-bit unsigned integer","type_name":"primitive"},{"html_filename":"primitive.u8.html","module_info":["std"],"name":"u8","preview":"8-bit unsigned integer","type_name":"primitive"},{"html_filename":"primitive.b256.html","module_info":["std"],"name":"b256","preview":"256 bits (32 bytes), i.e. a hash","type_name":"primitive"},{"html_filename":"primitive.str.html","module_info":["std"],"name":"str","preview":"string slice","type_name":"primitive"},{"html_filename":"primitive.bool.html","module_info":["std"],"name":"bool","preview":"Boolean true or false","type_name":"primitive"},{"html_filename":"primitive.str[0].html","module_info":["std"],"name":"str[0]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[1].html","module_info":["std"],"name":"str[1]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[2].html","module_info":["std"],"name":"str[2]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[3].html","module_info":["std"],"name":"str[3]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[4].html","module_info":["std"],"name":"str[4]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[5].html","module_info":["std"],"name":"str[5]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[6].html","module_info":["std"],"name":"str[6]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[7].html","module_info":["std"],"name":"str[7]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[8].html","module_info":["std"],"name":"str[8]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[9].html","module_info":["std"],"name":"str[9]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[10].html","module_info":["std"],"name":"str[10]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[11].html","module_info":["std"],"name":"str[11]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[12].html","module_info":["std"],"name":"str[12]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[13].html","module_info":["std"],"name":"str[13]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[14].html","module_info":["std"],"name":"str[14]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[15].html","module_info":["std"],"name":"str[15]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[16].html","module_info":["std"],"name":"str[16]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[17].html","module_info":["std"],"name":"str[17]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[18].html","module_info":["std"],"name":"str[18]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[19].html","module_info":["std"],"name":"str[19]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[20].html","module_info":["std"],"name":"str[20]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[21].html","module_info":["std"],"name":"str[21]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[22].html","module_info":["std"],"name":"str[22]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[23].html","module_info":["std"],"name":"str[23]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[24].html","module_info":["std"],"name":"str[24]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[25].html","module_info":["std"],"name":"str[25]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[26].html","module_info":["std"],"name":"str[26]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[27].html","module_info":["std"],"name":"str[27]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[28].html","module_info":["std"],"name":"str[28]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[29].html","module_info":["std"],"name":"str[29]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[30].html","module_info":["std"],"name":"str[30]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[31].html","module_info":["std"],"name":"str[31]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[32].html","module_info":["std"],"name":"str[32]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[33].html","module_info":["std"],"name":"str[33]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[34].html","module_info":["std"],"name":"str[34]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[35].html","module_info":["std"],"name":"str[35]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[36].html","module_info":["std"],"name":"str[36]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[37].html","module_info":["std"],"name":"str[37]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[38].html","module_info":["std"],"name":"str[38]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[39].html","module_info":["std"],"name":"str[39]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[40].html","module_info":["std"],"name":"str[40]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[41].html","module_info":["std"],"name":"str[41]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[42].html","module_info":["std"],"name":"str[42]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[43].html","module_info":["std"],"name":"str[43]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[44].html","module_info":["std"],"name":"str[44]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[45].html","module_info":["std"],"name":"str[45]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[46].html","module_info":["std"],"name":"str[46]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[47].html","module_info":["std"],"name":"str[47]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[48].html","module_info":["std"],"name":"str[48]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[49].html","module_info":["std"],"name":"str[49]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[50].html","module_info":["std"],"name":"str[50]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[51].html","module_info":["std"],"name":"str[51]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[52].html","module_info":["std"],"name":"str[52]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[53].html","module_info":["std"],"name":"str[53]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[54].html","module_info":["std"],"name":"str[54]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[55].html","module_info":["std"],"name":"str[55]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[56].html","module_info":["std"],"name":"str[56]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[57].html","module_info":["std"],"name":"str[57]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[58].html","module_info":["std"],"name":"str[58]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[59].html","module_info":["std"],"name":"str[59]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[60].html","module_info":["std"],"name":"str[60]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[61].html","module_info":["std"],"name":"str[61]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[62].html","module_info":["std"],"name":"str[62]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[63].html","module_info":["std"],"name":"str[63]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"primitive.str[64].html","module_info":["std"],"name":"str[64]","preview":"fixed-length string","type_name":"primitive"},{"html_filename":"index.html","module_info":["std"],"name":"std","preview":"","type_name":"module"},{"html_filename":"index.html","module_info":["std","codec"],"name":"codec","preview":"","type_name":"module"},{"html_filename":"index.html","module_info":["std","ops"],"name":"ops","preview":"","type_name":"module"},{"html_filename":"index.html","module_info":["std","raw_slice"],"name":"raw_slice","preview":"","type_name":"module"},{"html_filename":"index.html","module_info":["std","storage"],"name":"storage","preview":"","type_name":"module"},{"html_filename":"index.html","module_info":["std","str"],"name":"str","preview":"","type_name":"module"}],"impl_traits":[{"html_filename":"trait.Foo.html","module_info":["impl_traits","foo"],"name":"Foo","preview":"","type_name":"trait"},{"html_filename":"trait.Baz.html","module_info":["impl_traits","foo"],"name":"Baz","preview":"","type_name":"trait"},{"html_filename":"struct.Bar.html","module_info":["impl_traits","bar"],"name":"Bar","preview":"","type_name":"struct"},{"html_filename":"index.html","module_info":["impl_traits","bar"],"name":"bar","preview":"","type_name":"module"},{"html_filename":"index.html","module_info":["impl_traits","foo"],"name":"foo","preview":"","type_name":"module"}]};
            "object"==typeof exports&&"undefined"!=typeof module&&(module.exports=SEARCH_INDEX);"#]],
    );
    assert_file_tree(
        doc_dir_name,
        project_name,
        vec![
            "std/ops/trait.TotalOrd.html",
            "std/ops/trait.Mod.html",
            "impl_traits/bar/index.html",
            "impl_traits/all.html",
            "std/primitive.str[17].html",
            "std/primitive.str[51].html",
            "std/codec/fn.abi_decode.html",
            "std/codec/struct.Buffer.html",
            "std/codec/trait.AbiDecode.html",
            "std/ops/trait.Divide.html",
            "std/codec/fn.abi_decode_in_place.html",
            "std/primitive.str[34].html",
            "std/codec/fn.decode_first_param.html",
            "std/primitive.str[20].html",
            "std/primitive.str[4].html",
            "std/primitive.str[42].html",
            "std/primitive.str[10].html",
            "std/ops/trait.Subtract.html",
            "std/primitive.str[28].html",
            "std/primitive.u8.html",
            "std/primitive.u256.html",
            "std/str/fn.from_str_array.html",
            "std/primitive.str[62].html",
            "std/primitive.str[64].html",
            "std/codec/trait.AbiEncode.html",
            "std/primitive.str[8].html",
            "std/primitive.str[43].html",
            "std/codec/fn.decode_predicate_data_by_index.html",
            "std/primitive.str[37].html",
            "std/primitive.b256.html",
            "std/ops/trait.Eq.html",
            "std/storage/index.html",
            "std/primitive.str[58].html",
            "std/primitive.str[44].html",
            "std/raw_slice/index.html",
            "std/primitive.u16.html",
            "std/primitive.str[35].html",
            "std/primitive.str[60].html",
            "std/primitive.str[59].html",
            "std/primitive.str[39].html",
            "std/primitive.str[49].html",
            "std/primitive.str[40].html",
            "std/index.html",
            "search.js",
            "std/ops/trait.OrdEq.html",
            "std/primitive.bool.html",
            "impl_traits/foo/trait.Baz.html",
            "std/primitive.str.html",
            "std/primitive.str[13].html",
            "std/primitive.u64.html",
            "std/primitive.str[16].html",
            "std/codec/index.html",
            "std/primitive.str[47].html",
            "std/primitive.str[57].html",
            "std/ops/trait.Not.html",
            "std/primitive.str[50].html",
            "std/raw_slice/trait.AsRawSlice.html",
            "std/primitive.str[24].html",
            "std/primitive.str[56].html",
            "std/ops/index.html",
            "std/primitive.str[3].html",
            "std/primitive.str[36].html",
            "std/primitive.str[27].html",
            "std/codec/fn.decode_script_data.html",
            "impl_traits/index.html",
            "std/primitive.str[12].html",
            "impl_traits/foo/trait.Foo.html",
            "std/primitive.str[15].html",
            "std/primitive.str[29].html",
            "std/primitive.str[22].html",
            "std/ops/trait.BitwiseAnd.html",
            "std/all.html",
            "std/primitive.str[11].html",
            "std/ops/trait.Ord.html",
            "std/str/index.html",
            "std/primitive.str[31].html",
            "impl_traits/foo/index.html",
            "std/primitive.str[55].html",
            "std/primitive.str[38].html",
            "std/codec/struct.BufferReader.html",
            "std/primitive.str[54].html",
            "std/primitive.str[53].html",
            "std/primitive.str[19].html",
            "std/storage/struct.StorageKey.html",
            "std/primitive.str[25].html",
            "std/primitive.str[30].html",
            "std/codec/fn.encode.html",
            "std/primitive.str[26].html",
            "std/primitive.str[41].html",
            "std/codec/fn.decode_second_param.html",
            "std/primitive.u32.html",
            "std/ops/fn.ok_str_eq.html",
            "std/primitive.str[1].html",
            "std/primitive.str[18].html",
            "std/primitive.str[33].html",
            "std/primitive.str[0].html",
            "std/codec/fn.decode_predicate_data.html",
            "std/primitive.str[9].html",
            "std/ops/trait.Shift.html",
            "std/ops/trait.Multiply.html",
            "std/primitive.str[32].html",
            "std/primitive.str[46].html",
            "std/primitive.str[7].html",
            "std/primitive.str[14].html",
            "std/ops/trait.Add.html",
            "std/primitive.str[23].html",
            "std/primitive.str[52].html",
            "std/primitive.str[48].html",
            "std/primitive.str[2].html",
            "std/primitive.str[5].html",
            "impl_traits/bar/struct.Bar.html",
            "std/primitive.str[63].html",
            "std/primitive.str[21].html",
            "std/codec/fn.contract_call.html",
            "std/ops/trait.BitwiseXor.html",
            "std/primitive.str[61].html",
            "std/primitive.str[45].html",
            "std/ops/trait.BitwiseOr.html",
            "std/primitive.str[6].html",
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
            <!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="generator" content="swaydoc"><meta name="description" content="API documentation for the Sway `Bar` struct in `bar`."><meta name="keywords" content="sway, swaylang, sway-lang, Bar"><link rel="icon" href="../../static.files/sway-logo.svg"><title>Bar in bar - Sway</title><link rel="stylesheet" type="text/css" href="../../static.files/normalize.css"><link rel="stylesheet" type="text/css" href="../../static.files/swaydoc.css" id="mainThemeStyle"><link rel="stylesheet" type="text/css" href="../../static.files/ayu.css"><link rel="stylesheet" href="../../static.files/ayu.min.css"></head><body class="swaydoc struct"><nav class="sidebar"><a class="sidebar-logo" href="../../impl_traits_generic/index.html"><div class="logo-container"><img class="sway-logo" src="../../static.files/sway-logo.svg" alt="logo"></div></a><h2 class="location">Struct Bar</h2><div class="sidebar-elems"><section><h3><a href="#trait-implementations">Trait Implementations</a></h3><ul class="block method"><li><a href="#impl-AbiEncode">AbiEncode</a></li><li><a href="#impl-AbiDecode">AbiDecode</a></li><li><a href="#impl-Foo">Foo</a></li><li><a href="#impl-Baz">Baz</a></li><li><a href="#impl-Bar">Bar</a></li><li><a href="#impl-Add">Add</a></li><li><a href="#impl-Subtract">Subtract</a></li></ul></section></div></nav><main><div class="width-limiter"><script src="../../search.js" type="text/javascript"></script><script>function onSearchFormSubmit(event){event.preventDefault();const searchQuery=document.getElementById("search-input").value;const url=new URL(window.location.href);if(searchQuery){url.searchParams.set('search',searchQuery)}else{url.searchParams.delete('search')}history.pushState({search:searchQuery},"",url);window.dispatchEvent(new HashChangeEvent("hashchange"))}document.addEventListener('DOMContentLoaded',()=>{const searchbar=document.getElementById("search-input");const searchForm=document.getElementById("search-form");searchbar.addEventListener("keyup",function(event){onSearchFormSubmit(event)});searchbar.addEventListener("search",function(event){onSearchFormSubmit(event)});function onQueryParamsChange(){const searchParams=new URLSearchParams(window.location.search);const query=searchParams.get("search");const searchSection=document.getElementById('search');const mainSection=document.getElementById('main-content');const searchInput=document.getElementById('search-input');if(query){searchInput.value=query;const results=Object.values(SEARCH_INDEX).flat().filter(item=>{const lowerQuery=query.toLowerCase();return item.name.toLowerCase().includes(lowerQuery)});const header=`<h1>Results for ${query}</h1>`;if(results.length>0){const resultList=results.map(item=>{const formattedName=`<span class="type ${item.type_name}">${item.name}</span>`;const name=item.type_name==="module"?[...item.module_info.slice(0,-1),formattedName].join("::"):[...item.module_info,formattedName].join("::");const path=["../..",...item.module_info,item.html_filename].join("/");const left=`<td><span>${name}</span></td>`;const right=`<td><p>${item.preview}</p></td>`;return`<tr onclick="window.location='${path}';">${left}${right}</tr>`}).join('');searchSection.innerHTML=`${header}<table>${resultList}</table>`}else{searchSection.innerHTML=`${header}<p>No results found.</p>`}searchSection.setAttribute("class","search-results");mainSection.setAttribute("class","content hidden")}else{searchSection.setAttribute("class","search-results hidden");mainSection.setAttribute("class","content")}}window.addEventListener('hashchange',onQueryParamsChange);onQueryParamsChange()})</script><nav class="sub"><form id="search-form" class="search-form" onsubmit="onSearchFormSubmit(event)"><div class="search-container"><input id="search-input" class="search-input" name="search" autocomplete="off" spellcheck="false" placeholder="Search the docs..." type="search"></div></form></nav><section id="main-content" class="content"><div class="main-heading"><h1 class="fqn"><span class="in-band">Struct <a class="mod" href="../index.html">impl_traits_generic</a><span>::</span><a class="mod" href="index.html">bar</a><span>::</span><a class="struct" href="#">Bar</a></span></h1></div><div class="docblock item-decl"><pre class="sway struct"><code>pub struct Bar&lt;T&gt; {}</code></pre></div><h2 id="trait-implementations" class="small-section-header">Trait Implementations<a href="#trait-implementations" class="anchor"></a></h2><div id="trait-implementations-list"><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-AbiEncode" class="impl has-srclink"><a href="#impl-AbiEncode" class="anchor"></a><h3 class="code-header in-band">impl AbiEncode for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><div id="method.abi_encode" class="method trait-impl"><a href="#method.abi_encode" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.abi_encode">abi_encode</a>(self, buffer: Buffer) -&gt; Buffer</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-AbiDecode" class="impl has-srclink"><a href="#impl-AbiDecode" class="anchor"></a><h3 class="code-header in-band">impl AbiDecode for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><div id="method.abi_decode" class="method trait-impl"><a href="#method.abi_decode" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.abi_decode">abi_decode</a>(refmut _buffer: BufferReader) -&gt; Self</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Foo" class="impl has-srclink"><a href="#impl-Foo" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Foo.html">Foo</a> for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><details class="swaydoc-toggle method-toggle" open><summary><div id="method.foo" class="method trait-impl"><a href="#method.foo" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.foo">foo</a>()</h4></div></summary><div class="docblock"><p>something more about foo();</p>
            </div></details></div></details><div id="impl-Baz" class="impl has-srclink"><a href="#impl-Baz" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Baz.html">Baz</a> for Bar&lt;T&gt;</h3></div><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Bar" class="impl has-srclink"><a href="#impl-Bar" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../bar/trait.Bar.html">Bar</a> for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><div id="method.foo_bar" class="method trait-impl"><a href="#method.foo_bar" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.foo_bar">foo_bar</a>()</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Add" class="impl has-srclink"><a href="#impl-Add" class="anchor"></a><h3 class="code-header in-band">impl Add for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><div id="method.add" class="method trait-impl"><a href="#method.add" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.add">add</a>(self, other: Self) -&gt; Self</h4></div></div></details><details class="swaydoc-toggle implementors-toggle" open><summary><div id="impl-Subtract" class="impl has-srclink"><a href="#impl-Subtract" class="anchor"></a><h3 class="code-header in-band">impl Subtract for Bar&lt;T&gt;</h3></div></summary><div class="impl-items"><div id="method.subtract" class="method trait-impl"><a href="#method.subtract" class="anchor"></a><h4 class="code-header">pub fn <a class="fnname" href="#method.subtract">subtract</a>(self, other: Self) -&gt; Self</h4></div></div></details></div></section><section id="search" class="search-results"></section></div></main><script src="../../static.files/highlight.js"></script><script>hljs.highlightAll();</script></body></html>"##]],
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
