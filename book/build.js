#!/usr/bin/env node

const fs = require('fs');
const cheerio = require('cheerio');

const githublink = `\
<li class="part-title">\
<a href="https://github.com/dtolnay/cxx">\
<i class="fa fa-github" style="font-size:20px;padding-right:5px;padding-top:12px;position:relative;top:1px"></i>\
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

    const out = $.html();
    fs.writeFileSync(path, out);
  });
}
