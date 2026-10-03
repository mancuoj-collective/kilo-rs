// Progressive enhancements for the course pages (no dependencies):
//   1. a "#" anchor on every section heading (deep links);
//   2. a copy button on every code block;
//   3. a scroll-spy that highlights the current section in the page TOC.
(function () {
  "use strict";

  // 1. Section anchors.
  document.querySelectorAll("h2[id]").forEach(function (heading) {
    var anchor = document.createElement("a");
    anchor.className = "anchor";
    anchor.href = "#" + heading.id;
    anchor.textContent = "#";
    anchor.setAttribute("aria-label", "本节链接");
    heading.appendChild(anchor);
  });

  // 2. Copy buttons.
  document.querySelectorAll("pre").forEach(function (pre) {
    var button = document.createElement("button");
    button.type = "button";
    button.className = "copy";
    button.textContent = "复制";
    button.addEventListener("click", function () {
      var code = pre.querySelector("code");
      var text = (code || pre).innerText;
      var done = function () {
        button.textContent = "已复制";
        setTimeout(function () {
          button.textContent = "复制";
        }, 1200);
      };
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, function () {});
      } else {
        var area = document.createElement("textarea");
        area.value = text;
        document.body.appendChild(area);
        area.select();
        try {
          document.execCommand("copy");
          done();
        } catch (error) {
          /* clipboard unavailable */
        }
        document.body.removeChild(area);
      }
    });
    pre.appendChild(button);
  });

  // 3. Scroll-spy for the page TOC.
  var toc = document.querySelector(".toc");
  if (!toc) {
    return;
  }
  var links = Array.prototype.slice.call(toc.querySelectorAll('a[href^="#"]'));
  var targets = links
    .map(function (link) {
      return document.getElementById(link.getAttribute("href").slice(1));
    })
    .filter(Boolean);
  if (!targets.length) {
    return;
  }
  var spy = function () {
    var y = window.scrollY + 96;
    var active = targets[0];
    targets.forEach(function (target) {
      if (target.offsetTop <= y) {
        active = target;
      }
    });
    links.forEach(function (link) {
      link.classList.toggle("active", link.getAttribute("href") === "#" + active.id);
    });
  };
  document.addEventListener("scroll", spy, { passive: true });
  window.addEventListener("resize", spy);
  spy();
})();
