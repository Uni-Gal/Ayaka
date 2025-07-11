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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="intro.html"><strong aria-hidden="true">1.</strong> 介绍</a></li><li class="chapter-item expanded "><a href="cook/summary.html"><strong aria-hidden="true">2.</strong> 从零开始</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="cook/01-install-rust.html"><strong aria-hidden="true">2.1.</strong> 安装 Rust</a></li><li class="chapter-item expanded "><a href="cook/02-install-makefile.html"><strong aria-hidden="true">2.2.</strong> 安装 Makefile</a></li></ol></li><li class="chapter-item expanded "><a href="quick_start.html"><strong aria-hidden="true">3.</strong> 从源代码开始</a></li><li class="chapter-item expanded "><a href="config/summary.html"><strong aria-hidden="true">4.</strong> 配置</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="config/structure.html"><strong aria-hidden="true">4.1.</strong> 文件结构</a></li><li class="chapter-item expanded "><a href="config/character.html"><strong aria-hidden="true">4.2.</strong> 指定角色</a></li><li class="chapter-item expanded "><a href="config/resources.html"><strong aria-hidden="true">4.3.</strong> 资源</a></li><li class="chapter-item expanded "><a href="config/i18n.html"><strong aria-hidden="true">4.4.</strong> 国际化</a></li><li class="chapter-item expanded "><a href="config/switches.html"><strong aria-hidden="true">4.5.</strong> 分支</a></li><li class="chapter-item expanded "><a href="config/script.html"><strong aria-hidden="true">4.6.</strong> 脚本</a></li></ol></li><li class="chapter-item expanded "><a href="runtime/summary.html"><strong aria-hidden="true">5.</strong> 运行时</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="runtime/run.html"><strong aria-hidden="true">5.1.</strong> 运行一个游戏</a></li></ol></li><li class="chapter-item expanded "><a href="plugin/summary.html"><strong aria-hidden="true">6.</strong> 插件</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="plugin/script_plugin.html"><strong aria-hidden="true">6.1.</strong> Script 插件</a></li><li class="chapter-item expanded "><a href="plugin/text_plugin.html"><strong aria-hidden="true">6.2.</strong> Text 插件</a></li><li class="chapter-item expanded "><a href="plugin/line_plugin.html"><strong aria-hidden="true">6.3.</strong> Line 插件</a></li><li class="chapter-item expanded "><a href="plugin/action_plugin.html"><strong aria-hidden="true">6.4.</strong> Action 插件</a></li><li class="chapter-item expanded "><a href="plugin/game_plugin.html"><strong aria-hidden="true">6.5.</strong> Game 插件</a></li></ol></li><li class="chapter-item expanded "><a href="gui/summary.html"><strong aria-hidden="true">7.</strong> GUI</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="gui/live2d.html"><strong aria-hidden="true">7.1.</strong> Live2D</a></li></ol></li><li class="chapter-item expanded "><a href="packaging.html"><strong aria-hidden="true">8.</strong> 打包</a></li><li class="chapter-item expanded "><a href="platforms.html"><strong aria-hidden="true">9.</strong> 支持的平台</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
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
