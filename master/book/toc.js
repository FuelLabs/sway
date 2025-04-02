// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">The Sway Programming Language</a></li><li class="chapter-item expanded "><a href="introduction/index.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="introduction/getting_started.html"><strong aria-hidden="true">1.1.</strong> Getting Started</a></li><li class="chapter-item expanded "><a href="introduction/fuel_toolchain.html"><strong aria-hidden="true">1.2.</strong> The Fuel Toolchain</a></li><li class="chapter-item expanded "><a href="introduction/forc_project.html"><strong aria-hidden="true">1.3.</strong> A Forc Project</a></li><li class="chapter-item expanded "><a href="introduction/standard_library.html"><strong aria-hidden="true">1.4.</strong> Standard Library</a></li><li class="chapter-item expanded "><a href="introduction/sway_standards.html"><strong aria-hidden="true">1.5.</strong> Sway Language Standards</a></li></ol></li><li class="chapter-item expanded "><a href="examples/index.html"><strong aria-hidden="true">2.</strong> Examples</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="examples/counter.html"><strong aria-hidden="true">2.1.</strong> Counter</a></li><li class="chapter-item expanded "><a href="examples/fizzbuzz.html"><strong aria-hidden="true">2.2.</strong> FizzBuzz</a></li><li class="chapter-item expanded "><a href="examples/wallet_smart_contract.html"><strong aria-hidden="true">2.3.</strong> Wallet Smart Contract</a></li><li class="chapter-item expanded "><a href="examples/liquidity_pool.html"><strong aria-hidden="true">2.4.</strong> Liquidity Pool</a></li><li class="chapter-item expanded "><a href="examples/sway_applications.html"><strong aria-hidden="true">2.5.</strong> Sway Applications</a></li></ol></li><li class="chapter-item expanded "><a href="sway-program-types/index.html"><strong aria-hidden="true">3.</strong> Program Types</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="sway-program-types/smart_contracts.html"><strong aria-hidden="true">3.1.</strong> Contracts</a></li><li class="chapter-item expanded "><a href="sway-program-types/libraries.html"><strong aria-hidden="true">3.2.</strong> Libraries</a></li><li class="chapter-item expanded "><a href="sway-program-types/scripts.html"><strong aria-hidden="true">3.3.</strong> Scripts</a></li><li class="chapter-item expanded "><a href="sway-program-types/predicates.html"><strong aria-hidden="true">3.4.</strong> Predicates</a></li></ol></li><li class="chapter-item expanded "><a href="basics/index.html"><strong aria-hidden="true">4.</strong> Sway Language Basics</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="basics/variables.html"><strong aria-hidden="true">4.1.</strong> Variables</a></li><li class="chapter-item expanded "><a href="basics/built_in_types.html"><strong aria-hidden="true">4.2.</strong> Built-in Types</a></li><li class="chapter-item expanded "><a href="basics/commonly_used_library_types.html"><strong aria-hidden="true">4.3.</strong> Commonly Used Library Types</a></li><li class="chapter-item expanded "><a href="basics/blockchain_types.html"><strong aria-hidden="true">4.4.</strong> Blockchain Types</a></li><li class="chapter-item expanded "><a href="basics/converting_types.html"><strong aria-hidden="true">4.5.</strong> Converting Types</a></li><li class="chapter-item expanded "><a href="basics/functions.html"><strong aria-hidden="true">4.6.</strong> Functions</a></li><li class="chapter-item expanded "><a href="basics/structs_tuples_and_enums.html"><strong aria-hidden="true">4.7.</strong> Structs, Tuples, and Enums</a></li><li class="chapter-item expanded "><a href="basics/methods_and_associated_functions.html"><strong aria-hidden="true">4.8.</strong> Methods and Associated Functions</a></li><li class="chapter-item expanded "><a href="basics/constants.html"><strong aria-hidden="true">4.9.</strong> Constants</a></li><li class="chapter-item expanded "><a href="basics/comments_and_logging.html"><strong aria-hidden="true">4.10.</strong> Comments and Logging</a></li><li class="chapter-item expanded "><a href="basics/control_flow.html"><strong aria-hidden="true">4.11.</strong> Control Flow</a></li></ol></li><li class="chapter-item expanded "><a href="blockchain-development/index.html"><strong aria-hidden="true">5.</strong> Blockchain Development with Sway</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="blockchain-development/hashing_and_cryptography.html"><strong aria-hidden="true">5.1.</strong> Hashing and Cryptography</a></li><li class="chapter-item expanded "><a href="blockchain-development/storage.html"><strong aria-hidden="true">5.2.</strong> Contract Storage</a></li><li class="chapter-item expanded "><a href="blockchain-development/purity.html"><strong aria-hidden="true">5.3.</strong> Function Purity</a></li><li class="chapter-item expanded "><a href="blockchain-development/identifiers.html"><strong aria-hidden="true">5.4.</strong> Identifiers</a></li><li class="chapter-item expanded "><a href="blockchain-development/native_assets.html"><strong aria-hidden="true">5.5.</strong> Native Assets</a></li><li class="chapter-item expanded "><a href="blockchain-development/access_control.html"><strong aria-hidden="true">5.6.</strong> Access Control</a></li><li class="chapter-item expanded "><a href="blockchain-development/calling_contracts.html"><strong aria-hidden="true">5.7.</strong> Calling Contracts</a></li><li class="chapter-item expanded "><a href="blockchain-development/external_code.html"><strong aria-hidden="true">5.8.</strong> External Code</a></li></ol></li><li class="chapter-item expanded "><a href="advanced/index.html"><strong aria-hidden="true">6.</strong> Advanced Concepts</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="advanced/advanced_types.html"><strong aria-hidden="true">6.1.</strong> Advanced Types</a></li><li class="chapter-item expanded "><a href="advanced/advanced_storage.html"><strong aria-hidden="true">6.2.</strong> Advanced Storage</a></li><li class="chapter-item expanded "><a href="advanced/generic_types.html"><strong aria-hidden="true">6.3.</strong> Generic Types</a></li><li class="chapter-item expanded "><a href="advanced/traits.html"><strong aria-hidden="true">6.4.</strong> Traits</a></li><li class="chapter-item expanded "><a href="advanced/associated_types.html"><strong aria-hidden="true">6.5.</strong> Associated Types</a></li><li class="chapter-item expanded "><a href="advanced/generics_and_trait_constraints.html"><strong aria-hidden="true">6.6.</strong> Generics and Trait Constraints</a></li><li class="chapter-item expanded "><a href="advanced/assembly.html"><strong aria-hidden="true">6.7.</strong> Assembly</a></li><li class="chapter-item expanded "><a href="advanced/never_type.html"><strong aria-hidden="true">6.8.</strong> Never Type</a></li></ol></li><li class="chapter-item expanded "><a href="common-collections/index.html"><strong aria-hidden="true">7.</strong> Common Collections</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="common-collections/vec.html"><strong aria-hidden="true">7.1.</strong> Vectors on the Heap</a></li><li class="chapter-item expanded "><a href="common-collections/storage_vec.html"><strong aria-hidden="true">7.2.</strong> Storage Vectors</a></li><li class="chapter-item expanded "><a href="common-collections/storage_map.html"><strong aria-hidden="true">7.3.</strong> Storage Maps</a></li></ol></li><li class="chapter-item expanded "><a href="testing/index.html"><strong aria-hidden="true">8.</strong> Testing</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="testing/unit-testing.html"><strong aria-hidden="true">8.1.</strong> Unit Testing</a></li><li class="chapter-item expanded "><a href="testing/testing-with-rust.html"><strong aria-hidden="true">8.2.</strong> Testing with Rust</a></li></ol></li><li class="chapter-item expanded "><a href="debugging/index.html"><strong aria-hidden="true">9.</strong> Debugging</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="debugging/debugging_with_cli.html"><strong aria-hidden="true">9.1.</strong> Debugging with CLI</a></li><li class="chapter-item expanded "><a href="debugging/debugging_with_ide.html"><strong aria-hidden="true">9.2.</strong> Debugging with IDE</a></li></ol></li><li class="chapter-item expanded "><a href="lsp/index.html"><strong aria-hidden="true">10.</strong> Sway LSP</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="lsp/installation.html"><strong aria-hidden="true">10.1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="lsp/features.html"><strong aria-hidden="true">10.2.</strong> Features</a></li><li class="chapter-item expanded "><a href="lsp/troubleshooting.html"><strong aria-hidden="true">10.3.</strong> Troubleshooting</a></li></ol></li><li class="chapter-item expanded "><a href="reference/index.html"><strong aria-hidden="true">11.</strong> Sway Reference</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="reference/sway_libs.html"><strong aria-hidden="true">11.1.</strong> Sway Libraries</a></li><li class="chapter-item expanded "><a href="reference/compiler_intrinsics.html"><strong aria-hidden="true">11.2.</strong> Compiler Intrinsics</a></li><li class="chapter-item expanded "><a href="reference/attributes.html"><strong aria-hidden="true">11.3.</strong> Attributes</a></li><li class="chapter-item expanded "><a href="reference/style_guide.html"><strong aria-hidden="true">11.4.</strong> Style Guide</a></li><li class="chapter-item expanded "><a href="reference/known_issues_and_workarounds.html"><strong aria-hidden="true">11.5.</strong> Known Issues and Workarounds</a></li><li class="chapter-item expanded "><a href="reference/undefined_behavior.html"><strong aria-hidden="true">11.6.</strong> Behavior Considered Undefined</a></li><li class="chapter-item expanded "><a href="reference/solidity_differences.html"><strong aria-hidden="true">11.7.</strong> Differences From Solidity</a></li><li class="chapter-item expanded "><a href="reference/rust_differences.html"><strong aria-hidden="true">11.8.</strong> Differences From Rust</a></li><li class="chapter-item expanded "><a href="reference/contributing_to_sway.html"><strong aria-hidden="true">11.9.</strong> Contributing To Sway</a></li><li class="chapter-item expanded "><a href="reference/keywords.html"><strong aria-hidden="true">11.10.</strong> Keywords</a></li></ol></li><li class="chapter-item expanded "><a href="forc/index.html"><strong aria-hidden="true">12.</strong> Forc Reference</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="forc/manifest_reference.html"><strong aria-hidden="true">12.1.</strong> Manifest Reference</a></li><li class="chapter-item expanded "><a href="forc/workspaces.html"><strong aria-hidden="true">12.2.</strong> Workspaces</a></li><li class="chapter-item expanded "><a href="forc/dependencies.html"><strong aria-hidden="true">12.3.</strong> Dependencies</a></li><li class="chapter-item expanded "><a href="forc/commands/index.html"><strong aria-hidden="true">12.4.</strong> Commands</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="forc/commands/forc_addr2line.html"><strong aria-hidden="true">12.4.1.</strong> forc addr2line</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_build.html"><strong aria-hidden="true">12.4.2.</strong> forc build</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_check.html"><strong aria-hidden="true">12.4.3.</strong> forc check</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_clean.html"><strong aria-hidden="true">12.4.4.</strong> forc clean</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_completions.html"><strong aria-hidden="true">12.4.5.</strong> forc completions</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_contract-id.html"><strong aria-hidden="true">12.4.6.</strong> forc contract-id</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_init.html"><strong aria-hidden="true">12.4.7.</strong> forc init</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_new.html"><strong aria-hidden="true">12.4.8.</strong> forc new</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_parse-bytecode.html"><strong aria-hidden="true">12.4.9.</strong> forc parse-bytecode</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_plugins.html"><strong aria-hidden="true">12.4.10.</strong> forc plugins</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_predicate-root.html"><strong aria-hidden="true">12.4.11.</strong> forc predicate-root</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_test.html"><strong aria-hidden="true">12.4.12.</strong> forc test</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_update.html"><strong aria-hidden="true">12.4.13.</strong> forc update</a></li><li class="chapter-item expanded "><a href="forc/commands/forc_template.html"><strong aria-hidden="true">12.4.14.</strong> forc template</a></li></ol></li><li class="chapter-item expanded "><a href="forc/plugins/index.html"><strong aria-hidden="true">12.5.</strong> Plugins</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="forc/plugins/forc_client/index.html"><strong aria-hidden="true">12.5.1.</strong> forc client</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="forc/plugins/forc_client/forc_deploy.html"><strong aria-hidden="true">12.5.1.1.</strong> forc deploy</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_client/forc_run.html"><strong aria-hidden="true">12.5.1.2.</strong> forc run</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_client/forc_submit.html"><strong aria-hidden="true">12.5.1.3.</strong> forc submit</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_client/forc_call.html"><strong aria-hidden="true">12.5.1.4.</strong> forc call</a></li></ol></li><li class="chapter-item expanded "><a href="forc/plugins/forc_crypto.html"><strong aria-hidden="true">12.5.2.</strong> forc crypto</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_debug.html"><strong aria-hidden="true">12.5.3.</strong> forc debug</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_doc.html"><strong aria-hidden="true">12.5.4.</strong> forc doc</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_explore.html"><strong aria-hidden="true">12.5.5.</strong> forc explore</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_fmt.html"><strong aria-hidden="true">12.5.6.</strong> forc fmt</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_lsp.html"><strong aria-hidden="true">12.5.7.</strong> forc lsp</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_migrate.html"><strong aria-hidden="true">12.5.8.</strong> forc migrate</a></li><li class="chapter-item expanded "><a href="forc/plugins/forc_node.html"><strong aria-hidden="true">12.5.9.</strong> forc node</a></li></ol></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
