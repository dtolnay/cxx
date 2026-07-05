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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="index.html"><strong aria-hidden="true">1.</strong> Rust ❤️ C++</a></li><li class="chapter-item expanded "><a href="concepts.html"><strong aria-hidden="true">2.</strong> Core concepts</a></li><li class="chapter-item expanded "><a href="tutorial.html"><strong aria-hidden="true">3.</strong> Tutorial</a></li><li class="chapter-item expanded "><a href="context.html"><strong aria-hidden="true">4.</strong> Other Rust–C++ interop tools</a></li><li class="chapter-item expanded "><a href="building.html"><strong aria-hidden="true">5.</strong> Multi-language build system options</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="build/cargo.html"><strong aria-hidden="true">5.1.</strong> Cargo</a></li><li class="chapter-item expanded "><a href="build/bazel.html"><strong aria-hidden="true">5.2.</strong> Bazel or Buck2</a></li><li class="chapter-item expanded "><a href="build/cmake.html"><strong aria-hidden="true">5.3.</strong> CMake</a></li><li class="chapter-item expanded "><a href="build/other.html"><strong aria-hidden="true">5.4.</strong> More...</a></li></ol></li><li class="chapter-item expanded "><a href="reference.html"><strong aria-hidden="true">6.</strong> Reference: the bridge module</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="extern-rust.html"><strong aria-hidden="true">6.1.</strong> extern "Rust"</a></li><li class="chapter-item expanded "><a href="extern-c++.html"><strong aria-hidden="true">6.2.</strong> extern "C++"</a></li><li class="chapter-item expanded "><a href="shared.html"><strong aria-hidden="true">6.3.</strong> Shared types</a></li><li class="chapter-item expanded "><a href="attributes.html"><strong aria-hidden="true">6.4.</strong> Attributes</a></li><li class="chapter-item expanded "><a href="async.html"><strong aria-hidden="true">6.5.</strong> Async functions</a></li><li class="chapter-item expanded "><a href="binding/result.html"><strong aria-hidden="true">6.6.</strong> Error handling</a></li></ol></li><li class="chapter-item expanded "><a href="bindings.html"><strong aria-hidden="true">7.</strong> Reference: built-in bindings</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="binding/string.html"><strong aria-hidden="true">7.1.</strong> String — rust::String</a></li><li class="chapter-item expanded "><a href="binding/str.html"><strong aria-hidden="true">7.2.</strong> &amp;str — rust::Str</a></li><li class="chapter-item expanded "><a href="binding/slice.html"><strong aria-hidden="true">7.3.</strong> &amp;[T], &amp;mut [T] — rust::Slice&lt;T&gt;</a></li><li class="chapter-item expanded "><a href="binding/cxxstring.html"><strong aria-hidden="true">7.4.</strong> CxxString — std::string</a></li><li class="chapter-item expanded "><a href="binding/box.html"><strong aria-hidden="true">7.5.</strong> Box&lt;T&gt; — rust::Box&lt;T&gt;</a></li><li class="chapter-item expanded "><a href="binding/uniqueptr.html"><strong aria-hidden="true">7.6.</strong> UniquePtr&lt;T&gt; — std::unique_ptr&lt;T&gt;</a></li><li class="chapter-item expanded "><a href="binding/sharedptr.html"><strong aria-hidden="true">7.7.</strong> SharedPtr&lt;T&gt; — std::shared_ptr&lt;T&gt;</a></li><li class="chapter-item expanded "><a href="binding/vec.html"><strong aria-hidden="true">7.8.</strong> Vec&lt;T&gt; — rust::Vec&lt;T&gt;</a></li><li class="chapter-item expanded "><a href="binding/cxxvector.html"><strong aria-hidden="true">7.9.</strong> CxxVector&lt;T&gt; — std::vector&lt;T&gt;</a></li><li class="chapter-item expanded "><a href="binding/rawptr.html"><strong aria-hidden="true">7.10.</strong> *mut T, *const T raw pointers</a></li><li class="chapter-item expanded "><a href="binding/fn.html"><strong aria-hidden="true">7.11.</strong> Function pointers</a></li><li class="chapter-item expanded "><a href="binding/result.html"><strong aria-hidden="true">7.12.</strong> Result&lt;T&gt;</a></li></ol></li></ol>';
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
