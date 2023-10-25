#![cfg(test)]
use crate::{cli::Command, tests::expects::check};
use expect_test::expect;
use std::path::PathBuf;

/// The path to the generated HTML of the type the traits are implemented on.
const IMPL_FOR: &str = "impl_traits/bar/struct.Bar.html";
const IMPL_TRAIT_FILE_PATH: &str = "src/tests/data/impl_traits";

#[test]
fn impl_traits_default() {
    const DOC_DIR_NAME: &str = "impl_traits_default";
    let command = Command {
        manifest_path: Some(IMPL_TRAIT_FILE_PATH.into()),
        doc_path: Some(DOC_DIR_NAME.into()),
        ..Default::default()
    };
    let path_to_file = PathBuf::from(IMPL_FOR);
    check(
        command,
        path_to_file,
        &expect![[r##"
            <!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="generator" content="swaydoc"><meta name="description" content="API documentation for the Sway `Bar` struct in `bar`."><meta name="keywords" content="sway, swaylang, sway-lang, Bar"><link rel="icon" href="../../static.files/sway-logo.svg"><title>Bar in bar - Sway</title><link rel="stylesheet" type="text/css" href="../../static.files/normalize.css"><link rel="stylesheet" type="text/css" href="../../static.files/swaydoc.css" id="mainThemeStyle"><link rel="stylesheet" type="text/css" href="../../static.files/ayu.css"><link rel="stylesheet" href="../../static.files/ayu.min.css"></head><body class="swaydoc struct"><nav class="sidebar"><a class="sidebar-logo" href="../../impl_traits/index.html"><div class="logo-container"><img class="sway-logo" src="../../static.files/sway-logo.svg" alt="logo"></div></a><h2 class="location">Struct Bar</h2><div class="sidebar-elems"><section><div class="block"><ul></ul></div></section></div></nav><main><div class="width-limiter"><section id="main-content" class="content"><div class="main-heading"><h1 class="fqn"><span class="in-band">Struct <a class="mod" href="../index.html">impl_traits</a><span>::</span><a class="mod" href="index.html">bar</a><span>::</span><a class="struct" href="#">Bar</a></span></h1></div><div class="docblock item-decl"><pre class="sway struct"><code>pub struct Bar {}</code></pre></div><h2 id="trait-implementations" class="small-section-header">Trait Implementations<a href="#trait-implementations" class="anchor"></a></h2><div id="trait-implementations-list"><details class="swaydoc-toggle implementors-toggle"><summary><div id="impl-Foo" class="impl has-srclink"><a href="#impl-Foo" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Foo.html">Foo</a> for Bar</h3></div></summary><div class="impl-items"><details class="swaydoc-toggle method-toggle" open><summary><div id="method.foo" class="method trait-impl"><a href="#method.foo" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.foo">foo</a>()</h4></div></summary><div class="doc-block"><p>something more about foo();</p>
            </div></details></div></details><div id="impl-Baz" class="impl has-srclink"><a href="#impl-Baz" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Baz.html">Baz</a> for Bar</h3></div><details class="swaydoc-toggle implementors-toggle"><summary><div id="impl-Add" class="impl has-srclink"><a href="#impl-Add" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../../core/ops/trait.Add.html">Add</a> for Bar</h3></div></summary><div class="impl-items"><div id="method.add" class="method trait-impl"><a href="#method.add" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.add">add</a>(self, other: Self) -&gt; Self</h4></div></div></details><details class="swaydoc-toggle implementors-toggle"><summary><div id="impl-Subtract" class="impl has-srclink"><a href="#impl-Subtract" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../../core/ops/trait.Subtract.html">Subtract</a> for Bar</h3></div></summary><div class="impl-items"><div id="method.subtract" class="method trait-impl"><a href="#method.subtract" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.subtract">subtract</a>(self, other: Self) -&gt; Self</h4></div></div></details></div></section></div></main><script src="../../static.files/highlight.js"></script><script>hljs.highlightAll();</script></body></html>"##]],
    );
}

#[test]
fn impl_traits_no_deps() {
    const DOC_DIR_NAME: &str = "impl_traits_no_deps";
    let command = Command {
        manifest_path: Some(IMPL_TRAIT_FILE_PATH.into()),
        doc_path: Some(DOC_DIR_NAME.into()),
        no_deps: true,
        ..Default::default()
    };
    let path_to_file = PathBuf::from(IMPL_FOR);
    check(
        command,
        path_to_file,
        &expect![[r##"
            <!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="generator" content="swaydoc"><meta name="description" content="API documentation for the Sway `Bar` struct in `bar`."><meta name="keywords" content="sway, swaylang, sway-lang, Bar"><link rel="icon" href="../../static.files/sway-logo.svg"><title>Bar in bar - Sway</title><link rel="stylesheet" type="text/css" href="../../static.files/normalize.css"><link rel="stylesheet" type="text/css" href="../../static.files/swaydoc.css" id="mainThemeStyle"><link rel="stylesheet" type="text/css" href="../../static.files/ayu.css"><link rel="stylesheet" href="../../static.files/ayu.min.css"></head><body class="swaydoc struct"><nav class="sidebar"><a class="sidebar-logo" href="../../impl_traits/index.html"><div class="logo-container"><img class="sway-logo" src="../../static.files/sway-logo.svg" alt="logo"></div></a><h2 class="location">Struct Bar</h2><div class="sidebar-elems"><section><div class="block"><ul></ul></div></section></div></nav><main><div class="width-limiter"><section id="main-content" class="content"><div class="main-heading"><h1 class="fqn"><span class="in-band">Struct <a class="mod" href="../index.html">impl_traits</a><span>::</span><a class="mod" href="index.html">bar</a><span>::</span><a class="struct" href="#">Bar</a></span></h1></div><div class="docblock item-decl"><pre class="sway struct"><code>pub struct Bar {}</code></pre></div><h2 id="trait-implementations" class="small-section-header">Trait Implementations<a href="#trait-implementations" class="anchor"></a></h2><div id="trait-implementations-list"><details class="swaydoc-toggle implementors-toggle"><summary><div id="impl-Foo" class="impl has-srclink"><a href="#impl-Foo" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Foo.html">Foo</a> for Bar</h3></div></summary><div class="impl-items"><details class="swaydoc-toggle method-toggle" open><summary><div id="method.foo" class="method trait-impl"><a href="#method.foo" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.foo">foo</a>()</h4></div></summary><div class="doc-block"><p>something more about foo();</p>
            </div></details></div></details><div id="impl-Baz" class="impl has-srclink"><a href="#impl-Baz" class="anchor"></a><h3 class="code-header in-band">impl <a class="trait" href="../foo/trait.Baz.html">Baz</a> for Bar</h3></div><details class="swaydoc-toggle implementors-toggle"><summary><div id="impl-Add" class="impl has-srclink"><a href="#impl-Add" class="anchor"></a><h3 class="code-header in-band">impl Add for Bar</h3></div></summary><div class="impl-items"><div id="method.add" class="method trait-impl"><a href="#method.add" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.add">add</a>(self, other: Self) -&gt; Self</h4></div></div></details><details class="swaydoc-toggle implementors-toggle"><summary><div id="impl-Subtract" class="impl has-srclink"><a href="#impl-Subtract" class="anchor"></a><h3 class="code-header in-band">impl Subtract for Bar</h3></div></summary><div class="impl-items"><div id="method.subtract" class="method trait-impl"><a href="#method.subtract" class="anchor"></a><h4 class="code-header">fn <a class="fnname" href="#method.subtract">subtract</a>(self, other: Self) -&gt; Self</h4></div></div></details></div></section></div></main><script src="../../static.files/highlight.js"></script><script>hljs.highlightAll();</script></body></html>"##]],
    );
}
