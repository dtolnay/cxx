#!/usr/bin/env node

const fs = require('fs');
const cheerio = require('cheerio');
const hljs = require('./build/highlight.js');
const Entities = require('html-entities').AllHtmlEntities;
const entities = new Entities();

const githublink = `\
<li class="part-title">\
<a href="https://github.com/dtolnay/cxx">\
<i class="fa fa-github"></i>\
https://github.com/dtolnay/cxx\
</a>\
</li>`;

const dirs = ['build'];
while (dirs.length) {
  const dir = dirs.pop();
  fs.readdirSync(dir).forEach((entry) => {
    path = dir + '/' + entry;
    const stat = fs.statSync(path);
    if (stat.isDirectory()) {
      dirs.push(path);
      return;
    }

    if (!path.endsWith('.html')) {
      return;
    }

    const index = fs.readFileSync(path, 'utf8');
    const $ = cheerio.load(index, { decodeEntities: false });

    $('nav#sidebar ol.chapter').append(githublink);
    $('pre code').each(function () {
      const node = $(this);
      const langClass = node.attr('class').split(' ', 2)[0];
      if (!langClass.startsWith('language-')) {
        return;
      }
      const lang = langClass.replace('language-', '');
      const lines = node.html().split('\n');
      const boring = lines.map((line) =>
        line.includes('<span class="boring">')
      );
      const target = entities.decode(node.text());
      const highlighted = hljs.highlight(lang, target).value;
      const result = highlighted
        .split('\n')
        .map(function (line, i) {
          if (boring[i]) {
            line = '<span class="boring">' + line;
          }
          if (i > 0 && boring[i - 1]) {
            line = '</span>' + line;
          }
          return line;
        })
        .join('\n');
      node.text(result).removeClass(langClass).addClass('hidelines');
    });
    $('code').each(function () {
      $(this).addClass('hljs');
    });

    const out = $.html();
    fs.writeFileSync(path, out);
  });
}
